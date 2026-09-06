use crate::cmd;
use crate::config::AppConfig;
use crate::state::{ConfigKey, ConfigValue};

cmd!(
    Set,
    name: "set",
    aliases: ["setting"],
    category: "root",
    access: { owner_only: true },
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
            "autoread" => {
                let _ = ctx.state.set_config(ConfigKey::Autoread, ConfigValue::Text(val_str.clone()));
                ctx.react("✅️").await?;
            },
            "show_online" => {
                let enabled = matches!(val_str.to_lowercase().as_str(), "on" | "true" | "yes" | "enable" | "enabled");
                let _ = ctx.state.set_config(ConfigKey::ShowOnline, ConfigValue::Bool(enabled));

                if enabled {
                    let _ = ctx.client.presence().set_available().await;
                } else {
                    let _ = ctx.client.presence().set_unavailable().await;
                }

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
                pairing: state.config.pairing,
                warmup: state.get_warmup(),
                autoread: state.get_autoread(),
                show_online: state.get_show_online(),
                wa_log_level: state.config.wa_log_level,
                debug_dump: state.config.debug_dump,
                mode: state.get_mode(),
                prefixes: state.get_prefixes().to_vec(),
            };

            if let Ok(toml_string) = toml::to_string(&updated_config)
                && let Err(e) = tokio::fs::write("Config.toml", toml_string).await {
                    crate::logger::error("config", format!("unable to write Config.toml: {}", e));
                }
        });
    }
);
