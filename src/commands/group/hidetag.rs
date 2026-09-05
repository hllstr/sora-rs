use crate::cmd;
use whatsapp_rust::waproto::whatsapp as wa;

cmd!(
    HideTag,
    name: "hidetag",
    aliases: ["ht", "tagall"],
    category: "group",
    privilege: { admin_only: true },
    execute: |ctx| {
        let info = ctx.client.groups().query_info(&ctx.info.source.chat).await?;
        let mentions: Vec<String> = info.participants.iter().map(|jid| jid.to_string()).collect();

        let text = if ctx.body.is_empty() {
            "📢".to_string()
        } else {
            ctx.body.to_string()
        };

        let expiration = ctx.state.get_expiration(&ctx.info.source.chat.to_string());
        let mut context = wa::ContextInfo {
            mentioned_jid: mentions,
            ..Default::default()
        };
        if expiration > 0 {
            context.expiration = Some(expiration);
        }

        let message = wa::Message {
            extended_text_message: whatsapp_rust::buffa::MessageField::some(wa::message::ExtendedTextMessage {
                text: Some(text),
                context_info: whatsapp_rust::buffa::MessageField::some(context),
                ..Default::default()
            }),
            ..Default::default()
        };

        ctx.client.send_message(ctx.info.source.chat.clone(), message).await?;
    }
);
