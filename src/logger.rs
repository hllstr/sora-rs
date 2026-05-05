use crate::utils::MessageExt;
use crate::utils::{extract_context, extract_type_only};
use colored::*;
use waproto::whatsapp::Message;
use whatsapp_rust::types::message::MessageInfo;

pub fn dump(info: &MessageInfo, msg: &Message) {
    let (msg_type, _) = extract_type_only(msg);
    let body = msg.text_content().cloned().unwrap_or_default();

    println!();
    println!(
        "{} [{}] {} ({})",
        chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
        msg_type.bright_cyan(),
        info.push_name.bright_green(),
        info.source
            .sender_alt
            .as_ref()
            .map(|j| j.to_string())
            .unwrap_or(info.source.sender.to_string())
            .bright_yellow()
    );

    println!("ID   : {}", info.id.white());
    println!("Chat : {}", info.source.chat.to_string().white());

    let display_body = if body.is_empty() {
        "None".dimmed()
    } else {
        body.to_string().white()
    };
    println!("Body : {}", display_body);

    if let Some(ctx) = extract_context(msg) {
        println!("\n{}", "--- Context Info ---".blue());

        if let Some(sid) = &ctx.stanza_id {
            println!("StanzaID    : {}", sid.white());
        }

        if let Some(part) = &ctx.participant {
            println!("Participant : {}", part.white());
        }

        if let Some(exp) = ctx.expiration {
            println!("Expiration  : {}s", exp.to_string().yellow());
        }

        if let Some(quoted) = &ctx.quoted_message {
            let q_body = quoted
                .text_content()
                .cloned()
                .unwrap_or_else(|| "None".to_string());
            let (q_type, _) = extract_type_only(quoted);
            println!("Quoted [{}]: {}", q_type.bright_magenta(), q_body.white());
        }
        println!("{}", "--------------------".blue());
    }
}
