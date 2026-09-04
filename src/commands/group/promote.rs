use crate::cmd;
use whatsapp_rust::Jid;

cmd!(
    Promote,
    name: "promote",
    aliases: ["pm"],
    category: "group",
    execute: |ctx| {
        let mut targets: Vec<Jid> = Vec::new();
        if let Some(ext_msg) = &ctx.msg.extended_text_message
            && let Some(context) = &ext_msg.context_info {
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
        println!("{:?}", targets);
        match ctx.client.groups().promote_participants(&ctx.info.source.chat, &targets).await {
            Ok(_) => {
                ctx.react("✅").await?;
            }
            Err(e) => {
                eprintln!("err: {}", e);
                ctx.react("❌").await?;
            }
        }
    }
);