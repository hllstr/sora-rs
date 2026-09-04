use crate::utils::MessageExt;
use crate::utils::{extract_context, extract_type_only};
use colored::*;
use whatsapp_rust::waproto::whatsapp::Message;
use whatsapp_rust::types::message::MessageInfo;

const LABEL_WIDTH: usize = 11;

pub fn info(scope: &str, msg: impl std::fmt::Display) {
    println!("  {} {} {}", "›".cyan(), format!("[{scope}]").dimmed(), msg);
}

pub fn warn(scope: &str, msg: impl std::fmt::Display) {
    println!("  {} {} {}", "!".yellow(), format!("[{scope}]").dimmed(), msg.to_string().yellow());
}

pub fn error(scope: &str, msg: impl std::fmt::Display) {
    eprintln!("  {} {} {}", "✗".red(), format!("[{scope}]").dimmed(), msg.to_string().red());
}

fn row(label: &str, value: impl std::fmt::Display) {
    println!("  {:>width$} {}", format!("{label}:").dimmed(), value, width = LABEL_WIDTH);
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.replace('\n', " ↵ ");
    }
    let clipped: String = text.chars().take(max).collect();
    format!("{} …", clipped.replace('\n', " ↵ "))
}

pub fn dump(info: &MessageInfo, msg: &Message) {
    let msg_type = extract_type_only(msg);
    let body = msg.text_content().cloned().unwrap_or_default();
    let sender = info
        .source
        .sender_alt
        .as_ref()
        .map(|j| j.to_string())
        .unwrap_or(info.source.sender.to_string());

    println!();
    println!(
        "{} {} {}",
        chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
        "MESSAGE".bold().bright_cyan(),
        format!("[{msg_type}]").cyan()
    );

    row("From", format!("{} ({})", info.push_name.bright_green(), sender.yellow()));
    row("Chat", info.source.chat.to_string());
    row("ID", info.id.as_str());

    let display_body = if body.is_empty() {
        "-".dimmed().to_string()
    } else {
        truncate(&body, 120).white().to_string()
    };
    row("Body", display_body);

    if let Some(ctx) = extract_context(msg) {
        let has_reply = ctx.quoted_message.as_option().is_some();
        let has_meta = ctx.stanza_id.is_some() || ctx.participant.is_some() || ctx.expiration.is_some();

        if has_reply || has_meta {
            println!("  {}", "reply to:".dimmed());

            if let Some(quoted) = ctx.quoted_message.as_option() {
                let q_body = quoted.text_content().cloned().unwrap_or_default();
                let q_type = extract_type_only(quoted);
                let q_display = if q_body.is_empty() {
                    "-".dimmed().to_string()
                } else {
                    truncate(&q_body, 100).white().to_string()
                };
                row("Type", q_type.magenta());
                row("Text", q_display);
            }

            if let Some(part) = &ctx.participant {
                row("Sender", part.as_str());
            }

            if let Some(exp) = ctx.expiration {
                row("Expires", format!("{exp}s").yellow());
            }
        }
    }
}
