use crate::cmd;

cmd!(
    InspectCache,
    name: "cache",
    aliases: ["listcache", "inspectcache"],
    category: "root",
    access: { owner_only: true },
    execute: |ctx| {
        if ctx.state.cache.is_empty() {
            ctx.reply("*Empty!*").await?;
            return Ok(());
        }

        let mut response = String::from("*In-Memory Cache (DashMap)*\n\n");
        let mut count = 0;

        for entry in ctx.state.cache.iter() {
            count += 1;
            let key = entry.key();
            let value = entry.value();

            let display_val = if value.len() > 40 {
                format!("{}...", &value[..40])
            } else {
                value.clone()
            };

            response.push_str(&format!("*•* `{}`\n  ↳ _{}_\n", key, display_val));


        }

        response.push_str(&format!("\n*Total Unique Keys:* {}", count));
        response.push_str("\n*Uptime:* ");

        let uptime = ctx.state.start_time.elapsed();
        response.push_str(&format!("{}s", uptime.as_secs()));

        ctx.reply(&response).await?;
    }
);
