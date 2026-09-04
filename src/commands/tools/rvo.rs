use crate::cmd;
use whatsapp_rust::waproto::whatsapp as wa;

cmd!(
    Rvo,
    name: "rvo",
    aliases: [],
    category: "tools",
    execute: |ctx| {
        let quoted = if let Some(ext) = ctx.msg.extended_text_message.as_option()
            && let Some(ci) = ext.context_info.as_option()
            && let Some(q) = ci.quoted_message.as_option() {
            q
        } else {
            ctx.react("❔").await?;
            return Ok(());
        };

        let current_expiration = ctx.state.get_expiration(&ctx.info.source.chat.to_string());
        let apply_expiration = |context_info: &mut whatsapp_rust::buffa::MessageField<wa::ContextInfo>| {
            if current_expiration > 0 {
                context_info.get_or_insert_default().expiration = Some(current_expiration);
            }
        };
        let mut target_msg = quoted.clone();
        let mut is_vo = false;

        if let Some(img) = target_msg.image_message.as_option_mut() {
            if img.view_once.unwrap_or(false) {
                img.view_once = Some(false);
                apply_expiration(&mut img.context_info);
                is_vo = true;
            }
        }

        else if let Some(vid) = target_msg.video_message.as_option_mut()
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
