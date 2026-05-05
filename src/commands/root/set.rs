use crate::cmd;
use crate::config::AppConfig;
use crate::state::{ConfigKey, ConfigValue};

cmd!(
    Set,
    name: "set",
    aliases: ["setting"],
    category: "root",
    execute: |ctx| {
        if ctx.args.len() < 2 {
            ctx.react("❔").await?;
            return Ok(());
        }
        let key = ctx.args[0].to_lowercase();
        let val_str = ctx.args[1..].join(" ");

        match key.as_str() {
            "mode" => {
                let _ = ctx.state.set_config(ConfigKey::Mode, ConfigValue::Text(val_str.clone()));
                ctx.react("✅️").await?;
            },
            "prefixes" | "prefix" => {
                let new_prefixes: Vec<String> = val_str.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                let _ = ctx.state.set_config(ConfigKey::Prefixes, ConfigValue::List(new_prefixes));
                ctx.react("✅️").await?;
            },
            "warmup" => {
                let _ = ctx.state.set_config(ConfigKey::Warmup, ConfigValue::Text(val_str.clone()));
                ctx.react("✅️").await?;
            },
            _ => {
                ctx.react("❔").await?;
                return Ok(());
            }
        };

        let state = ctx.state.clone();
        tokio::spawn(async move {
            let updated_config = AppConfig {
                phone_number: state.config.phone_number.clone(),
                superuser: state.config.superuser.clone(),
                custom_code: state.config.custom_code.clone(),
                session_path: state.config.session_path.clone(),
                warmup: state.get_warmup(),
                mode: state.get_mode(),
                prefixes: state.get_prefixes().to_vec(),
            };

            if let Ok(toml_string) = toml::to_string(&updated_config)
                && let Err(e) = tokio::fs::write("Config.toml", toml_string).await {
                    eprintln!("unable write config to Config.toml: {}", e);
                }
        });
    }
);
