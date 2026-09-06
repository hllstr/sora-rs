use crate::{cmd, commands::cmd::Context};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const EXTERNAL_PROCESS_TIMEOUT: Duration = Duration::from_secs(120);

const MAX_DOWNLOADS_CACHE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn cache_key(input: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn evict_downloads_cache_if_needed() {
    let dir = Path::new("downloads");
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    let mut entries: Vec<(std::path::PathBuf, u64, std::time::SystemTime)> = Vec::new();
    let mut total: u64 = 0;

    for entry in read_dir.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let size = meta.len();
        let accessed = meta
            .accessed()
            .or_else(|_| meta.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        total += size;
        entries.push((entry.path(), size, accessed));
    }

    if total <= MAX_DOWNLOADS_CACHE_BYTES {
        return;
    }

    entries.sort_unstable_by_key(|(_, _, accessed)| *accessed);

    let mut to_free = total - MAX_DOWNLOADS_CACHE_BYTES;
    for (path, size, _) in entries {
        if to_free == 0 {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            to_free = to_free.saturating_sub(size);
        }
    }
}

cmd!(
    Play,
    name: "play",
    aliases: ["ytmp3", "song"],
    category: "downloader",
    execute: |ctx| {
        play_audio(ctx).await?;
    }
);

async fn play_audio(ctx: Context<'_>) -> anyhow::Result<()> {
    let _ = fs::create_dir_all("cookies");

    let _ = fs::create_dir_all("downloads");
    let cookie_path = "cookies/www.youtube.com_cookies.txt";
    let input = if ctx.args.is_empty() {
        ctx.reply("Input title or url.").await?;
        return Ok(());
    } else {
        ctx.args.join(" ")
    };

    ctx.react("🕒").await?;
    let raw_metadata: String;
    let metadata_path = format!("downloads/{}.txt", cache_key(&input));
    if Path::new(&metadata_path).exists() {
        crate::logger::info("play", "metadata cache hit, skipping fetch");
        raw_metadata = tokio::fs::read_to_string(&metadata_path).await?;
    } else {
        crate::logger::info("play", "metadata cache miss, fetching...");
        let mut metadata_cmd = Command::new("yt-dlp");
        metadata_cmd.env_remove("NODE_CHANNEL_FD").args([
            "--print",
            "%(id)s|%(title)s|%(uploader)s|%(thumbnail)s",
            "--no-playlist",
            &format!("ytsearch:{}", input),
            "--cookies",
            cookie_path,
        ]);
        let metadata_output = timeout(EXTERNAL_PROCESS_TIMEOUT, metadata_cmd.output()).await??;

        raw_metadata = String::from_utf8_lossy(&metadata_output.stdout)
            .trim()
            .to_string();

        tokio::fs::write(&metadata_path, &raw_metadata).await?;
    }

    let parts: Vec<&str> = raw_metadata.split('|').collect();
    if parts.len() < 4 {
        ctx.reply("Video not found. perhaps something went wrong?")
            .await?;
        return Ok(());
    }
    let video_id = parts[0];
    let title = parts[1];
    let channel = parts[2];
    let thumbnail_url = parts[3];

    if video_id.is_empty()
        || !video_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        ctx.reply("Video not found. perhaps something went wrong?")
            .await?;
        return Ok(());
    }

    let file_path = format!("downloads/{}.mp3", video_id);

    if Path::new(&file_path).exists() {
        crate::logger::info("play", "audio cache hit, skipping download");
        ctx.react("✅").await?;
        send_audio!(
            context: ctx,
            audio_data: file_path,
            dst: ctx.info.source.chat,
            reply: true,
            config_context: |context_info: &mut whatsapp_rust::waproto::whatsapp::ContextInfo| {
                context_info.external_ad_reply = whatsapp_rust::buffa::MessageField::some(whatsapp_rust::waproto::whatsapp::context_info::ExternalAdReplyInfo {
                    title: Some(title.to_string()),
                    body: Some(channel.to_string()),
                    media_type: Some(whatsapp_rust::waproto::whatsapp::context_info::external_ad_reply_info::MediaType::Image),
                    thumbnail_url: Some(thumbnail_url.to_string()),
                    render_larger_thumbnail: Some(true),
                    ..Default::default()
                });
            }
        )
        .await?;
        return Ok(());
    }

    ctx.react("👀").await?;
    let download_process = Command::new("yt-dlp")
        .env_remove("NODE_CHANNEL_FD")
        .args([
            "-x",
            "--audio-format",
            "mp3",
            "--no-playlist",
            "-o",
            "downloads/%(id)s.%(ext)s",
            "--cookies",
            cookie_path,
            &format!("https://www.youtube.com/watch?v={}", video_id),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = match timeout(
        EXTERNAL_PROCESS_TIMEOUT,
        download_process.wait_with_output(),
    )
    .await
    {
        Ok(res) => res?,
        Err(_) => {
            ctx.reply("Download timed out.").await?;
            return Ok(());
        }
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        ctx.reply(&format!("Failed to download audio: {}", err))
            .await?;
        return Ok(());
    }

    ctx.react("✅").await?;
    send_audio!(
        context: ctx,
        audio_data: file_path,
        dst: ctx.info.source.chat,
        reply: true,
        config_context: |context_info: &mut whatsapp_rust::waproto::whatsapp::ContextInfo| {
            context_info.external_ad_reply = whatsapp_rust::buffa::MessageField::some(whatsapp_rust::waproto::whatsapp::context_info::ExternalAdReplyInfo {
                title: Some(title.to_string()),
                body: Some(channel.to_string()),
                media_type: Some(whatsapp_rust::waproto::whatsapp::context_info::external_ad_reply_info::MediaType::Image),
                thumbnail_url: Some(thumbnail_url.to_string()),
                render_larger_thumbnail: Some(true),
                ..Default::default()
            });
        }
    )
    .await?;

    tokio::task::spawn_blocking(evict_downloads_cache_if_needed);

    Ok(())
}
