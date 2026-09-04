use crate::cmd;
use crate::commands::cmd::COMMANDS;
use std::collections::BTreeMap;

cmd!(
    Menu,
    name: "menu",
    aliases: ["help", "h"],
    category: "general",
    execute: |ctx| {
        let mut tree_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut total_commands = 0;

        for command in COMMANDS {
            let cat = command.category().to_string();
            let name = command.name().to_string();
            tree_map.entry(cat).or_default().push(name);
            total_commands += 1;
        }

        let total_categories = tree_map.len();

        let mut output = String::from("sora-on-rust/\n");

        for (i, (category, mut cmds)) in tree_map.into_iter().enumerate() {
            cmds.sort();
            let is_last_cat = i == total_categories - 1;

            let cat_branch = if is_last_cat { "└─ " } else { "├─ " };
            let child_pipe = if is_last_cat { "    " } else { "│   " };

            output.push_str(&format!("{}{}\n", cat_branch, category));

            let cmds_count = cmds.len();
            for (j, cmd) in cmds.into_iter().enumerate() {
                let is_last_cmd = j == cmds_count - 1;

                let cmd_branch = if is_last_cmd { "└─ " } else { "├─ " };
                output.push_str(&format!("{}{}{}\n", child_pipe, cmd_branch, cmd));
            }
        }

        output.push_str(&format!(
            "\n{} category, {} commands",
            total_categories,
            total_commands
        ));

        ctx.reply(&format!("```{}```", output)).await?;
    }
);