use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    if let Ok(output) = Command::new("rustc").arg("-V").output()
        && let Ok(version_full) = String::from_utf8(output.stdout)
    {
        let version = version_full
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");

        println!("cargo:rustc-env=RUSTC_VERSION={}", version);
    }
    let commands_path = Path::new("src/commands");
    let content = generate_mod_content(commands_path);
    let mod_file = commands_path.join("mod.rs");
    fs::write(mod_file, content).expect("Unable to write src/commands/mod.rs");

    println!("cargo:rerun-if-changed=src/commands");
}

fn generate_mod_content(path: &Path) -> String {
    let mut entries: Vec<_> = fs::read_dir(path)
        .unwrap()
        .filter_map(|res| res.ok())
        .collect();

    entries.sort_by_key(|e| e.path());

    let mut lines = Vec::new();

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        if path.is_dir() {
            let inner_content = generate_mod_content(&path);
            if !inner_content.trim().is_empty() {
                let indented = inner_content.replace("\n", "\n    ");
                lines.push(format!("pub mod {} {{\n    {}\n}}", file_name, indented));
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();

            if file_stem != "mod" {
                lines.push(format!("pub mod {};", file_stem));
            }
        }
    }

    lines.join("\n")
}
