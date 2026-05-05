use crate::config::AppConfig;
use crate::config::BotMode;
use crate::config::WarmupMode;
use crate::state::AppState;
use crate::utils::MessageExt;
use chrono::Utc;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::RwLock;
use wacore::stanza::GroupNotificationAction;
use wacore::types::events::GroupUpdate;
use wacore::types::message::MessageInfo;
use wacore::{client::context::SendContextResolver, types::events::Event};
use whatsapp_rust::client::Client;

static SUPERUSER_LID: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| RwLock::new(vec![]));

pub async fn event_handler(
    event: Event,
    client: Arc<Client>,
    config: Arc<AppConfig>,
    state: Arc<AppState>,
) {
    match event {
        Event::Connected(_) => handle_connected(config, client).await,
        Event::Message(msg, info) => {
            crate::logger::dump(&info, &msg);
            handle_message(*msg, client, config, info, state).await;
        }
        Event::GroupUpdate(update) => handle_group_exp(update, state).await,
        Event::PairingCode { code, .. } => {
            println!("Pair code: {}", code);
        }
        _ => {}
    }
}

async fn handle_connected(config: Arc<AppConfig>, client: Arc<Client>) {
    let current_name = client.get_push_name().await;
    if current_name.is_empty() {
        let _ = client.profile().set_push_name("sora-on-rust").await;
    }

    let _ = client.presence().set_available().await;
    let mut lids = vec![];
    for su_pn in &config.superuser {
        let mut found_lid = client.get_lid_for_phone(su_pn).await.map(|j| j.to_string());
        if found_lid.is_none() {
            match client.contacts().get_info(&[su_pn.as_str()]).await {
                Ok(contacts) => {
                    if let Some(contact) = contacts.into_iter().next()
                        && let Some(lid) = contact.lid
                    {
                        found_lid = Some(lid.user);
                    }
                }
                Err(e) => log::error!("Unable retrieve contact info from server: {}", e),
            }
        }
        if let Some(lid) = found_lid {
            lids.push(lid);
        } else {
            log::warn!("Unable to get LID for superuser: {}", su_pn);
        }
    }
    let mut lock = SUPERUSER_LID.write().await;
    *lock = lids;
}

async fn handle_message(
    msg: waproto::whatsapp::Message,
    client: Arc<Client>,
    config: Arc<AppConfig>,
    info: MessageInfo,
    state: Arc<AppState>,
) {
    let msg_timestamp = Utc::now() - info.timestamp;
    if msg_timestamp.to_std().unwrap_or_default() > state.start_time.elapsed() {
        return;
    }

    if let Some(exp) = msg.get_expiration_timer() {
        state.set_expiration(info.source.chat.to_string(), exp);
    }

    let text = match msg.text_content() {
        Some(t) => t,
        None => return,
    };

    let prefixes = state.get_prefixes();
    let found_prefix = prefixes
        .iter()
        .find(|p| text.starts_with(p.as_str()))
        .cloned();
    let is_command = found_prefix.is_some();

    let prefix_str = found_prefix.unwrap_or_default();

    let base = text.strip_prefix(&prefix_str).unwrap_or(text);
    let mut parts = base.split_whitespace();
    let cmd_name = parts.next().unwrap_or("").to_lowercase();
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();
    let body = base
        .strip_prefix(&cmd_name)
        .unwrap_or("")
        .trim()
        .to_string();

    let client_c = Arc::clone(&client);
    let state_c = Arc::clone(&state);
    let info_c = info.clone();
    let msg_c = msg.clone();
    let config_c = Arc::clone(&config);
    let cmd_name_c = cmd_name.clone();

    tokio::spawn(async move {
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let ctx = crate::commands::cmd::Context {
            client: Arc::clone(&client_c),
            msg: &msg_c,
            info: &info_c,
            state: Arc::clone(&state_c),
            args: &args_ref,
            body: &body,
        };

        for interceptor in crate::commands::cmd::INTERCEPTORS {
            if let Ok(true) = interceptor.intercept(ctx.clone()).await {
                return;
            }
        }

        if is_command {
            if let Some(cmd) = crate::commands::cmd::COMMAND_MAP.get(&cmd_name_c) {
                let privileged =
                    is_privileged(info_c.source.sender.user.as_str(), &info_c, &config_c).await;

                if state_c.get_mode() == BotMode::SelfMode && !privileged {
                    return;
                }

                if cmd.category() == "root" && !privileged {
                    return;
                }

                if cmd.category() == "group" {
                    if !info_c.source.is_group {
                        return;
                    }
                    if let Ok(metadata) = client_c.groups().get_metadata(&info_c.source.chat).await
                    {
                        let is_admin = metadata
                            .participants
                            .iter()
                            .any(|p| p.jid.user == info_c.source.sender.user && p.is_admin);
                        if !is_admin {
                            return;
                        }
                    } else {
                        return;
                    }
                }

                let _ = client_c
                    .chatstate()
                    .send_composing(&info_c.source.chat)
                    .await;
                if let Err(e) = cmd.execute(ctx).await {
                    log::error!("Command error: {}", e);
                }
                let _ = client_c.chatstate().send_paused(&info_c.source.chat).await;
            }
        } else {
            if state_c.get_warmup() != WarmupMode::Off {
                let chat_jid = info_c.source.chat.clone();
                let msg_id = info_c.id.clone();
                let sender_jid = info_c.source.sender.to_string();

                let _ =
                    crate::utils::send_warmup(client_c, chat_jid, msg_id, Some(sender_jid)).await;
            }
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
