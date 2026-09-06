use crate::config::AppConfig;
use crate::config::AutoreadMode;
use crate::config::BotMode;
use crate::config::PairingMethod;
use crate::config::WarmupMode;
use crate::state::AppState;
use crate::utils::MessageExt;
use chrono::Utc;
use qr2term::print_qr;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::{RwLock, Semaphore};
use whatsapp_rust::client::Client;
use whatsapp_rust::wacore::stanza::GroupNotificationAction;
use whatsapp_rust::wacore::types::events::GroupUpdate;
use whatsapp_rust::wacore::types::events::InboundMessage;
use whatsapp_rust::wacore::types::events::PairingCode;
use whatsapp_rust::wacore::types::events::PairingQrCode;
use whatsapp_rust::wacore::types::message::MessageInfo;
use whatsapp_rust::wacore::{client::context::SendContextResolver, types::events::Event};

static SUPERUSER_LID: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| RwLock::new(vec![]));

static MESSAGE_CONCURRENCY: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(64));

pub async fn event_handler(
    event: Arc<Event>,
    client: Arc<Client>,
    config: Arc<AppConfig>,
    state: Arc<AppState>,
) {
    match &*event {
        Event::Connected(_) => handle_connected(config, client, state).await,
        Event::Messages(batch) => {
            for InboundMessage { message, info, .. } in batch.iter() {
                if config.debug_dump {
                    crate::logger::dump(info, message);
                }
                handle_message(
                    Arc::clone(message),
                    Arc::clone(&client),
                    Arc::clone(&config),
                    Arc::clone(info),
                    Arc::clone(&state),
                )
                .await;
            }
        }
        Event::GroupUpdate(update) => handle_group_exp(update.clone(), state).await,
        Event::PairingCode(PairingCode { code, .. }) => {
            if config.pairing == PairingMethod::Code {
                crate::logger::info("pairing", format!("pair code: {}", code));
            }
        }
        Event::PairingQrCode(PairingQrCode { code, .. }) => {
            if config.pairing == PairingMethod::Qr
                && let Err(e) = print_qr(code)
            {
                crate::logger::error("pairing", format!("failed to print QR code: {}", e));
            }
        }
        _ => {}
    }
}

async fn handle_connected(config: Arc<AppConfig>, client: Arc<Client>, state: Arc<AppState>) {
    let current_name = client.push_name();
    if current_name.is_empty() {
        let _ = client.profile().set_push_name("sora-on-rust").await;
    }

    if state.get_show_online() {
        let _ = client.presence().set_available().await;
    } else {
        let _ = client.presence().set_unavailable().await;
    }
    let mut lids = vec![];
    for su_pn in &config.superuser {
        let found_lid = client.get_lid_for_phone(su_pn).await.map(|j| j.to_string());
        if let Some(lid) = found_lid {
            lids.push(lid);
        } else {
            crate::logger::warn(
                "startup",
                format!("unable to get LID for superuser: {}", su_pn),
            );
        }
    }
    let mut lock = SUPERUSER_LID.write().await;
    *lock = lids;
}

