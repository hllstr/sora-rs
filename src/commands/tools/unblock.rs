use crate::cmd;
use whatsapp_rust::Jid;

cmd!(
    Unblock,
    name: "unblock",
    aliases: ["bukablokir"],
    category: "tools",
    privilege: { owner_only: true },
    execute: |ctx| {
        let target: Jid = if let Some(arg) = ctx.args.first() {
            let normalized = arg.trim_start_matches('+').replace([' ', '-'], "");
            match format!("{}@s.whatsapp.net", normalized).parse() {
                Ok(jid) => jid,
                Err(_) => {
                    ctx.react("❔").await?;
                    return Ok(());
                }
            }
        } else if let Some(ext) = ctx.msg.extended_text_message.as_option()
            && let Some(ci) = ext.context_info.as_option()
            && let Some(participant) = &ci.participant
            && let Ok(jid) = participant.parse() {
            jid
        } else {
            ctx.react("❔").await?;
            return Ok(());
        };

        match ctx.client.blocking().unblock(&target).await {
            Ok(_) => {
                ctx.reply(&format!("✅ Berhasil membuka blokir {}", target)).await?;
            }
            Err(e) => {
                crate::logger::error("unblock", e);
                ctx.react("❌").await?;
            }
        }
    }
);
