#[macro_export]
macro_rules! send_video {
    (
        context: $ctx:expr,
        video_data: $data:expr,
        dst: $dst:expr,
        caption: $caption:expr,
        reply: $is_reply:expr
        $(, config_context: $config_fn:expr)?
    ) => {
        async {
            use wacore::proto_helpers::build_quote_context_with_info;
            use waproto::whatsapp::{Message, message::VideoMessage, ContextInfo};
            use wacore::download::MediaType;
            use std::sync::Arc;
            use whatsapp_rust::UploadOptions;

            let client = &$ctx.client;
            let info = &$ctx.info;
            let state = &$ctx.state;
            let raw_data: Vec<u8> = $data.into();
            let video_bytes = $crate::utils::get_media_bytes(Arc::clone(&state), raw_data).await?;
            let thumbnail_bytes = $crate::utils::generate_video_thumbnail(&video_bytes).await.ok();

            let upload = client.upload(video_bytes, MediaType::Video, UploadOptions::default()).await?;

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

            context_info.remote_jid = Some(info.source.chat.to_string());

            let video_msg = Message {
                video_message: Some(Box::new(VideoMessage {
                    url: Some(upload.url),
                    direct_path: Some(upload.direct_path),
                    media_key: Some(upload.media_key.to_vec()),
                    file_sha256: Some(upload.file_sha256.to_vec()),
                    file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
                    file_length: Some(upload.file_length),
                    mimetype: Some("video/mp4".to_string()),
                    caption: Some($caption.to_string()),
                    jpeg_thumbnail: thumbnail_bytes,
                    context_info: Some(Box::new(context_info)),
                    ..Default::default()
                })),
                ..Default::default()
            };

            client.send_message($dst.clone(), video_msg).await
        }
    };
}
