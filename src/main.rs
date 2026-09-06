#![recursion_limit = "256"]

#[cfg(target_os = "windows")]
compile_error!(
    "Sorry but this program and it's author don't want their code to be compiled in garbage OS like Windogs. Please delete your OS and install linux instead. Tq.\n- hllstr"
);

#[cfg(all(
    feature = "stable",
    not(feature = "performance"),
    not(feature = "profiling")
))]
#[unsafe(no_mangle)]
pub static malloc_conf: [u8; 73] =
    *b"background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:1000,narenas:1\0";

#[cfg(all(
    feature = "stable",
    not(feature = "performance"),
    not(feature = "profiling")
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(feature = "performance", not(feature = "profiling")))]
#[global_allocator]
static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[cfg(feature = "profiling")]
#[global_allocator]
static GLOBAL: dhat::Alloc = dhat::Alloc;

#[macro_use]
mod macros;
mod client;
mod commands;
mod config;
mod handler;
mod logger;
mod state;
mod utils;

use colored::*;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(windows) {
        panic!("Please delete your garbage OS and install Linux instead to run this program.");
    }

    #[cfg(feature = "profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let config = Arc::new(config::AppConfig::load()?);

    if config.wa_log_level != config::WaLogLevel::Off {
        let filter = config.wa_log_level.as_filter_str().to_string();
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    }

    let state = state::AppState::load(config.clone());
    let bot = client::create_bot(config.clone(), state.clone()).await?;

    let mut bot_handle = bot.spawn();

    display_startup(
        config.phone_number.as_str(),
        &if config.superuser.is_empty() {
            "None".to_string()
        } else {
            config.superuser.join(", ")
        },
        state.get_prefixes().to_vec(),
    );

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            logger::info("shutdown", "SIGINT received, performing graceful shutdown...");
            bot_handle.shutdown().await;
        }
        _ = &mut bot_handle => {}
    }
    Ok(())
}

fn display_startup(phone_number: &str, superuser: &str, prefixes: Vec<String>) {
    const LABEL_WIDTH: usize = 10;

    #[cfg(all(
        feature = "stable",
        not(feature = "performance"),
        not(feature = "profiling")
    ))]
    let allocator = "Jemalloc";
    #[cfg(all(feature = "performance", not(feature = "profiling")))]
    let allocator = "snmalloc";
    #[cfg(feature = "profiling")]
    let allocator = "dhat";

    let formatted_prefixes = prefixes
        .iter()
        .map(|p| format!("[ {} ]", p).bright_blue().to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!(
        "{}",
        "╭────────────────────────────────────────────────────────╮".bright_cyan()
    );
    println!(
        "{}  {}                 {}  {}",
        "│".bright_cyan(),
        "S O R A  O N  R U S T".bold().white(),
        format!("[ ver. {} ]", env!("CARGO_PKG_VERSION")).magenta(),
        "│".bright_cyan()
    );
    println!(
        "{}",
        "╰────────────────────────────────────────────────────────╯".bright_cyan()
    );
    println!();

    let line = |label: &str, value: String| {
        println!(
            " {} {:<width$} {}",
            "»".bright_cyan(),
            label.to_string().green(),
            value,
            width = LABEL_WIDTH
        );
    };

    line("Author", "hllstr".on_bright_black().to_string());
    line("Allocator", allocator.yellow().to_string());
    line("Bot Number", phone_number.white().to_string());
    line("Superuser", superuser.bright_red().to_string());
    line("Prefixes", formatted_prefixes);

    println!(
        "\n {}",
        "\"Nice, All set! Starting bot...\""
            .italic()
            .bright_magenta()
    );
    println!(
        "{}",
        "──────────────────────────────────────────────────────────".dimmed()
    );
}
