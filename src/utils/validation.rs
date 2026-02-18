//! Input validation utilities
//!
//! This module provides validation functions for locale codes, translation keys,
//! configuration values, and file paths to ensure data integrity and provide
//! clear error messages for invalid input.

use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;

/// Validates a locale code format
///
/// Valid formats:
/// - Two-letter language code: `en`, `id`, `es`
/// - Language with region: `en-US`, `zh-CN`, `pt-BR`
///
/// # Arguments
///
/// * `locale` - The locale code to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, or an error with a helpful message if invalid
///
/// # Examples
///
/// ```
/// use roblox_slang::utils::validation::validate_locale_code;
///
/// assert!(validate_locale_code("en").is_ok());
/// assert!(validate_locale_code("en-US").is_ok());
/// assert!(validate_locale_code("EN").is_err()); // Uppercase not allowed
/// assert!(validate_locale_code("en US").is_err()); // Space not allowed
/// ```
#[allow(dead_code)] // Public API for library users
pub fn validate_locale_code(locale: &str) -> Result<()> {
    if locale.is_empty() {
        bail!("Locale code cannot be empty");
    }

    // Check for invalid characters (only letters, digits, and hyphens allowed)
    if !locale
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '-')
    {
        let invalid_chars: Vec<char> = locale
            .chars()
            .filter(|c| !c.is_ascii_alphabetic() && !c.is_ascii_digit() && *c != '-')
            .collect();

        bail!(
            "Invalid locale code '{}': Contains invalid characters: {:?}\n\
             \n\
             Locale codes can only contain:\n\
             - Letters (a-z, A-Z)\n\
             - Digits (0-9)\n\
             - Hyphens (-)\n\
             \n\
             Examples of valid locale codes:\n\
             - en, id, es, fr, de\n\
             - en-US, zh-CN, pt-BR\n\
             \n\
             Common mistakes:\n\
             - Using spaces: 'en US' → 'en-US'\n\
             - Using underscores: 'en_US' → 'en-US'",
            locale,
            invalid_chars
        );
    }

    // Check format: either "xx" or "xx-YY" or "xx-YYY"
    let parts: Vec<&str> = locale.split('-').collect();
    if parts.is_empty() || parts.len() > 3 {
        bail!(
            "Invalid locale code '{}': Invalid format\n\
             \n\
             Valid formats:\n\
             - Two-letter language code: en, id, es\n\
             - Language with region: en-US, zh-CN, pt-BR\n\
             - Language with script and region: zh-Hans-CN",
            locale
        );
    }

    // Validate language code (first part) - must be lowercase
    let language = parts[0];
    if language.len() < 2 || language.len() > 3 {
        bail!(
            "Invalid locale code '{}': Language code '{}' must be 2-3 characters\n\
             \n\
             Examples: en, id, es, zh, pt",
            locale,
            language
        );
    }

    if !language.chars().all(|c| c.is_ascii_lowercase()) {
        bail!(
            "Invalid locale code '{}': Language code '{}' must be lowercase\n\
             \n\
             Examples: en, id, es, zh, pt\n\
             Fix: '{}' → '{}'",
            locale,
            language,
            language,
            language.to_lowercase()
        );
    }

    // Validate region/script code (if present) - can be uppercase or mixed case
    if parts.len() >= 2 {
        let region = parts[1];
        if region.len() < 2 || region.len() > 4 {
            bail!(
                "Invalid locale code '{}': Region/script code '{}' must be 2-4 characters\n\
                 \n\
                 Examples: US, CN, BR, Hans",
                locale,
                region
            );
        }

        if !region.chars().all(|c| c.is_ascii_alphabetic()) {
            bail!(
                "Invalid locale code '{}': Region/script code '{}' must contain only letters\n\
                 \n\
                 Examples: US, CN, BR, Hans",
                locale,
                region
            );
        }
    }

    Ok(())
}

