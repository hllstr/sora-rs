use crate::config::AppConfig;
use crate::handler::event_handler;
use crate::state::AppState;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use wacore::pair_code::PairCodeOptions;
use whatsapp_rust::TokioRuntime;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::pair::CompanionWebClientType;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

pub async fn create_bot(config: Arc<AppConfig>, state: Arc<AppState>) -> anyhow::Result<Bot> {
    let db_path = Path::new(&config.session_path);
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let backend = Arc::new(SqliteStore::new(&config.session_path).await?);
    let bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .with_runtime(TokioRuntime)
        .with_pair_code(PairCodeOptions {
            phone_number: config.phone_number.clone(),
            show_push_notification: true,
            custom_code: Some(config.custom_code.clone()),
            platform_id: Some(CompanionWebClientType::Chrome),
        })
        .on_event(move |event, client| {
            let st = Arc::clone(&state);
            let cfg = Arc::clone(&config);
            async move {
                event_handler(Arc::unwrap_or_clone(event), client, cfg, st).await;
            }
        })
        .build()
        .await?;

    Ok(bot)
}
