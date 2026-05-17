use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

use crate::{config, generator, utils::flatten};
pub fn import_csv(csv_path: &Path, config_path: &Path) -> Result<()> {
    println!("{} Importing translations from CSV...", "→".blue());
    let config = config::load_config(config_path).context("Failed to load config")?;

    println!(
        "{} Loaded config from {}",
        "✓".green(),
        config_path.display()
    );
    if !csv_path.exists() {
        bail!("CSV file not found: {}", csv_path.display());
    }

    let csv_content = std::fs::read_to_string(csv_path)
        .context(format!("Failed to read {}", csv_path.display()))?;
    let translations = generator::parse_csv(&csv_content).context("Failed to parse CSV")?;

    if translations.is_empty() {
        println!("{} No translations found in CSV", "⚠".yellow());
        return Ok(());
    }

    println!(
        "{} Parsed {} translation entries",
        "✓".green(),
        translations.len()
    );
    let mut by_locale: HashMap<String, Vec<&crate::parser::Translation>> = HashMap::new();
    for translation in &translations {
        by_locale
            .entry(translation.locale.clone())
            .or_default()
            .push(translation);
    }
    let input_dir = Path::new(&config.input_directory);
    std::fs::create_dir_all(input_dir).context("Failed to create input directory")?;
    let locale_count = by_locale.len();
    for (locale, locale_translations) in by_locale {
        let mut flat_map: HashMap<String, String> = HashMap::new();
        for translation in locale_translations {
            flat_map.insert(translation.key.clone(), translation.value.clone());
        }
        let json = flatten::unflatten_to_json(&flat_map);
        let output_path = input_dir.join(format!("{}.json", locale));
        let json_string =
            serde_json::to_string_pretty(&json).context("Failed to serialize JSON")?;

        std::fs::write(&output_path, json_string)
            .context(format!("Failed to write {}", output_path.display()))?;

        println!(
            "{} Created {} ({} keys)",
            "✓".green(),
            output_path.display(),
            flat_map.len()
        );
    }

    println!();
    println!("{} Import completed successfully!", "✓".green().bold());
    println!("  Imported {} locales", locale_count);
    println!();
    println!("Next steps:");
    println!(
        "  1. Review the generated JSON files in {}/",
        config.input_directory
    );
    println!("  2. Run 'roblox-slang build' to generate Luau code");

    Ok(())
}
