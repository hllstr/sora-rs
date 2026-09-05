use crate::cmd;

cmd!(
    ListBlock,
    name: "listblock",
    aliases: ["blocklist"],
    category: "tools",
    privilege: { owner_only: true },
    execute: |ctx| {
        match ctx.client.blocking().get_blocklist().await {
            Ok(list) if list.is_empty() => {
                ctx.reply("*Tidak ada kontak yang diblokir*").await?;
            }
            Ok(list) => {
                let mut response = format!("*Daftar Blokir ({})*\n\n", list.len());
                for entry in list {
                    response.push_str(&format!("• {}\n", entry.jid));
                }
                ctx.reply(response.trim()).await?;
            }
            Err(e) => {
                crate::logger::error("listblock", e);
                ctx.react("❌").await?;
            }
        }
    }
);
