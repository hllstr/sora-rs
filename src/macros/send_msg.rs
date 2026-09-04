#[macro_export]
macro_rules! send_msg {
    (context: $ctx:expr, dst: $dst:expr, text: $text:expr, reply: $is_reply:expr) => {
        $crate::send_msg!(
            $ctx.client,
            $ctx.msg,
            $ctx.info,
            $ctx.state,
            dst: $dst,
            text: $text,
            reply: $is_reply
        )
    };

    ($client:expr, $msg:expr, $info:expr, $state:expr, dst: $dst:expr, text: $text:expr, reply: $is_reply:expr) => {{
        let expiration = $state.get_expiration(&$dst.to_string());
        let needs_extended = $is_reply || expiration > 0;
        let message = if needs_extended {
            let mut context = whatsapp_rust::waproto::whatsapp::ContextInfo::default();
            if $is_reply {
                context = whatsapp_rust::wacore::proto_helpers::build_quote_context_with_info(
                    $info.id.clone(),
                    &$info.source.sender.to_non_ad(),
                    &$info.source.chat,
                    &$info.source.chat,
                    &$msg
                );

                context.mentioned_jid = vec![$info.source.sender.to_non_ad().to_string()];
            }

            if expiration > 0 {
                context.expiration = Some(expiration);
            }

            context.remote_jid = Some($info.source.chat.to_string());

            whatsapp_rust::waproto::whatsapp::Message {
                extended_text_message: buffa::MessageField::some(whatsapp_rust::waproto::whatsapp::message::ExtendedTextMessage {
                    text: Some($text.to_string()),
                    context_info: buffa::MessageField::some(context),
                    ..Default::default()
                }),
                ..Default::default()
            }
        } else {
            whatsapp_rust::waproto::whatsapp::Message {
                conversation: Some($text.to_string()),
                ..Default::default()
            }
        };

        $client.send_message($dst.clone(), message)
    }};
}
