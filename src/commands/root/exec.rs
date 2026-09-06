use crate::cmd;
use tokio::process::Command;
cmd!(
    Exec,
    name: "exec",
    aliases: ["$"],
    category: "root",
    access: { owner_only: true },
    execute: |ctx| {
        let command = ctx.body;
        let output = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .await?;

        let mut result = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() {
            result = String::from_utf8_lossy(&output.stderr)
        }

        ctx.reply(result.trim()).await?;
    }
);
