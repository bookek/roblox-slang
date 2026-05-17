use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::{config, parser, validator};
pub fn validate(
    config_path: &Path,
    check_missing: bool,
    check_unused: bool,
    check_conflicts: bool,
    show_coverage: bool,
    source_dir: Option<&Path>,
) -> Result<()> {
    println!("{} Validating translations...", "→".blue());
    let config = config::load_config(config_path).context("Failed to load config")?;
    let mut all_translations = Vec::new();

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
            continue;
        };

        all_translations.extend(translations);
    }

    if all_translations.is_empty() {
        println!("{} No translations found", "⚠".yellow());
        return Ok(());
    }

    let mut has_issues = false;
    if check_missing {
        println!("\n{} Checking for missing translations...", "→".blue());
        let missing = validator::missing::detect_missing_keys(
            &all_translations,
            &config.base_locale,
            &config.supported_locales,
        );

        if missing.is_empty() {
            println!("{} No missing translations found", "✓".green());
        } else {
            has_issues = true;
            for (locale, keys) in &missing {
                println!("\n{} Missing in '{}':", "✗".red(), locale.yellow());
                for key in keys {
                    println!("  - {}", key);
                }
            }
        }
    }
    if check_conflicts {
        println!("\n{} Checking for conflicts...", "→".blue());
        let conflicts = validator::conflicts::detect_conflicts(&all_translations);

        if conflicts.is_empty() {
            println!("{} No conflicts found", "✓".green());
        } else {
            has_issues = true;
            println!("\n{} Conflicts found:", "✗".red());
            for conflict in &conflicts {
                println!("  - {}", conflict);
            }
        }
    }
    if check_unused {
        if let Some(src_dir) = source_dir {
            println!(
                "\n{} Checking for unused keys in {}...",
                "→".blue(),
                src_dir.display()
            );
            let keys: Vec<String> = all_translations
                .iter()
                .filter(|t| t.locale == config.base_locale)
                .map(|t| t.key.clone())
                .collect();

            let unused = validator::unused::detect_unused_keys(&keys, src_dir)
                .context("Failed to detect unused keys")?;

            if unused.is_empty() {
                println!("{} No unused keys found", "✓".green());
            } else {
                has_issues = true;
                println!("\n{} Unused keys:", "⚠".yellow());
                for key in &unused {
                    println!("  - {}", key);
                }
            }
        } else {
            println!(
                "{} Skipping unused keys check (no source directory specified)",
                "⚠".yellow()
            );
        }
    }
    if show_coverage {
        println!("\n{} Translation Coverage Report", "→".blue());
        println!();

        let coverage = validator::coverage::generate_coverage_report(
            &all_translations,
            &config.base_locale,
            &config.supported_locales,
        );
        println!(
            "{:<10} {:<10} {:<12} {:<10}",
            "Locale", "Keys", "Coverage", "Missing"
        );
        println!("{}", "-".repeat(45));
        for locale in &config.supported_locales {
            if let Some(info) = coverage.get(locale) {
                let coverage_str = format!("{:.1}%", info.coverage_percent);
                let missing_str = if info.missing_keys.is_empty() {
                    "-".to_string()
                } else {
                    info.missing_keys.len().to_string()
                };

                println!(
                    "{:<10} {:<10} {:<12} {:<10}",
                    locale, info.translated_keys, coverage_str, missing_str
                );
            }
        }
        let total_keys: usize = coverage.values().map(|c| c.total_keys).max().unwrap_or(0);
        let total_translated: usize = coverage.values().map(|c| c.translated_keys).sum();
        let total_possible = total_keys * config.supported_locales.len();
        let overall_coverage = if total_possible > 0 {
            (total_translated as f64 / total_possible as f64) * 100.0
        } else {
            100.0
        };

        println!();
        println!("Overall: {:.1}% coverage", overall_coverage);
    }

    println!();
    if has_issues {
        println!("{} Validation completed with issues", "⚠".yellow().bold());
    } else {
        println!("{} Validation completed successfully!", "✓".green().bold());
    }

    Ok(())
}
