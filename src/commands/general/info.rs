use crate::cmd;
use crate::commands::cmd::COMMANDS;
use std::collections::HashSet;
use std::fs;

fn format_kb_to_mb(kb_str: String) -> String {
    let kb = kb_str
        .split_whitespace()
        .next()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    if kb == 0.0 { return "0MB".to_string(); }
    format!("{:.0}MB", kb / 1024.0)
}

fn get_proc_value(path: &str, key: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with(key))
        .map(|line| line.split(':').nth(1).unwrap_or("").trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_os_name() -> String {
    if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line.replace("PRETTY_NAME=", "").replace("\"", "");
            }
        }
    }
    std::env::consts::OS.to_string()
}

fn get_disk_info() -> String {
    unsafe {
        let mut stats: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c"/".as_ptr() as *const libc::c_char, &mut stats) == 0 {
            let total = (stats.f_blocks * stats.f_frsize) / 1024 / 1024 / 1024;
            let free = (stats.f_bfree * stats.f_frsize) / 1024 / 1024 / 1024;
            return format!("{}GB / {}GB Free", free, total);
        }
    }
    "Unknown".to_string()
}

fn format_duration(total_seconds: u64) -> String {
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {:02}h {:02}m {:02}s", days, hours, minutes, seconds)
    } else {
        format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
    }
}
fn get_system_uptime() -> u64 {
    fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| s as u64)
        .unwrap_or(0)
}

fn get_lib_version() -> String {
    let cargo_toml = include_str!("../../../Cargo.toml");
    for line in cargo_toml.lines() {
        if line.trim().starts_with("whatsapp-rust") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 2 {
                return parts[1].to_string();
            }
        }
    }
    "Unknown".to_string()
}

cmd!(
    Info,
    name: "info",
    aliases: ["i", "inf"],
    category: "general",
    execute: |ctx| {
        let app_name = env!("CARGO_PKG_NAME");
        let app_version = env!("CARGO_PKG_VERSION");
        let compiler = option_env!("RUSTC_VERSION").unwrap_or("Rustc (Stable)");
        let build_profile = if cfg!(debug_assertions) { "debug" } else { "release" };
        #[cfg(feature = "profiling")]
        let allocator = "dhat";
        #[cfg(feature = "stable")]
        let allocator = "Jemalloc";
        #[cfg(feature = "performance")]
        let allocator = "mimalloc";
        let mut categories = HashSet::new();
        for cmd in COMMANDS.iter() {
            categories.insert(cmd.category());
        }

        let cpu_brand = get_proc_value("/proc/cpuinfo", "model name");
        let cpu_info = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let logical_cores = cpu_info.lines().filter(|l| l.starts_with("processor")).count();
        let physical_cores = cpu_info.lines()
            .find(|l| l.starts_with("cpu cores"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(logical_cores);

        let total_mem = format_kb_to_mb(get_proc_value("/proc/meminfo", "MemTotal"));
        let free_mem = format_kb_to_mb(get_proc_value("/proc/meminfo", "MemAvailable"));

        let status_raw = fs::read_to_string("/proc/self/status").unwrap_or_default();
        let mut rss = "0 KB".to_string();
        let mut hwm = "0 KB".to_string();
        let mut vmsize = "0 KB".to_string();
        let mut vmswap = "0 KB".to_string();
        let mut vmdata = "0 KB".to_string();
        let mut threads = "0".to_string();
        let mut vmexe = "0 KB".to_string();

        for line in status_raw.lines() {
            if line.starts_with("VmRSS:") { rss = format_kb_to_mb(line.split(':').nth(1).unwrap().trim().to_string()); }
            if line.starts_with("VmHWM:") { hwm = format_kb_to_mb(line.split(':').nth(1).unwrap().trim().to_string()); }
            if line.starts_with("VmSize:") { vmsize = format_kb_to_mb(line.split(':').nth(1).unwrap().trim().to_string()); }
            if line.starts_with("VmSwap:") { vmswap = format_kb_to_mb(line.split(':').nth(1).unwrap().trim().to_string()); }
            if line.starts_with("VmData:") { vmdata = format_kb_to_mb(line.split(':').nth(1).unwrap().trim().to_string()); }
            if line.starts_with("VmExe:") { vmexe = format_kb_to_mb(line.split(':').nth(1).unwrap().trim().to_string()); }
            if line.starts_with("Threads:") { threads = line.split(':').nth(1).unwrap().trim().to_string(); }
        }

        let runtime_secs = ctx.state.start_time.elapsed().as_secs();
        let runtime = format_duration(runtime_secs);

        let system_uptime_secs = get_system_uptime();
        let system_uptime = format_duration(system_uptime_secs);

        let response = format!(
"```INFORMATION
-----------
App: {} v{}
Library: whatsapp-rust v{}
Compiler: {}
Allocator: {}
Build Profile: {}
Total Commands: {}
Total Categories: {}

STATE
-----
Current Mode: {:?}
Active Prefix: {:?}
Warmup Mode: {:?}
Runtime: {}

SYSTEM
------
OS: {}
CPU: {}
Physical Cores: {}
Logical Cores: {}
Total Memory: {}
Free Memory: {}
Disk: {}
Uptime: {}

MEMORY DETAIL
-------------
RSS (Resident): {}
Peak RSS (HWM): {}
Virtual Memory: {}
Data Segment: {}
Binary Code: {}
Swap Used: {}
Active Threads: {}
```",
            app_name,
            app_version,
            get_lib_version(),
            compiler,
            allocator,
            build_profile,
            COMMANDS.len(),
            categories.len(),
            ctx.state.get_mode(),
            ctx.state.get_prefixes(),
            ctx.state.get_warmup(),
            runtime,
            get_os_name(),
            cpu_brand,
            physical_cores,
            logical_cores,
            total_mem,
            free_mem,
            get_disk_info(),
            system_uptime,
            rss,
            hwm,
            vmsize,
            vmdata,
            vmexe,
            vmswap,
            threads
        );

        ctx.reply(&response).await?;
    }
);
