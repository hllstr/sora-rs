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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PairingMethod {
    Qr,
    Code,
}

impl From<&str> for AutoreadMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "all" => AutoreadMode::All,
            "group" | "groups" => AutoreadMode::Group,
            "dm" | "chat" | "chats" | "private" => AutoreadMode::Dm,
            _ => AutoreadMode::Off,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AutoreadMode {
    #[default]
    Off,
    All,
    Group,
    Dm,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub prefixes: Vec<String>,
    pub session_path: String,
    pub custom_code: String,
    pub mode: BotMode,
    pub warmup: WarmupMode,
    #[serde(default)]
    pub autoread: AutoreadMode,
    pub pairing: PairingMethod,
    #[serde(skip)]
    pub phone_number: String,
    #[serde(skip)]
    pub superuser: Vec<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let phone = match std::env::var("PHONE_NUMBER") {
            Ok(phone) => phone,
            Err(_) => {
                crate::logger::warn("config", "PHONE_NUMBER is not set in .env");
                std::process::exit(1);
            }
        };
        let su = std::env::var("SUPERUSER").ok();
        let toml_str = fs::read_to_string("Config.toml")?;
        let mut config: AppConfig = toml::from_str(&toml_str)?;
        config.superuser = if let Some(su_str) = su {
            su_str.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec![]
        };
        config.phone_number = phone;
        Ok(config)
    }
}
