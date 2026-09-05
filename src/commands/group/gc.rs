use std::sync::Arc;

use whatsapp_rust::{Client, Jid};

use crate::cmd;
cmd!(
    Group,
    name: "group",
    aliases: ["gc"],
    category: "group",
    privilege: { admin_only: true },
    execute: |ctx| {
        let Some(subcommand) = ctx.args.first().copied() else {
            ctx.react("❔").await?;
            return Ok(());
        };
        let group_jid = &ctx.info.source.chat;
        let client = ctx.client.clone();
        match subcommand {
            "open" => lock(group_jid, client, false).await,
            "close" => lock(group_jid, client, true).await,
            "link" => {
                ctx.reply(get_link(group_jid, client).await.to_string().as_str()).await?;
            }
            _ => {
                ctx.react("❔").await?;
            }
        }
    }
);

async fn get_link(group_jid: &Jid, client: Arc<Client>) -> String {
    let link = client.groups().get_invite_link(group_jid, false).await;
    match link {
        Ok(link) => link,
        Err(e) => e.to_string(),
    }
}

async fn lock(group_jid: &Jid, client: Arc<Client>, lock: bool) {
    let _ = client.groups().set_announce(group_jid, lock).await;
}