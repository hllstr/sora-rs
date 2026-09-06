use crate::cmd;
use whatsapp_rust::RevokeType;

cmd!(
    Delete,
    name: "delete",
    aliases: ["del", "hapus"],
    category: "group",
    access: { admin_only: true },
    execute: |ctx| {
        let quoted_id = if let Some(ext) = ctx.msg.extended_text_message.as_option()
            && let Some(ci) = ext.context_info.as_option()
            && let Some(id) = &ci.stanza_id {
            id.clone()
        } else {
            ctx.react("❔").await?;
            return Ok(());
        };

        let original_sender = if let Some(ext) = ctx.msg.extended_text_message.as_option()
            && let Some(ci) = ext.context_info.as_option()
            && let Some(participant) = &ci.participant
            && let Ok(jid) = participant.parse() {
            jid
        } else {
            ctx.info.source.sender.clone()
        };

        match ctx.client.revoke_message(
            ctx.info.source.chat.clone(),
            quoted_id,
            RevokeType::Admin { original_sender },
        ).await {
            Ok(_) => {
                ctx.react("🗑️").await?;
            }
            Err(e) => {
                crate::logger::error("delete", e);
                ctx.react("❌").await?;
            }
        }
    }
);
