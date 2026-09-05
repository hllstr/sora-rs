use whatsapp_rust::RevokeType;

use crate::cmd;

cmd!(
    Mute,
    name: "mute",
    aliases: ["bungkam"],
    category: "group",
    privilege: { admin_only: true },
    intercept: |ctx| {
        let chat_jid = ctx.info.source.chat.to_string();
        let sender_jid = ctx.info.source.sender.to_non_ad().to_string();
        let key = format!("mute:{}:{}", chat_jid, sender_jid);

        if ctx.state.has_cache(&key) {
            let original_sender = ctx.info.source.sender.clone();
            ctx.client.revoke_message(
                ctx.info.source.chat.clone(),
                ctx.info.id.clone(),
                RevokeType::Admin { original_sender },
            ).await?;
            return Ok(true);
        }
        Ok(false)
    },
    execute: |ctx| {
        let target_jid = if let Some(ext_msg) = ctx.msg.extended_text_message.as_option()
        && let Some(context) = ext_msg.context_info.as_option() {
            if let Some(participant) = &context.participant {
                participant.clone()
            } else if let Some(mention) = context.mentioned_jid.first() {
                mention.clone()
            } else {
                ctx.react("❔").await?;
                return Ok(());
            }
        } else {
            ctx.react("❔").await?;
            return Ok(());
        };
        let key = format!("mute:{}:{}", ctx.info.source.chat, target_jid);
        if ctx.state.has_cache(&key) {
            ctx.state.del_cache(&key);
            ctx.react("🔊").await?;
        } else {
            ctx.state.set_cache(&key, "1");
            ctx.react("🤫").await?;
        }
    }
);
