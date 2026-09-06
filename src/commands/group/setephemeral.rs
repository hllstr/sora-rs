use crate::cmd;

cmd!(
    SetEphemeral,
    name: "setephemeral",
    aliases: ["ephemeral", "pesansementara"],
    category: "group",
    access: { admin_only: true },
    execute: |ctx| {
        let seconds: u32 = match ctx.args.first().copied() {
            Some("off") | Some("0") => 0,
            Some("24h") => 86400,
            Some("7d") => 604800,
            Some("90d") => 7776000,
            Some(other) => match other.parse() {
                Ok(v) => v,
                Err(_) => {
                    ctx.react("❔").await?;
                    return Ok(());
                }
            },
            None => {
                ctx.react("❔").await?;
                return Ok(());
            }
        };

        match ctx.client.groups().set_ephemeral(&ctx.info.source.chat, seconds).await {
            Ok(_) => {
                ctx.state.set_expiration(ctx.info.source.chat.to_string(), seconds);
                ctx.react("✅").await?;
            }
            Err(e) => {
                crate::logger::error("setephemeral", e);
                ctx.react("❌").await?;
            }
        }
    }
);
