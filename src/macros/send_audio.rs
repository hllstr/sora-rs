#[macro_export]
macro_rules! send_audio {
    (
        context: $ctx:expr,
        audio_data: $data:expr,
        dst: $dst:expr,
        reply: $is_reply:expr
        $(, config_context: $config_fn:expr)?
    ) => {{
        async  {
            use wacore::proto_helpers::build_quote_context_with_info;
            use waproto::whatsapp::{Message, message::AudioMessage, ContextInfo};
            use wacore::download::MediaType;
            use whatsapp_rust::UploadOptions;

            let client = &$ctx.client;
            let info = &$ctx.info;
            let state = &$ctx.state;

            let raw_data: Vec<u8> = $data.into();
            let audio_bytes = $crate::utils::get_media_bytes(std::sync::Arc::clone(state), raw_data).await?;
            let upload = client.upload(audio_bytes, MediaType::Audio, UploadOptions::default()).await?;

            let mut context_info = if $is_reply {
                let mut ctx_info = build_quote_context_with_info(
                    &info.id,
                    &info.source.sender,
                    &info.source.chat,
                    $ctx.msg,
                );
                ctx_info.mentioned_jid = vec![info.source.sender.to_non_ad().to_string()];
                ctx_info
            } else {
                ContextInfo::default()
            };

            let expiration = state.get_expiration(&$dst.to_string());
            if expiration > 0 {
                context_info.expiration = Some(expiration);
            }

            $(
                ($config_fn)(&mut context_info);
            )?

            context_info.remote_jid = Some($ctx.info.source.chat.to_string());

            let audio_msg = Message {
                audio_message: Some(Box::new(AudioMessage {
                    url: Some(upload.url),
                    direct_path: Some(upload.direct_path),
                    media_key: Some(upload.media_key.to_vec()),
                    file_sha256: Some(upload.file_sha256.to_vec()),
                    file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
                    file_length: Some(upload.file_length),
                    mimetype: Some("audio/mpeg".to_string()),
                    context_info: Some(Box::new(context_info)),
                    ..Default::default()
                })),
                ..Default::default()
            };

            client.send_message($dst.clone(), audio_msg).await
        }
    }};
}
