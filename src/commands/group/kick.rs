use crate::cmd;
use rand::seq::IndexedRandom;
use whatsapp_rust::Jid;

cmd!(
    Kick,
    name: "kick",
    aliases: ["dor"],
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

        crate::logger::info("kick", format!("targets: {:?}", targets));

        if ctx.body == "random" {
            let info = ctx.client.groups().query_info(&ctx.info.source.chat).await?;
            let participants = &info.participants;
            if let Some(random_jid) = participants.choose(&mut rand::rng()) {
                targets.push(random_jid.clone());
            }
        }
        if targets.is_empty() {
            ctx.react("❔").await?;
            return Ok(());
        }
        match ctx.client.groups().remove_participants(&ctx.info.source.chat, &targets).await {
            Ok(_) => {
                ctx.react("💥").await?;
            }
            Err(e) => {
                crate::logger::error("kick", e);
                ctx.react("❌").await?;
            }
        }
    }
);
