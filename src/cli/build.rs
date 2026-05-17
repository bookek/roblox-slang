use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::{config, generator, parser};
pub fn build(config_path: &Path) -> Result<()> {
    println!("{} Building translations...", "→".blue());
    let config = config::load_config(config_path).context("Failed to load config")?;

    println!(
        "{} Loaded config from {}",
        "✓".green(),
        config_path.display()
    );
    let mut all_translations = Vec::new();
    let mut total_keys = 0;

    for locale in &config.supported_locales {
        let json_path = Path::new(&config.input_directory).join(format!("{}.json", locale));
        let yaml_path = Path::new(&config.input_directory).join(format!("{}.yaml", locale));

        let yml_path = Path::new(&config.input_directory).join(format!("{}.yml", locale));

        let translations = if json_path.exists() {
            parser::parse_json_file(&json_path, locale)
                .context(format!("Failed to parse JSON for {}", locale))?
        } else if yaml_path.exists() {
            parser::parse_yaml_file(&yaml_path, locale)
                .context(format!("Failed to parse YAML for {}", locale))?
        } else if yml_path.exists() {
            parser::parse_yaml_file(&yml_path, locale)
                .context(format!("Failed to parse YAML for {}", locale))?
        } else {
            log::warn!("Translation file not found for locale: {}", locale);
            println!(
                "{} Translation file not found for locale: {} (tried .json, .yaml, .yml)",
                "⚠".yellow(),
                locale
            );
            continue;
        };

        let key_count = translations.len();
        total_keys += key_count;
        all_translations.extend(translations);

        println!(
            "{} Parsed {} ({} keys)",
            "✓".green(),
            locale.cyan(),
            key_count
        );
    }

    if all_translations.is_empty() {
        println!("{} No translations found", "⚠".yellow());
        return Ok(());
    }
    if let Some(override_config) = &config.overrides {
        if override_config.enabled {
            let override_path = Path::new(&override_config.file);

            if override_path.exists() {
                let overrides =
                    parser::parse_overrides(override_path).context("Failed to parse overrides")?;

                if !overrides.is_empty() {
                    println!(
                        "{} Loaded {} overrides from {}",
                        "✓".green(),
                        overrides.len(),
                        override_path.display()
                    );

                    all_translations = parser::merge_translations(all_translations, overrides);
                }
            } else {
                log::warn!("Override file not found: {}", override_path.display());
            }
        }
    }
    let output_dir = Path::new(&config.output_directory);
    std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    let luau_code = generator::generate_luau_with_full_config(
        &all_translations,
        &config.base_locale,
        config.analytics.as_ref(),
        config.localization.as_ref(),
    )
    .context("Failed to generate Luau code")?;

    let output_file = output_dir.join("Translations.lua");
    std::fs::write(&output_file, luau_code).context("Failed to write Luau file")?;

    println!("{} Generated {}", "✓".green(), output_file.display());
    let types_dir = output_dir.join("types");
    std::fs::create_dir_all(&types_dir).context("Failed to create types directory")?;

    let type_defs = generator::generate_type_definitions(&all_translations, &config.base_locale)
        .context("Failed to generate type definitions")?;

    let types_file = types_dir.join("Translations.d.luau");
    std::fs::write(&types_file, type_defs).context("Failed to write type definitions")?;

    println!("{} Generated {}", "✓".green(), types_file.display());
    let csv_content = generator::generate_csv(
        &all_translations,
        &config.base_locale,
        &config.supported_locales,
    )
    .context("Failed to generate CSV")?;

    let csv_file = output_dir.join("roblox_upload.csv");
    std::fs::write(&csv_file, csv_content).context("Failed to write CSV file")?;

    println!("{} Generated {}", "✓".green(), csv_file.display());

    println!();
    println!("{} Build completed successfully!", "✓".green().bold());
    println!("  Total keys: {}", total_keys);
    println!("  Locales: {}", config.supported_locales.join(", "));
    println!();
    println!("Generated files:");
    println!("  • {} - Main translation module", output_file.display());
    println!(
        "  • {} - Type definitions for autocomplete",
        types_file.display()
    );
    println!("  • {} - CSV for Roblox Cloud upload", csv_file.display());

    Ok(())
}
