use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;
pub fn validate_locale_code(locale: &str) -> Result<()> {
    if crate::utils::locales::is_roblox_locale(locale) {
        return Ok(());
    }

    let supported = crate::utils::locales::get_supported_locale_codes();
    bail!(
        "Unsupported locale '{}'\n\
         \n\
         Roblox supports these locales:\n\
         {}",
        locale,
        supported.join(", ")
    )
}
pub fn validate_translation_key(key: &str) -> Result<()> {
    if key.is_empty() {
        bail!("Translation key cannot be empty");
    }
    if key.starts_with('.') {
        bail!(
            "Invalid translation key '{}': Cannot start with a dot\n\
             \n\
             Fix: Remove the leading dot\n\
             Example: '.ui.button' → 'ui.button'",
            key
        );
    }
    if key.ends_with('.') {
        bail!(
            "Invalid translation key '{}': Cannot end with a dot\n\
             \n\
             Fix: Remove the trailing dot\n\
             Example: 'ui.button.' → 'ui.button'",
            key
        );
    }
    if key.contains("..") {
        bail!(
            "Invalid translation key '{}': Cannot contain consecutive dots\n\
             \n\
             Fix: Remove extra dots\n\
             Example: 'ui..button' → 'ui.button'",
            key
        );
    }
    let reserved_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    if let Some(invalid_char) = key.chars().find(|c| reserved_chars.contains(c)) {
        bail!(
            "Invalid translation key '{}': Contains reserved character '{}'\n\
             \n\
             Reserved characters that cannot be used in keys:\n\
             / \\ : * ? \" < > | (null)\n\
             \n\
             These characters are reserved for file system compatibility.",
            key,
            invalid_char
        );
    }
    if key.chars().any(|c| c.is_whitespace()) {
        bail!(
            "Invalid translation key '{}': Cannot contain whitespace\n\
             \n\
             Fix: Replace spaces with dots or underscores\n\
             Example: 'ui button' → 'ui.button' or 'ui_button'",
            key
        );
    }

    Ok(())
}
pub fn validate_file_exists(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        bail!(
            "{} not found: {}\n\
             \n\
             Check that the file exists at the specified path.\n\
             \n\
             If you're starting a new project, run:\n\
             roblox-slang init",
            description
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .to_string()
                + &description[1..],
            path.display()
        );
    }

    if !path.is_file() {
        bail!(
            "{} is not a file: {}\n\
             \n\
             Expected a file, but found a directory.",
            description
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .to_string()
                + &description[1..],
            path.display()
        );
    }

    Ok(())
}
pub fn validate_directory_exists(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        bail!(
            "{} not found: {}\n\
             \n\
             Check that the directory exists at the specified path.\n\
             \n\
             If you're starting a new project, run:\n\
             roblox-slang init",
            description
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .to_string()
                + &description[1..],
            path.display()
        );
    }

    if !path.is_dir() {
        bail!(
            "{} is not a directory: {}\n\
             \n\
             Expected a directory, but found a file.",
            description
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .to_string()
                + &description[1..],
            path.display()
        );
    }

    Ok(())
}
pub fn validate_safe_path(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        bail!(
            "Invalid path '{}': Path traversal not allowed\n\
             \n\
             Paths cannot contain '..' for security reasons.\n\
             Please use absolute paths or paths relative to the project root.",
            path.display()
        );
    }
    #[cfg(unix)]
    {
        if path.is_absolute() && !path_str.starts_with("/tmp") {
            if let Ok(current_dir) = std::env::current_dir() {
                if let Ok(canonical) = path.canonicalize() {
                    if !canonical.starts_with(&current_dir) {
                        bail!(
                            "Invalid path '{}': Absolute paths outside project directory not allowed\n\
                             \n\
                             Please use paths relative to the project root.",
                            path.display()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
pub fn validate_config(config: &Config) -> Result<()> {
    config.validate()?;
    validate_locale_code(&config.base_locale)
        .map_err(|e| anyhow::anyhow!("Configuration error in base_locale:\n{}", e))?;

    for locale in &config.supported_locales {
        validate_locale_code(locale)
            .map_err(|e| anyhow::anyhow!("Configuration error in supported_locales:\n{}", e))?;
    }
    if let Some(ref namespace) = config.namespace {
        if namespace.is_empty() {
            bail!(
                "Configuration error: namespace cannot be empty\n\
                 \n\
                 Either remove the namespace field or provide a valid value.\n\
                 Example: namespace: MyGame"
            );
        }
        if !namespace.chars().all(|c| c.is_alphanumeric() || c == '_') {
            bail!(
                "Configuration error: namespace '{}' contains invalid characters\n\
                 \n\
                 Namespace can only contain:\n\
                 - Letters (a-z, A-Z)\n\
                 - Digits (0-9)\n\
                 - Underscores (_)\n\
                 \n\
                 Example: namespace: MyGame_Translations",
                namespace
            );
        }
        if namespace.chars().next().unwrap().is_ascii_digit() {
            bail!(
                "Configuration error: namespace '{}' cannot start with a digit\n\
                 \n\
                 Luau identifiers cannot start with digits.\n\
                 Example: '{}' → 'Game{}'",
                namespace,
                namespace,
                namespace
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_locale_code_valid() {
        assert!(validate_locale_code("en").is_ok());
        assert!(validate_locale_code("id").is_ok());
        assert!(validate_locale_code("es").is_ok());
        assert!(validate_locale_code("zh-cn").is_ok());
        assert!(validate_locale_code("zh-tw").is_ok());
    }

    #[test]
    fn test_validate_locale_code_empty() {
        assert!(validate_locale_code("").is_err());
    }

    #[test]
    fn test_validate_locale_code_uppercase() {
        let result = validate_locale_code("EN");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported"));
        assert!(validate_locale_code("en-us").is_err());
    }

    #[test]
    fn test_validate_locale_code_invalid_chars() {
        assert!(validate_locale_code("en US").is_err());
        assert!(validate_locale_code("en_US").is_err());
        assert!(validate_locale_code("en@US").is_err());
    }

    #[test]
    fn test_validate_locale_code_invalid_format() {
        assert!(validate_locale_code("e").is_err()); // Too short
        assert!(validate_locale_code("engl").is_err()); // Too long
    }

    #[test]
    fn test_validate_translation_key_valid() {
        assert!(validate_translation_key("ui").is_ok());
        assert!(validate_translation_key("ui.button").is_ok());
        assert!(validate_translation_key("ui.buttons.buy").is_ok());
        assert!(validate_translation_key("ui.buttons.buy_now").is_ok());
        assert!(validate_translation_key("ui.buttons.buy-now").is_ok());
    }

    #[test]
    fn test_validate_translation_key_empty() {
        assert!(validate_translation_key("").is_err());
    }

    #[test]
    fn test_validate_translation_key_leading_dot() {
        let result = validate_translation_key(".ui.button");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot start with a dot"));
    }

    #[test]
    fn test_validate_translation_key_trailing_dot() {
        let result = validate_translation_key("ui.button.");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot end with a dot"));
    }

    #[test]
    fn test_validate_translation_key_consecutive_dots() {
        let result = validate_translation_key("ui..button");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("consecutive dots"));
    }

    #[test]
    fn test_validate_translation_key_reserved_chars() {
        assert!(validate_translation_key("ui/button").is_err());
        assert!(validate_translation_key("ui\\button").is_err());
        assert!(validate_translation_key("ui:button").is_err());
        assert!(validate_translation_key("ui*button").is_err());
        assert!(validate_translation_key("ui?button").is_err());
    }

    #[test]
    fn test_validate_translation_key_whitespace() {
        let result = validate_translation_key("ui button");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("whitespace"));
    }

    #[test]
    fn test_validate_safe_path_valid() {
        assert!(validate_safe_path(Path::new("translations/en.json")).is_ok());
        assert!(validate_safe_path(Path::new("output/Translations.lua")).is_ok());
    }

    #[test]
    fn test_validate_safe_path_traversal() {
        let result = validate_safe_path(Path::new("../etc/passwd"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Path traversal"));
    }

    #[test]
    fn test_validate_safe_path_double_dot() {
        assert!(validate_safe_path(Path::new("../../secret")).is_err());
        assert!(validate_safe_path(Path::new("translations/../../../etc")).is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string(), "id".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_with_namespace() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: Some("MyGame".to_string()),
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_invalid_base_locale() {
        let config = Config {
            base_locale: "EN".to_string(), // Uppercase not allowed
            supported_locales: vec!["EN".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported") || err.contains("base_locale"));
    }

    #[test]
    fn test_validate_config_invalid_supported_locale() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string(), "EN US".to_string()], // Invalid format
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported") || err.contains("supported_locales"));
    }

    #[test]
    fn test_validate_config_empty_namespace() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: Some("".to_string()), // Empty namespace
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("namespace"));
    }

    #[test]
    fn test_validate_config_invalid_namespace_chars() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: Some("My-Game".to_string()), // Hyphen not allowed
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid characters"));
    }

    #[test]
    fn test_validate_config_namespace_starts_with_digit() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: Some("123Game".to_string()), // Starts with digit
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot start with a digit"));
    }
}
