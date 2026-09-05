use crate::{cmd, commands::cmd::Context};
use std::time::Instant;
use tokio::net::TcpStream;
use whatsapp_rust::waproto::whatsapp as wa;

cmd!(
    Ping,
    name: "ping",
    aliases: ["p"],
    category: "general",
    execute: |ctx| {
        ping(ctx).await?;
    }
);

async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    let server_wangsaf = "g.whatsapp.net:443";
    let count = ctx
        .args
        .first()
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(1);

    let net_start = Instant::now();
    TcpStream::connect(server_wangsaf).await.ok();
    let latency = net_start.elapsed();

    let msg = ctx
        .reply("```Pong!\n----------------------\nMeasuring...```")
        .await?;
    let mut lines = Vec::new();
    let mut last_rtt = None;

    for _ in 0..count {
        let edit_start = Instant::now();

        let line = match last_rtt {
            Some(rtt) => format!("Response   Time: {:.2}ms", rtt),
            None => format!("Response   Time: {:.2}ms", net_start.elapsed().as_millis()),
        };

        lines.push(line);

        let updated = wa::Message {
            conversation: Some(format!(
                "```Pong!\n----------------------\nNetwork Latency: {}ms\n{}```",
                latency.as_millis(),
                lines.join("\n")
            )),
            ..Default::default()
        };

        ctx.client
            .edit_message(ctx.info.source.chat.clone(), msg.clone(), updated)
            .await?;

        let rtt = edit_start.elapsed().as_secs_f32() * 1000.0;
        last_rtt = Some(rtt);
    }

    Ok(())
}
