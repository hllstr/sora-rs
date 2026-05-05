use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use waproto::whatsapp as wa;
use waproto::whatsapp::Message;
use whatsapp_rust::Jid;
use whatsapp_rust::client::Client;

use crate::state::AppState;

pub trait MessageExt {
    fn get_expiration_timer(&self) -> Option<u32>;
    fn text_content(&self) -> Option<&String>;
}

impl MessageExt for Message {
    fn get_expiration_timer(&self) -> Option<u32> {
        if let Some(proto) = &self.protocol_message
            && let Some(exp) = proto.ephemeral_expiration
        {
            return Some(exp);
        }

        if self.conversation.is_some() {
            return Some(0);
        }

        let context = if let Some(m) = &self.extended_text_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.image_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.video_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.document_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.sticker_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.audio_message {
            m.context_info.as_deref()
        } else {
            None
        };
        context.map(|ctx| ctx.expiration.unwrap_or(0))
    }

    fn text_content(&self) -> Option<&String> {
        if let Some(t) = &self.conversation {
            return Some(t);
        }
        if let Some(m) = &self.extended_text_message {
            return m.text.as_ref();
        }
        if let Some(m) = &self.image_message {
            return m.caption.as_ref();
        }
        if let Some(m) = &self.video_message {
            return m.caption.as_ref();
        }
        if let Some(m) = &self.document_message {
            return m.caption.as_ref();
        }
        if let Some(m) = &self.document_with_caption_message {
            return m
                .message
                .as_ref()
                .and_then(|nested| nested.document_message.as_ref())
                .and_then(|doc| doc.caption.as_ref());
        }
        None
    }
}

pub fn extract_type_only(msg: &Message) -> (String, String) {
    let debug_str = format!("{:?}", msg);
    let raw_type = debug_str
        .split(": Some(")
        .next()
        .and_then(|s| s.split_whitespace().last())
        .unwrap_or("unknown")
        .to_string();
    (raw_type, String::new())
}

pub fn extract_context(msg: &Message) -> Option<&waproto::whatsapp::ContextInfo> {
    if let Some(ctx) = &msg
        .extended_text_message
        .as_ref()
        .and_then(|m| m.context_info.as_ref())
    {
        return Some(ctx);
    }
    if let Some(ctx) = &msg
        .image_message
        .as_ref()
        .and_then(|m| m.context_info.as_ref())
    {
        return Some(ctx);
    }
    if let Some(ctx) = &msg
        .video_message
        .as_ref()
        .and_then(|m| m.context_info.as_ref())
    {
        return Some(ctx);
    }
    if let Some(ctx) = &msg
        .sticker_message
        .as_ref()
        .and_then(|m| m.context_info.as_ref())
    {
        return Some(ctx);
    }
    None
}

pub async fn get_media_bytes(state: Arc<AppState>, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    if let Ok(str_val) = std::str::from_utf8(&data) {
        let trimmed_val = str_val.trim();
        if trimmed_val.starts_with("http") {
            let resp = state.http_client.get(trimmed_val).send().await?;
            return Ok(resp.bytes().await?.to_vec());
        }
        if tokio::fs::metadata(trimmed_val).await.is_ok() {
            return Ok(tokio::fs::read(trimmed_val).await?);
        }
    }
    Ok(data)
}

pub async fn generate_video_thumbnail(video_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let data = Arc::new(video_bytes.to_vec());

    let mut child = Command::new("ffmpeg")
        .args([
            "-ss",
            "00:00:00",
            "-i",
            "pipe:0",
            "-vframes",
            "1",
            "-f",
            "image2",
            "-vf",
            "scale=160:-1",
            "-q:v",
            "2",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;

    let writer_task = tokio::spawn(async move {
        let _ = stdin.write_all(&data).await;
        let _ = stdin.flush().await;
        drop(stdin);
    });

    let output = child.wait_with_output().await?;
    let _ = writer_task.await;

    if output.status.success() && !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        Err(anyhow::anyhow!("ffmpeg: failed to generate thumbnail"))
    }
}

pub async fn send_warmup(
    client: Arc<Client>,
    chat_jid: Jid,
    msg_id: String,
    participant: Option<String>,
) -> anyhow::Result<()> {
    println!("Sending warmup message");
    let warmup_msg = wa::Message {
        reaction_message: Some(wa::message::ReactionMessage {
            key: Some(wa::MessageKey {
                remote_jid: Some(chat_jid.to_string()),
                from_me: Some(false),
                id: Some(msg_id),
                participant,
            }),
            text: Some("".to_string()),
            sender_timestamp_ms: Some(chrono::Utc::now().timestamp_millis()),
            ..Default::default()
        }),
        ..Default::default()
    };

    client.send_message(chat_jid, warmup_msg).await?;
    Ok(())
}
