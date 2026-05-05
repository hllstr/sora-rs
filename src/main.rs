#[cfg(target_os = "windows")]
compile_error!(
    "Sorry but this program and it's author don't want their code to be compiled in garbage OS like Windogs. Please delete your OS and install linux instead. Tq.\n- hllstr"
);

#[cfg(feature = "stable")]
#[unsafe(no_mangle)]
pub static malloc_conf: [u8; 73] =
    *b"background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:1000,narenas:1\0";

#[cfg(feature = "stable")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(feature = "performance")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
use log::info;
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
    let state = state::AppState::load(config.clone());
    let mut bot = client::create_bot(config.clone(), state.clone()).await?;

    let client = bot.client().clone();
    let bot_handle = bot.run().await?;

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
            info!("SIGINT received, Performing graceful shutdown...");
            client.disconnect().await;
        }
        res = bot_handle => {
            res?;
        }
    }
    Ok(())
}

fn display_startup(phone_number: &str, superuser: &str, prefixes: Vec<String>) {
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

    println!(
        " {} {}    : {}",
        "»".bright_cyan(),
        "Author ".green(),
        "hllstr".on_bright_black()
    );
    #[cfg(feature = "profiling")]
    let allocator = "dhat";
    #[cfg(feature = "stable")]
    let allocator = "Jemalloc";
    #[cfg(feature = "performance")]
    let allocator = "mimalloc";
    println!(
        " {} {} : {}",
        "»".bright_cyan(),
        "Allocator ".green(),
        allocator.yellow()
    );

    println!(
        " {} {} : {}",
        "»".bright_cyan(),
        "Bot Number".green(),
        phone_number.white()
    );
    println!(
        " {} {}: {}",
        "»".bright_cyan(),
        "Superuser  ".green(),
        superuser.bright_red()
    );

    let formatted_prefixes = prefixes
        .iter()
        .map(|p| format!("[ {} ]", p).bright_blue().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        " {} {}   : {}",
        "»".bright_cyan(),
        "Prefixes".green(),
        formatted_prefixes
    );

    println!(
        "\n {}",
        " \"Nice, All set! Starting bot...\""
            .italic()
            .bright_magenta()
    );
    println!(
        "{}",
        "──────────────────────────────────────────────────────────".dimmed()
    );
}
