use crate::{cmd, commands::cmd::Context};
use std::time::Instant;
use whatsapp_rust::waproto::whatsapp as wa;

cmd!(
    SpamEdit,
    name: "spamedit",
    aliases: ["se", "spedit"],
    category: "root",
    privilege: { owner_only: true },
    execute: |ctx| {
        spam_edit(ctx).await?;
    }
);

async fn spam_edit(ctx: Context<'_>) -> anyhow::Result<()> {
    let count = ctx
        .args
        .first()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(5)
        .min(1000);

    let start = Instant::now();

    let initial_text = "*Starting spamedit 1x*";
    let msg_id = ctx.reply(initial_text).await?;

    for i in 1..=count {
        let current_text = format!("```Starting spamedit {}x```", i,);

        let message = wa::Message {
            conversation: Some(current_text),
            ..Default::default()
        };

        ctx.client
            .edit_message(ctx.info.source.chat.clone(), msg_id.clone(), message)
            .await?;
    }

    let total_elapsed = start.elapsed();
    let final_report = format!(
        "*Done! spamedit {}x*\n```Elapsed time: {:.2}s\nAvg Speed: {:.2}ms/edit```",
        count,
        total_elapsed.as_secs_f32(),
        total_elapsed.as_millis() as f32 / count as f32
    );

    let final_msg = wa::Message {
        conversation: Some(final_report),
        ..Default::default()
    };

    ctx.client
        .edit_message(ctx.info.source.chat.clone(), msg_id, final_msg)
        .await?;

    Ok(())
}
