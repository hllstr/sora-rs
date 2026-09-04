use crate::cmd;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: bool,
    result: ApiResult,
}

#[derive(Debug, Deserialize)]
struct ApiResult {
    contents: Vec<Content>,
    metadata: Metadata,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Content {
    url: String,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    username: String,
    title: String,
}

cmd!(
    Reels,
    name: "reels",
    aliases: ["ig", "instagram"],
    category: "downloader",
    execute: |ctx| {
        if ctx.body.is_empty() {
            ctx.react("❔").await?;
            return Ok(());
        }
        let api = "https://api.apigratis.cc/downloader/instagram?url=".to_string();
        let target = ctx.body;
        let response = ctx.state.http_client.get(api + target).send().await?;
        let data: ApiResponse = response.json().await?;
        if !data.status {
            ctx.reply("Not found").await?;
            return Ok(());
        }

        let text = format!("*Author:* {}\n{}", data.result.metadata.username, data.result.metadata.title);
        send_video!(
            context: ctx,
            video_data: data.result.contents[0].url.clone(),
            dst: ctx.info.source.chat,
            caption: text,
            reply: true
        )
        .await?;

        println!("Done!");
    }
);
