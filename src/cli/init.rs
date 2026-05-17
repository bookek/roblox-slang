use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::config;
pub fn init(with_overrides: bool) -> Result<()> {
    println!("{} Initializing Roblox Slang project...", "→".blue());
    let config_path = Path::new("slang-roblox.yaml");
    if config_path.exists() {
        println!(
            "{} Config file already exists: {}",
            "✓".yellow(),
            config_path.display()
        );
    } else {
        config::create_default_config(config_path).context("Failed to create config file")?;
        println!(
            "{} Created config file: {}",
            "✓".green(),
            config_path.display()
        );
    }
    if with_overrides {
        let overrides_path = Path::new("overrides.yaml");
        if overrides_path.exists() {
            println!(
                "{} Overrides file already exists: {}",
                "✓".yellow(),
                overrides_path.display()
            );
        } else {
            config::create_default_overrides(overrides_path)
                .context("Failed to create overrides file")?;
            println!(
                "{} Created overrides file: {}",
                "✓".green(),
                overrides_path.display()
            );
        }
    }
    let translations_dir = Path::new("translations");
    if translations_dir.exists() {
        println!(
            "{} Translations directory already exists: {}",
            "✓".yellow(),
            translations_dir.display()
        );
    } else {
        std::fs::create_dir_all(translations_dir)
            .context("Failed to create translations directory")?;
        println!(
            "{} Created translations directory: {}",
            "✓".green(),
            translations_dir.display()
        );
    }
    let example_file = translations_dir.join("en.json");
    if example_file.exists() {
        println!(
            "{} Example translation file already exists: {}",
            "✓".yellow(),
            example_file.display()
        );
    } else {
        let example_content = r#"{
  "ui": {
    "buttons": {
      "buy": "Buy",
      "sell": "Sell",
      "cancel": "Cancel"
    },
    "labels": {
      "welcome": "Welcome to the game!"
    }
  }
}
"#;
        std::fs::write(&example_file, example_content)
            .context("Failed to create example translation file")?;
        println!(
            "{} Created example translation file: {}",
            "✓".green(),
            example_file.display()
        );
    }

    println!();
    println!("{} Project initialized successfully!", "✓".green().bold());
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to add more locales",
        "slang-roblox.yaml".cyan()
    );
    println!("  2. Add translation files to {}", "translations/".cyan());
    println!(
        "  3. Run {} to generate Luau code",
        "roblox-slang build".cyan()
    );

    if with_overrides {
        println!(
            "  4. (Optional) Edit {} to override translations",
            "overrides.yaml".cyan()
        );
    }

    Ok(())
}
