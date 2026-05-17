mod defaults;
mod schema;

pub use schema::*;

use anyhow::{bail, Result};
use std::path::Path;
pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        bail!(
            "Configuration file not found: {}\n\
             \n\
             Hint: Run `slang-roblox init` to create a default configuration.\n\
             Or create the file manually with:\n\
             \n\
             base_locale: en\n\
             supported_locales:\n\
               - en\n\
             input_directory: translations\n\
             output_directory: output",
            path.display()
        );
    }
    let content = std::fs::read_to_string(path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read configuration file: {}\n\
             Error: {}\n\
             \n\
             Hint: Check file permissions and readability.",
            path.display(),
            e
        )
    })?;
    if content.trim().is_empty() {
        bail!(
            "Configuration file is empty: {}\n\
             \n\
             Hint: Run `slang-roblox init` to create a default configuration.",
            path.display()
        );
    }
    let config: Config = serde_yaml::from_str(&content).map_err(|e| {
        let location = e.location();
        let (line, column) = if let Some(loc) = location {
            (loc.line(), loc.column())
        } else {
            (0, 0)
        };
        let lines: Vec<&str> = content.lines().collect();
        let context_line = if line > 0 && line <= lines.len() {
            lines[line - 1]
        } else {
            ""
        };

        anyhow::anyhow!(
            "Failed to parse configuration file: {}\n\
             Error at line {}, column {}: {}\n\
             \n\
             Problematic line:\n\
             {}\n\
             {}^\n\
             \n\
             Common configuration errors:\n\
             - Incorrect indentation (use 2 spaces)\n\
             - Missing colon after key\n\
             - Wrong field names (check spelling)\n\
             \n\
             Hint: Compare with the example in the documentation or run `slang-roblox init`.",
            path.display(),
            line,
            column,
            e,
            context_line,
            " ".repeat(column.saturating_sub(1))
        )
    })?;
    config.validate().map_err(|e| {
        anyhow::anyhow!(
            "{}\n\
             \n\
             Configuration file: {}",
            e,
            path.display()
        )
    })?;

    Ok(config)
}
pub fn create_default_config(path: &Path) -> Result<()> {
    let yaml = r#"# Roblox Slang Configuration
# Documentation: https://github.com/mathtechstudio/roblox-slang

# Base locale (fallback when translation is missing)
base_locale: en

# Supported locales for your game
supported_locales:
  - en

# Directory containing translation files (JSON/YAML)
input_directory: translations

# Directory for generated Luau code
output_directory: output

# Optional: Namespace for generated module (null = no namespace)
namespace: null

# Localization mode
# Controls how translations are loaded at runtime
localization:
  # Mode: embedded (default) - Translations embedded in generated code, no cloud dependency
  # Other options:
  #   - cloud: Use Roblox Cloud LocalizationService only (requires cloud upload)
  #   - hybrid: Try cloud first, fallback to embedded (best of both worlds)
  mode: embedded

# Optional: Translation overrides
# overrides:
#   enabled: true
#   file: overrides.yaml

# Optional: Analytics for tracking missing translations
# analytics:
#   enabled: true
#   track_missing: true
#   track_usage: true

# Optional: Roblox Cloud integration for syncing translations
# Enables upload, download, and bidirectional sync with Roblox Cloud Localization Tables
# cloud:
#   # Localization table ID (UUID format)
#   # Get from: Creator Dashboard > Localization > Table Settings
#   table_id: "your-table-id-here"
#   
#   # Game/Universe ID (numeric)
#   # Get from: Creator Dashboard > Game Settings > Basic Info
#   game_id: "your-game-id-here"
#   
#   # API Key (RECOMMENDED: Use environment variable instead)
#   # Set via: export ROBLOX_CLOUD_API_KEY=your_key_here
#   # Get from: https://create.roblox.com/credentials
#   # api_key: "your-api-key-here"
#   
#   # Default merge strategy for sync command
#   # Options:
#   #   - merge: Upload local-only, download cloud-only, prefer cloud for conflicts (recommended)
#   #   - overwrite: Replace all cloud translations with local (destructive)
#   #   - skip-conflicts: Only sync non-conflicting entries
#   strategy: merge
"#;

    std::fs::write(path, yaml).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write configuration file: {}\n\
             Error: {}\n\
             \n\
             Hint: Check directory permissions and write access.",
            path.display(),
            e
        )
    })?;

    Ok(())
}
pub fn create_default_overrides(path: &Path) -> Result<()> {
    let yaml = r#"# Translation Overrides
# Override specific translations per locale without modifying source files
# Useful for A/B testing, seasonal events, or quick fixes

# Example overrides:
# en:
#   ui.buttons.buy: "Purchase Now!"
#   ui.messages.greeting: "Hey there, {name}!"
#
# es:
#   ui.buttons.buy: "¡Comprar Ahora!"
#
# id:
#   ui.buttons.buy: "Beli Sekarang!"
"#;

    std::fs::write(path, yaml).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write overrides file: {}\n\
             Error: {}\n\
             \n\
             Hint: Check directory permissions and write access.",
            path.display(),
            e
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("slang-roblox.yaml");

        let yaml = r#"
base_locale: en
supported_locales:
  - en
  - id
input_directory: translations
output_directory: output
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.base_locale, "en");
        assert_eq!(config.supported_locales, vec!["en", "id"]);
        assert_eq!(config.input_directory, "translations");
        assert_eq!(config.output_directory, "output");
    }

    #[test]
    fn test_load_config_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yaml");

        let result = load_config(&config_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Configuration file not found"));
    }

    #[test]
    fn test_load_config_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty.yaml");

        fs::write(&config_path, "").unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Configuration file is empty"));
    }

    #[test]
    fn test_load_config_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");

        let yaml = r#"
