use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::config::{AppConfig, BotMode, WarmupMode};

pub enum ConfigKey {
    Mode,
    Prefixes,
    Warmup,
    WarmupInterval,
}

pub enum ConfigValue {
    Text(String),
    List(Vec<String>),
    Number(u64),
}

pub struct AppState {
    pub http_client: reqwest::Client,
    pub cache: DashMap<String, String>,
    pub start_time: Instant,
    pub config: Arc<AppConfig>,
    pub mode: RwLock<BotMode>,
    pub prefixes: RwLock<Arc<Vec<String>>>,
    pub warmup: RwLock<WarmupMode>,
    pub warmup_interval: RwLock<u64>,
}

impl AppState {
    pub fn load(config: Arc<AppConfig>) -> Arc<Self> {
        let start_time = Instant::now();
        let cache = DashMap::new();
        let http_client = reqwest::Client::new();

        Arc::new(Self {
            http_client,
            cache,
            start_time,
            prefixes: RwLock::new(Arc::new(config.prefixes.clone())),
            mode: RwLock::new(config.mode),
            warmup: RwLock::new(config.warmup),
            warmup_interval: RwLock::new(config.warmup_interval),
            config,
        })
    }

    pub fn set_expiration(&self, jid: String, seconds: u32) {
        self.cache
            .insert(format!("exp:{}", jid), seconds.to_string());
    }

    pub fn get_expiration(&self, jid: &str) -> u32 {
        self.cache
            .get(&format!("exp:{}", jid))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    pub fn get_mode(&self) -> BotMode {
        *self.mode.read().unwrap()
    }

    pub fn get_prefixes(&self) -> Arc<Vec<String>> {
        self.prefixes.read().unwrap().clone()
    }

    pub fn get_warmup(&self) -> WarmupMode {
        *self.warmup.read().unwrap()
    }

    pub fn get_warmup_interval(&self) -> u64 {
        *self.warmup_interval.read().unwrap()
    }

    pub fn set_cache(&self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    // not used yet, but someday I'll be uncommenting this
    // pub fn get_cache(&self, key: &str) -> Option<String> {
    //     self.cache.get(key).map(|v| v.value().clone())
    // }

    pub fn has_cache(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn del_cache(&self, key: &str) {
        self.cache.remove(key);
    }

    pub fn set_config(&self, key: ConfigKey, value: ConfigValue) -> Result<(), &'static str> {
        match (key, value) {
            (ConfigKey::Mode, ConfigValue::Text(val)) => {
                let mut mode = self.mode.write().unwrap();
                *mode = if val.to_lowercase() == "self" {
                    BotMode::SelfMode
                } else {
                    BotMode::Public
                };
                Ok(())
            }
            (ConfigKey::Prefixes, ConfigValue::List(val)) => {
                let mut prefixes = self.prefixes.write().unwrap();
                *prefixes = val.into();
                Ok(())
            }
            (ConfigKey::Warmup, ConfigValue::Text(val)) => {
                let mut warmup = self.warmup.write().unwrap();
                *warmup = WarmupMode::from(val.as_str());
                Ok(())
            }
            (ConfigKey::WarmupInterval, ConfigValue::Number(val)) => {
                let mut interval = self.warmup_interval.write().unwrap();
                *interval = val;
                Ok(())
            }
            _ => Err("invalid datatype for this field"),
        }
    }
}
