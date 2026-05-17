use crate::config;
use crate::roblox::{AuthConfig, RobloxCloudClient, SyncOrchestrator};
use crate::{parser, validator};
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
pub async fn upload(table_id: Option<String>, dry_run: bool, skip_validation: bool) -> Result<()> {
    let config_path = Path::new("slang-roblox.yaml");
    let config = config::load_config(config_path).context("Failed to load configuration")?;
    let table_id = table_id
        .or_else(|| config.cloud.as_ref().and_then(|c| c.table_id.clone()))
        .context("Table ID not provided. Specify via --table-id or set cloud.table_id in config")?;
    if !skip_validation {
        println!("{} Running pre-upload validation...", "→".blue());
        if let Err(e) = config.validate() {
            eprintln!("{} Config validation failed: {}", "✗".red(), e);
            eprintln!(
                "\n{} Use --skip-validation to bypass validation",
                "Hint:".yellow()
            );
            return Err(e);
        }
        let mut all_translations = Vec::new();
        let mut parse_errors = Vec::new();

        for locale in &config.supported_locales {
            let json_path = Path::new(&config.input_directory).join(format!("{}.json", locale));
            let yaml_path = Path::new(&config.input_directory).join(format!("{}.yaml", locale));
            let yml_path = Path::new(&config.input_directory).join(format!("{}.yml", locale));

            let translations = if json_path.exists() {
                match parser::parse_json_file(&json_path, locale) {
                    Ok(t) => t,
                    Err(e) => {
                        parse_errors.push(format!("Failed to parse {} JSON: {}", locale, e));
                        continue;
                    }
                }
            } else if let Some(path) = [&yaml_path, &yml_path].iter().find(|p| p.exists()) {
                match parser::parse_yaml_file(path, locale) {
                    Ok(t) => t,
                    Err(e) => {
                        parse_errors.push(format!("Failed to parse {} YAML: {}", locale, e));
                        continue;
                    }
                }
            } else {
                parse_errors.push(format!("No translation file found for locale: {}", locale));
                continue;
            };

            all_translations.extend(translations);
        }
        if !parse_errors.is_empty() {
            eprintln!("{} Translation parsing failed:", "✗".red());
            for error in &parse_errors {
                eprintln!("  - {}", error);
            }
            eprintln!(
                "\n{} Use --skip-validation to bypass validation",
                "Hint:".yellow()
            );
            anyhow::bail!("Translation validation failed");
        }

        if all_translations.is_empty() {
            eprintln!("{} No translations found to upload", "✗".red());
            eprintln!(
                "\n{} Use --skip-validation to bypass validation",
                "Hint:".yellow()
            );
            anyhow::bail!("No translations found");
        }
        let missing = validator::missing::detect_missing_keys(
            &all_translations,
            &config.base_locale,
            &config.supported_locales,
        );

        if !missing.is_empty() {
            eprintln!(
                "{} Missing translations detected (this may cause incomplete localization):",
                "⚠".yellow()
            );
            for (locale, keys) in &missing {
                eprintln!("  {} missing in '{}':", keys.len(), locale.yellow());
                for key in keys.iter().take(5) {
                    eprintln!("    - {}", key);
                }
                if keys.len() > 5 {
                    eprintln!("    ... and {} more", keys.len() - 5);
                }
            }
            eprintln!(
                "\n{} Use --skip-validation to upload anyway",
                "Hint:".yellow()
            );
            anyhow::bail!("Missing translations detected");
        }
        let conflicts = validator::conflicts::detect_conflicts(&all_translations);

        if !conflicts.is_empty() {
            eprintln!("{} Translation conflicts detected:", "✗".red());
            for conflict in conflicts.iter().take(10) {
                eprintln!("  - {}", conflict);
            }
            if conflicts.len() > 10 {
                eprintln!("  ... and {} more conflicts", conflicts.len() - 10);
            }
            eprintln!(
                "\n{} Use --skip-validation to upload anyway",
                "Hint:".yellow()
            );
            anyhow::bail!("Translation conflicts detected");
        }

        println!("{} Validation passed", "✓".green());
        println!(
            "  {} translations across {} locales",
            all_translations.len(),
            config.supported_locales.len()
        );
    }
    let auth = AuthConfig::load(&config).context("Failed to load authentication")?;
    let client =
        RobloxCloudClient::new(auth.api_key).context("Failed to create Roblox Cloud client")?;
    let locales_str = config.supported_locales.join(", ");
    let orchestrator = SyncOrchestrator::new(client, config);
    if dry_run {
        println!("{} Dry-run mode: No changes will be made", "ℹ".cyan());
    }

    println!("{} Uploading translations to cloud...", "→".blue());
    println!("  Table ID: {}", table_id);
    println!("  Locales: {}", locales_str);

    let stats = orchestrator
        .upload(&table_id, dry_run)
        .await
        .context("Upload failed")?;
    println!("\n{} Upload complete!", "✓".green().bold());
    println!("  Entries uploaded: {}", stats.entries_uploaded);
    println!("  Locales processed: {}", stats.locales_processed);
    println!("  Duration: {:.2}s", stats.duration.as_secs_f64());

    if dry_run {
        println!("\n{} This was a dry-run. No changes were made.", "ℹ".cyan());
    } else {
        println!(
            "\n{} Translations successfully uploaded to Roblox Cloud",
            "✓".green()
        );
    }

    Ok(())
}
