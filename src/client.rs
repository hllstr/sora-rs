use crate::config::AppConfig;
use crate::handler::event_handler;
use crate::state::AppState;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use whatsapp_rust::pair_code::PairCodeOptions;
use whatsapp_rust::pair::CompanionWebClientType;
use whatsapp_rust::prelude::*;
use whatsapp_rust::store::SqliteStore;

pub async fn create_bot(config: Arc<AppConfig>, state: Arc<AppState>) -> anyhow::Result<Bot> {
    let db_path = Path::new(&config.session_path);
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let backend = SqliteStore::new(&config.session_path).await?;
    let bot = Bot::builder()
        .with_backend(backend)
        .with_pair_code(PairCodeOptions {
            phone_number: config.phone_number.clone(),
            show_push_notification: true,
            custom_code: Some(config.custom_code.clone()),
            platform_id: Some(CompanionWebClientType::Chrome),
            ..Default::default()
        })
        .on_event(move |event, client| {
            let st = Arc::clone(&state);
            let cfg = Arc::clone(&config);
            async move {
                event_handler(event, client, cfg, st).await;
            }
        })
        .build()
        .await?;

    Ok(bot)
}
