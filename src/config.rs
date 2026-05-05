use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BotMode {
    #[serde(rename = "self")]
    SelfMode,

    #[serde(rename = "public")]
    Public,
}

impl From<&str> for WarmupMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "normal" => WarmupMode::Normal,
            _ => WarmupMode::Off,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WarmupMode {
    Normal,
    Off,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub prefixes: Vec<String>,
    pub session_path: String,
    pub custom_code: String,
    pub mode: BotMode,
    pub warmup: WarmupMode,
    pub warmup_interval: u64,
    #[serde(skip)]
    pub phone_number: String,
    #[serde(skip_serializing)]
    pub superuser: Option<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let phone = std::env::var("PHONE_NUMBER").expect("PHONE_NUMBER must be set in .env");
        let su = std::env::var("SUPERUSER").ok();
        let toml_str = fs::read_to_string("Config.toml")?;
        let mut config: AppConfig = toml::from_str(&toml_str)?;
        config.superuser = su;
        config.phone_number = phone;
        Ok(config)
    }
}
