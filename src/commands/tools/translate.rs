use crate::{cmd, commands::cmd::Context};
use crate::utils::MessageExt;

cmd!(
    Translate,
    name: "translate",
    aliases: ["tr", "trans"],
    category: "tools",
    execute: |ctx| {
        translate_text(ctx).await?;
    }
);

async fn translate_text(ctx: Context<'_>) -> anyhow::Result<()> {
    if ctx.args.is_empty() {
        ctx.reply("Format: .translate <kode_bahasa> <teks>\nContoh: .translate en Selamat pagi\n\nBisa juga reply pesan lalu ketik: .translate <kode_bahasa>").await?;
        return Ok(());
    }

    let target_lang = ctx.args[0].to_lowercase();

    let quoted_text = ctx.msg.extended_text_message.as_option()
        .and_then(|ext| ext.context_info.as_option())
        .and_then(|ci| ci.quoted_message.as_option())
        .and_then(|q| q.text_content())
        .cloned();

    let text = if let Some(quoted) = quoted_text {
        quoted
    } else if ctx.args.len() > 1 {
        ctx.args[1..].join(" ")
    } else {
        ctx.reply("Tidak ada teks untuk diterjemahkan. Reply sebuah pesan atau sertakan teksnya.").await?;
        return Ok(());
    };

    if text.trim().is_empty() {
        ctx.reply("Tidak ada teks untuk diterjemahkan.").await?;
        return Ok(());
    }

    ctx.react("🕒").await?;

    let response = ctx.state.http_client
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&[
            ("client", "gtx"),
            ("sl", "auto"),
            ("tl", target_lang.as_str()),
            ("dt", "t"),
            ("q", text.as_str()),
        ])
        .send()
        .await;

    let response = match response {
        Ok(res) => res,
        Err(e) => {
            crate::logger::error("translate", format!("request failed: {}", e));
            ctx.reply("Gagal menghubungi layanan terjemahan.").await?;
            return Ok(());
        }
    };

    if !response.status().is_success() {
        ctx.reply(&format!("Layanan terjemahan mengembalikan status {}.", response.status())).await?;
        return Ok(());
    }

    let body: serde_json::Value = match response.json().await {
        Ok(json) => json,
        Err(e) => {
            crate::logger::error("translate", format!("failed to parse response: {}", e));
            ctx.reply("Gagal membaca hasil terjemahan.").await?;
            return Ok(());
        }
    };

    let translated: String = body
        .get(0)
        .and_then(|segments| segments.as_array())
        .map(|segments| {
            segments
                .iter()
                .filter_map(|segment| segment.get(0).and_then(|s| s.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    let detected_lang = body
        .get(2)
        .and_then(|lang| lang.as_str())
        .unwrap_or("?");

    if translated.trim().is_empty() {
        ctx.reply("Tidak dapat menerjemahkan teks tersebut.").await?;
        return Ok(());
    }

    ctx.reply(&format!(
        "🌐 *{}* → *{}*\n\n{}",
        detected_lang.to_uppercase(),
        target_lang.to_uppercase(),
        translated
    )).await?;

    Ok(())
}
