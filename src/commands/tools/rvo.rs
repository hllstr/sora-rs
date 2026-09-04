use crate::cmd;
use whatsapp_rust::waproto::whatsapp as wa;

cmd!(
    Rvo,
    name: "rvo",
    aliases: [],
    category: "tools",
    execute: |ctx| {
        let quoted = if let Some(ext) = &ctx.msg.extended_text_message
            && let Some(ci) = &ext.context_info
            && let Some(q) = &ci.quoted_message {
            q
        } else {
            ctx.react("❔").await?;
            return Ok(());
        };

        let current_expiration = ctx.state.get_expiration(&ctx.info.source.chat.to_string());
        let apply_expiration = |context_info: &mut Option<Box<wa::ContextInfo>>| {
            if current_expiration > 0 {
                let ci = context_info.get_or_insert_with(|| Box::new(wa::ContextInfo::default()));
                ci.expiration = Some(current_expiration);
            }
        };
        let mut target_msg = *quoted.clone();
        let mut is_vo = false;

        if let Some(img) = &mut target_msg.image_message {
            if img.view_once.unwrap_or(false) {
                img.view_once = Some(false);
                apply_expiration(&mut img.context_info);
                is_vo = true;
            }
        }

        else if let Some(vid) = &mut target_msg.video_message
            && vid.view_once.unwrap_or(false) {
                vid.view_once = Some(false);
                apply_expiration(&mut vid.context_info);
                is_vo = true;
            }

        if is_vo {
            ctx.client.send_message(ctx.info.source.chat.clone(), target_msg).await?;
        } else {
            ctx.react("❔").await?;
        }
    }
);