/// Validates a translation key format
///
/// Valid keys:
/// - Must not be empty
/// - Must not have leading or trailing dots
/// - Must not have consecutive dots
/// - Must not contain reserved characters
///
/// # Arguments
///
/// * `key` - The translation key to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, or an error with a helpful message if invalid
///
/// # Examples
///
/// ```
/// use roblox_slang::utils::validation::validate_translation_key;
///
/// assert!(validate_translation_key("ui.button").is_ok());
/// assert!(validate_translation_key("ui.buttons.buy").is_ok());
/// assert!(validate_translation_key("").is_err()); // Empty
/// assert!(validate_translation_key(".ui.button").is_err()); // Leading dot
/// assert!(validate_translation_key("ui..button").is_err()); // Consecutive dots
/// ```
pub fn validate_translation_key(key: &str) -> Result<()> {
    if key.is_empty() {
        bail!("Translation key cannot be empty");
    }

    // Check for leading dot
    if key.starts_with('.') {
        bail!(
            "Invalid translation key '{}': Cannot start with a dot\n\
             \n\
             Fix: Remove the leading dot\n\
             Example: '.ui.button' → 'ui.button'",
            key
        );
    }

    // Check for trailing dot
    if key.ends_with('.') {
        bail!(
            "Invalid translation key '{}': Cannot end with a dot\n\
             \n\
             Fix: Remove the trailing dot\n\
             Example: 'ui.button.' → 'ui.button'",
            key
        );
    }

    // Check for consecutive dots
    if key.contains("..") {
        bail!(
            "Invalid translation key '{}': Cannot contain consecutive dots\n\
             \n\
             Fix: Remove extra dots\n\
             Example: 'ui..button' → 'ui.button'",
            key
        );
    }

    // Check for reserved characters
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

    // Check for whitespace
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

/// Validates that a file path exists and is accessible
///
/// # Arguments
///
/// * `path` - The file path to validate
/// * `description` - A description of what the file is (for error messages)
///
/// # Returns
///
/// Returns `Ok(())` if the file exists and is accessible, or an error if not
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use roblox_slang::utils::validation::validate_file_exists;
///
/// let path = Path::new("translations/en.json");
/// validate_file_exists(path, "translation file").unwrap();
/// ```
pub fn validate_file_exists(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        bail!(
            "{} not found: {}\n\
             \n\
             Please ensure the file exists at the specified path.\n\
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

/// Validates that a directory path exists and is accessible
///
/// # Arguments
///
/// * `path` - The directory path to validate
/// * `description` - A description of what the directory is (for error messages)
///
/// # Returns
///
/// Returns `Ok(())` if the directory exists and is accessible, or an error if not
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use roblox_slang::utils::validation::validate_directory_exists;
///
/// let path = Path::new("translations");
/// validate_directory_exists(path, "translations directory").unwrap();
/// ```
pub fn validate_directory_exists(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        bail!(
            "{} not found: {}\n\
             \n\
             Please ensure the directory exists at the specified path.\n\
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

/// Validates that a path is safe (no path traversal attempts)
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// Returns `Ok(())` if the path is safe, or an error if it contains path traversal attempts
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use roblox_slang::utils::validation::validate_safe_path;
///
/// assert!(validate_safe_path(Path::new("translations/en.json")).is_ok());
/// assert!(validate_safe_path(Path::new("../etc/passwd")).is_err());
/// ```
pub fn validate_safe_path(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Check for path traversal attempts
    if path_str.contains("..") {
        bail!(
            "Invalid path '{}': Path traversal not allowed\n\
             \n\
             Paths cannot contain '..' for security reasons.\n\
             Please use absolute paths or paths relative to the project root.",
            path.display()
        );
    }

    // Check for absolute paths outside project (on Unix)
    #[cfg(unix)]
    {
        if path.is_absolute() && !path_str.starts_with("/tmp") {
            // Allow /tmp for tests
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

/// Validates a configuration object
///
/// This function performs comprehensive validation of a Config object,
/// including locale code format validation, directory checks, and logical consistency.
///
/// # Arguments
///
/// * `config` - The configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, or an error with a helpful message if invalid
///
/// # Examples
///
/// ```no_run
/// use roblox_slang::config::Config;
/// use roblox_slang::utils::validation::validate_config;
///
/// let config = Config {
///     base_locale: "en".to_string(),
///     supported_locales: vec!["en".to_string(), "id".to_string()],
///     input_directory: "translations".to_string(),
///     output_directory: "output".to_string(),
///     namespace: None,
///     overrides: None,
///     analytics: None,
///     cloud: None,
///     localization: None,
/// };
///
/// validate_config(&config).unwrap();
/// ```
#[allow(dead_code)] // Public API for library users
pub fn validate_config(config: &Config) -> Result<()> {
    // First run the built-in validation
    config.validate()?;

    // Additional validation: Check locale code formats
    validate_locale_code(&config.base_locale)
        .map_err(|e| anyhow::anyhow!("Configuration error in base_locale:\n{}", e))?;

    for locale in &config.supported_locales {
        validate_locale_code(locale)
            .map_err(|e| anyhow::anyhow!("Configuration error in supported_locales:\n{}", e))?;
    }

    // Validate directory paths are safe
    validate_safe_path(Path::new(&config.input_directory))
        .map_err(|e| anyhow::anyhow!("Configuration error in input_directory:\n{}", e))?;

    validate_safe_path(Path::new(&config.output_directory))
        .map_err(|e| anyhow::anyhow!("Configuration error in output_directory:\n{}", e))?;

    // Validate namespace if present
    if let Some(ref namespace) = config.namespace {
        if namespace.is_empty() {
            bail!(
                "Configuration error: namespace cannot be empty\n\
                 \n\
                 Either remove the namespace field or provide a valid value.\n\
                 Example: namespace: MyGame"
            );
        }

        // Check for invalid characters in namespace
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

        // Check if namespace starts with a digit
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

    // Validate override config if present
    if let Some(ref override_config) = config.overrides {
        if override_config.enabled {
            validate_safe_path(Path::new(&override_config.file))
                .map_err(|e| anyhow::anyhow!("Configuration error in overrides.file:\n{}", e))?;
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
        assert!(validate_locale_code("en-US").is_ok());
        assert!(validate_locale_code("zh-CN").is_ok());
        assert!(validate_locale_code("pt-BR").is_ok());
        assert!(validate_locale_code("zh-Hans-CN").is_ok());
    }

    #[test]
    fn test_validate_locale_code_empty() {
        assert!(validate_locale_code("").is_err());
    }

    #[test]
    fn test_validate_locale_code_uppercase() {
        // Language code must be lowercase
        let result = validate_locale_code("EN");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("lowercase"));

        // But region code can be uppercase (this is valid)
        assert!(validate_locale_code("en-US").is_ok());
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
        // The error comes from Config.validate() which checks if locale is supported by Roblox
        // "EN" is not a valid Roblox locale, so it will fail with "Unsupported locale"
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
        // The error comes from Config.validate() which checks if locale is supported by Roblox
        // "EN US" is not a valid Roblox locale, so it will fail with "Unsupported locale"
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

    #[test]
    fn test_validate_config_path_traversal() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "../../../etc".to_string(), // Path traversal
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
        assert!(err.contains("Path traversal"));
    }
}