base_locale: en
supported_locales:
  - en
  invalid yaml syntax here
"#;
        fs::write(&config_path, yaml).unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse configuration file"));
    }

    #[test]
    fn test_load_config_validation_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid_config.yaml");

        let yaml = r#"
base_locale: en
supported_locales:
  - id
input_directory: translations
output_directory: output
"#;
        fs::write(&config_path, yaml).unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be included in supported_locales"));
    }

    #[test]
    fn test_create_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("slang-roblox.yaml");

        create_default_config(&config_path).unwrap();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("base_locale: en"));
        assert!(content.contains("supported_locales:"));
        assert!(content.contains("input_directory: translations"));
        assert!(content.contains("output_directory: output"));
    }

    #[test]
    fn test_create_default_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let overrides_path = temp_dir.path().join("overrides.yaml");

        create_default_overrides(&overrides_path).unwrap();

        assert!(overrides_path.exists());
        let content = fs::read_to_string(&overrides_path).unwrap();
        assert!(content.contains("Translation Overrides"));
        assert!(content.contains("Example overrides:"));
    }

    #[test]
    fn test_load_config_with_namespace() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml = r#"
base_locale: en
supported_locales:
  - en
input_directory: translations
output_directory: output
namespace: MyGame
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.namespace, Some("MyGame".to_string()));
    }

    #[test]
    fn test_load_config_with_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml = r#"
base_locale: en
supported_locales:
  - en
input_directory: translations
output_directory: output
overrides:
  enabled: true
  file: custom_overrides.yaml
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(&config_path).unwrap();
        assert!(config.overrides.is_some());
        let overrides = config.overrides.unwrap();
        assert!(overrides.enabled);
        assert_eq!(overrides.file, "custom_overrides.yaml");
    }

    #[test]
    fn test_load_config_with_analytics() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let yaml = r#"
base_locale: en
supported_locales:
  - en
input_directory: translations
output_directory: output
analytics:
  enabled: true
  track_missing: true
  track_usage: false
  callback: game.Analytics.Track
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(&config_path).unwrap();
        assert!(config.analytics.is_some());
        let analytics = config.analytics.unwrap();
        assert!(analytics.enabled);
        assert!(analytics.track_missing);
        assert!(!analytics.track_usage);
        assert_eq!(analytics.callback, Some("game.Analytics.Track".to_string()));
    }
}