async fn handle_message(
    msg: Arc<whatsapp_rust::waproto::whatsapp::Message>,
    client: Arc<Client>,
    config: Arc<AppConfig>,
    info: Arc<MessageInfo>,
    state: Arc<AppState>,
) {
    let msg_timestamp = Utc::now() - info.timestamp;
    if msg_timestamp.to_std().unwrap_or_default() > state.start_time.elapsed() {
        return;
    }

    if let Some(exp) = msg.get_expiration_timer() {
        state.set_expiration(info.source.chat.to_string(), exp);
    }

    apply_autoread(&client, &info, &state).await;

    let text = match msg.text_content() {
        Some(t) => t,
        None => return,
    };

    let prefixes = state.get_prefixes();
    let found_prefix = prefixes.iter().find(|p| text.starts_with(p.as_str()));
    let is_command = found_prefix.is_some();
    let prefix_len = found_prefix.map(|p| p.len()).unwrap_or(0);

    let base = &text[prefix_len..];

    let needs_warmup = !is_command && state.get_warmup() != WarmupMode::Off;
    let needs_interceptors = !crate::commands::cmd::INTERCEPTORS.is_empty() && !state.cache.is_empty();
    if !is_command && !needs_interceptors && !needs_warmup {
        return;
    }

    let mut parts = base.split_whitespace();
    let cmd_name = parts.next().unwrap_or("").to_lowercase();
    let args: Vec<&str> = parts.collect();
    let body = base
        .strip_prefix(&cmd_name)
        .unwrap_or("")
        .trim()
        .to_string();

    let client_c = Arc::clone(&client);
    let state_c = Arc::clone(&state);
    let info_c = Arc::clone(&info);
    let msg_c = Arc::clone(&msg);
    let config_c = Arc::clone(&config);
    let cmd_name_c = cmd_name.clone();
    let args_owned: Vec<String> = args.into_iter().map(str::to_owned).collect();

    tokio::spawn(async move {
        let _permit = MESSAGE_CONCURRENCY.acquire().await;

        let args_ref: Vec<&str> = args_owned.iter().map(String::as_str).collect();

        let ctx = crate::commands::cmd::Context {
            client: Arc::clone(&client_c),
            msg: &msg_c,
            info: &info_c,
            state: Arc::clone(&state_c),
            args: &args_ref,
            body: &body,
        };

        if needs_interceptors {
            for interceptor in crate::commands::cmd::INTERCEPTORS {
                if let Ok(true) = interceptor.intercept(ctx.clone()).await {
                    return;
                }
            }
        }

        if is_command {
            if let Some(cmd) = crate::commands::cmd::COMMAND_MAP.get(&cmd_name_c) {
                let privileged =
                    is_privileged(info_c.source.sender.user.as_str(), &info_c, &config_c).await;

                if state_c.get_mode() == BotMode::SelfMode && !privileged {
                    return;
                }

                let access = cmd.access();

                if access.owner_only && !privileged {
                    let _ = ctx.reply("Owner only command.").await;
                    return;
                }

                if access.group_only && !info_c.source.is_group {
                    let _ = ctx.reply("This command only works in groups.").await;
                    return;
                }

                if access.dm_only && info_c.source.is_group {
                    let _ = ctx.reply("This command only works in DM.").await;
                    return;
                }

                if access.admin_only && !info_c.source.is_group {
                    let _ = ctx.reply("This command only works in groups.").await;
                    return;
                }

                if access.admin_only && !privileged && !is_group_admin(&client_c, &info_c).await
                {
                    let _ = ctx.reply("Admin only command.").await;
                    return;
                }

                let _ = client_c
                    .chatstate()
                    .send_composing(&info_c.source.chat)
                    .await;
                if let Err(e) = cmd.execute(ctx).await {
                    crate::logger::error(&cmd_name_c, format!("command error: {}", e));
                }
                let _ = client_c.chatstate().send_paused(&info_c.source.chat).await;
            }
        } else if needs_warmup {
            let chat_jid = info_c.source.chat.clone();
            let msg_id = info_c.id.clone();
            let sender_jid = info_c.source.sender.to_string();

            let _ = crate::utils::send_warmup(client_c, chat_jid, msg_id, Some(sender_jid)).await;
        }
    });
}

async fn apply_autoread(client: &Arc<Client>, info: &Arc<MessageInfo>, state: &Arc<AppState>) {
    if info.source.is_from_me {
        return;
    }

    let mode = state.get_autoread();
    let should_read = match mode {
        AutoreadMode::Off => false,
        AutoreadMode::All => true,
        AutoreadMode::Group => info.source.is_group,
        AutoreadMode::Dm => !info.source.is_group,
    };

    if !should_read {
        return;
    }

    let chat = info.source.chat.clone();
    let sender = if info.source.is_group {
        Some(info.source.sender.clone())
    } else {
        None
    };
    let msg_id = info.id.clone();
    let client_c = Arc::clone(client);

    tokio::spawn(async move {
        let sender_ref = sender.as_ref();
        if let Err(e) = client_c
            .mark_as_read(&chat, sender_ref, &[msg_id.as_str()])
            .await
        {
            crate::logger::error("autoread", format!("failed to mark as read: {}", e));
        }
    });
}

async fn handle_group_exp(update: GroupUpdate, state: Arc<AppState>) {
    if let GroupNotificationAction::Ephemeral {
        expiration,
        trigger: _,
    } = &update.action
    {
        state.set_expiration(update.group_jid.to_string(), *expiration);
    }
}

async fn is_privileged(sender: &str, info: &MessageInfo, config: &Arc<AppConfig>) -> bool {
    let me = info.source.is_from_me;
    let su = if info.source.sender.is_lid() {
        let lock = SUPERUSER_LID.read().await;
        lock.contains(&sender.to_string())
    } else {
        config.superuser.contains(&sender.to_string())
    };

    me || su
}

async fn is_group_admin(client: &Arc<Client>, info: &MessageInfo) -> bool {
    let Ok(metadata) = client.groups().get_metadata(&info.source.chat).await else {
        return false;
    };

    metadata
        .participants
        .iter()
        .find(|p| p.jid == info.source.sender)
        .is_some_and(|p| p.participant_type.is_admin())
}
