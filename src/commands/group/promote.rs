use crate::cmd;
use whatsapp_rust::Jid;

cmd!(
    Promote,
    name: "promote",
    aliases: ["pm"],
    category: "group",
    privilege: { admin_only: true },
    execute: |ctx| {
        let mut targets: Vec<Jid> = Vec::new();
        if let Some(ext_msg) = ctx.msg.extended_text_message.as_option()
            && let Some(context) = ext_msg.context_info.as_option() {
                if ctx.args.is_empty() {
                    if let Some(participant) = &context.participant
                        && let Ok(jid) = participant.parse::<Jid>() {
                            targets.push(jid);
                        }
                } else {
                    for mention in &context.mentioned_jid {
                        if let Ok(jid) = mention.parse::<Jid>() {
                            targets.push(jid);
                        }
                    }
                }
            }
        if targets.is_empty() {
            ctx.react("❔").await?;
            return Ok(());
        }
        crate::logger::info("promote", format!("targets: {:?}", targets));
        match ctx.client.groups().promote_participants(&ctx.info.source.chat, &targets).await {
            Ok(_) => {
                ctx.react("✅").await?;
            }
            Err(e) => {
                crate::logger::error("promote", e);
                ctx.react("❌").await?;
            }
        }
    }
);