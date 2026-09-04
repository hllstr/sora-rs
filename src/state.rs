use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::config::{AppConfig, BotMode, WarmupMode};

const MAX_TRACKED_ENTRIES: usize = 20_000;

struct TrackedEntry<T> {
    value: T,
    last_touch: u64,
}

pub enum ConfigKey {
    Mode,
    Prefixes,
    Warmup,
}

pub enum ConfigValue {
    Text(String),
    List(Vec<String>),
}

pub struct AppState {
    pub http_client: reqwest::Client,
    pub cache: DashMap<String, String>,
    expirations: DashMap<String, TrackedEntry<u32>>,
    touch_counter: AtomicU64,
    pub start_time: Instant,
    pub config: Arc<AppConfig>,
    pub mode: RwLock<BotMode>,
    pub prefixes: RwLock<Arc<Vec<String>>>,
    pub warmup: RwLock<WarmupMode>,
}

impl AppState {
    pub fn load(config: Arc<AppConfig>) -> Arc<Self> {
        let start_time = Instant::now();
        let cache = DashMap::new();
        let expirations = DashMap::new();
        let http_client = reqwest::Client::new();

        let state = Arc::new(Self {
            http_client,
            cache,
            expirations,
            touch_counter: AtomicU64::new(0),
            start_time,
            prefixes: RwLock::new(Arc::new(config.prefixes.clone())),
            mode: RwLock::new(config.mode),
            warmup: RwLock::new(config.warmup),
            config,
        });

        let sweep_state = Arc::clone(&state);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600));
            loop {
                interval.tick().await;
                sweep_state.evict_stale_expirations();
            }
        });

        state
    }

    fn evict_stale_expirations(&self) {
        let len = self.expirations.len();
        if len <= MAX_TRACKED_ENTRIES {
            return;
        }

        let overflow = len - MAX_TRACKED_ENTRIES;
        let mut touches: Vec<(String, u64)> = self
            .expirations
            .iter()
            .map(|e| (e.key().clone(), e.value().last_touch))
            .collect();
        touches.sort_unstable_by_key(|(_, t)| *t);

        for (key, _) in touches.into_iter().take(overflow) {
            self.expirations.remove(&key);
        }
    }

    pub fn set_expiration(&self, jid: String, seconds: u32) {
        let tick = self.touch_counter.fetch_add(1, Ordering::Relaxed);
        self.expirations.insert(
            jid,
            TrackedEntry {
                value: seconds,
                last_touch: tick,
            },
        );
    }

    pub fn get_expiration(&self, jid: &str) -> u32 {
        let tick = self.touch_counter.fetch_add(1, Ordering::Relaxed);
        if let Some(mut entry) = self.expirations.get_mut(jid) {
            entry.last_touch = tick;
            entry.value
        } else {
            0
        }
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

    pub fn set_cache(&self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

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
            _ => Err("invalid datatype for this field"),
        }
    }
}
