use crate::roblox::types::CloudConfig;
use crate::utils::locales;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Main configuration structure for Roblox Slang
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Base locale (e.g., "en")
    pub base_locale: String,

    /// List of supported locales (e.g., ["en", "id", "es"])
    pub supported_locales: Vec<String>,

    /// Input directory containing translation files
    #[serde(default = "default_input_directory")]
    pub input_directory: String,

    /// Output directory for generated files
    #[serde(default = "default_output_directory")]
    pub output_directory: String,

    /// Optional namespace prefix for generated code
    #[serde(default)]
    pub namespace: Option<String>,

    /// Override configuration
    #[serde(default)]
    pub overrides: Option<OverrideConfig>,

    /// Analytics configuration
    #[serde(default)]
    pub analytics: Option<AnalyticsConfig>,

    /// Cloud sync configuration
    #[serde(default)]
    pub cloud: Option<CloudConfig>,

    /// Localization configuration
    #[serde(default)]
    pub localization: Option<LocalizationConfig>,
}

/// Override configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverrideConfig {
    /// Enable override system
    #[serde(default)]
    pub enabled: bool,

    /// Path to override file (relative to project root)
    #[serde(default = "default_override_file")]
    pub file: String,
}

/// Analytics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsConfig {
    /// Enable analytics tracking
    #[serde(default)]
    pub enabled: bool,

    /// Track missing translations
    #[serde(default = "default_true")]
    pub track_missing: bool,

    /// Track translation usage
    #[serde(default)]
    pub track_usage: bool,

    /// Optional custom callback module path
    #[serde(default)]
    pub callback: Option<String>,
}

/// Localization configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalizationConfig {
    /// Localization mode: "embedded", "cloud", or "hybrid"
    #[serde(default = "default_localization_mode")]
    pub mode: String,
}

impl LocalizationConfig {
    /// Validate localization configuration
    pub fn validate(&self) -> Result<()> {
        match self.mode.as_str() {
            "embedded" | "cloud" | "hybrid" => Ok(()),
            _ => bail!(
                "Invalid localization mode: '{}'\n\
                 \n\
                 Valid modes:\n\
                 - embedded: Use only embedded translations (default)\n\
                 - cloud: Use only LocalizationService (requires upload)\n\
                 - hybrid: Try LocalizationService first, fallback to embedded\n\
                 \n\
                 Example:\n\
                 localization:\n\
                   mode: embedded",
                self.mode
            ),
        }
    }
}

impl Config {
    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate base_locale
        if self.base_locale.is_empty() {
            bail!(
                "Configuration error: base_locale cannot be empty\n\
                 \n\
                 Expected format: base_locale: en\n\
                 \n\
                 Hint: The base_locale is your primary language (fallback).\n\
                 Common values: en, es, pt, de, fr, ja, ko, zh-cn, zh-tw"
            );
        }

        // Validate supported_locales
        if self.supported_locales.is_empty() {
            bail!(
                "Configuration error: supported_locales cannot be empty\n\
                 \n\
                 Expected format:\n\
                 supported_locales:\n\
                   - en\n\
                   - id\n\
                   - es\n\
                 \n\
                 Hint: List all languages your game will support."
            );
        }

        // Validate base_locale is in supported_locales
        if !self.supported_locales.contains(&self.base_locale) {
            bail!(
                "Configuration error: base_locale '{}' must be included in supported_locales\n\
                 \n\
                 Current supported_locales: [{}]\n\
                 \n\
                 Fix: Add '{}' to your supported_locales list:\n\
                 supported_locales:\n\
                   - {}\n\
                 {}",
                self.base_locale,
                self.supported_locales.join(", "),
                self.base_locale,
                self.base_locale,
                self.supported_locales
                    .iter()
                    .map(|l| format!("  - {}", l))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        // Validate that all locales are supported by Roblox
        let mut unsupported = Vec::new();
        for locale in &self.supported_locales {
            if !locales::is_roblox_locale(locale) {
                unsupported.push(locale.clone());
            }
        }

        if !unsupported.is_empty() {
            let supported = locales::get_supported_locale_codes();
            bail!(
                "Configuration error: Unsupported locale(s): {}\n\
                 \n\
                 Roblox supports these 17 locales:\n\
                 {}\n\
                 \n\
                 Common mistakes:\n\
                 - Using uppercase (use 'en' not 'EN')\n\
                 - Using wrong format (use 'zh-cn' not 'zh_CN')\n\
                 - Using unsupported locales\n\
                 \n\
                 Hint: Check https://create.roblox.com/docs/production/localization for details.",
                unsupported.join(", "),
                supported
                    .iter()
                    .map(|l| format!("  • {}", l))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        // Validate input_directory
        if self.input_directory.is_empty() {
            bail!(
                "Configuration error: input_directory cannot be empty\n\
                 \n\
                 Expected format: input_directory: translations\n\
                 \n\
                 Hint: This is where your JSON/YAML translation files are located."
            );
        }

        // Validate output_directory
        if self.output_directory.is_empty() {
            bail!(
                "Configuration error: output_directory cannot be empty\n\
                 \n\
                 Expected format: output_directory: output\n\
                 \n\
                 Hint: This is where generated Luau code will be placed."
            );
        }

        // Validate input and output are different
        if self.input_directory == self.output_directory {
            bail!(
                "Configuration error: input_directory and output_directory cannot be the same\n\
                 \n\
                 Current value: '{}'\n\
                 \n\
                 Hint: Use different directories to avoid overwriting source files.\n\
                 Example:\n\
                   input_directory: translations\n\
                   output_directory: output",
                self.input_directory
            );
        }

        // Validate localization config if present
        if let Some(ref localization) = self.localization {
            localization.validate()?;
        }

        Ok(())
    }
}

fn default_input_directory() -> String {
    "translations".to_string()
}

fn default_output_directory() -> String {
    "output".to_string()
}

fn default_override_file() -> String {
    "overrides.yaml".to_string()
}

fn default_true() -> bool {
    true
}

fn default_localization_mode() -> String {
    "embedded".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validate_valid() {
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

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_empty_base_locale() {
        let config = Config {
            base_locale: "".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("base_locale cannot be empty"));
    }

    #[test]
    fn test_config_validate_empty_supported_locales() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec![],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("supported_locales cannot be empty"));
    }

    #[test]
    fn test_config_validate_base_locale_not_in_supported() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["id".to_string(), "es".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be included in supported_locales"));
    }

    #[test]
    fn test_config_validate_unsupported_locale() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string(), "invalid-locale".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported locale"));
    }

    #[test]
    fn test_config_validate_empty_input_directory() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("input_directory cannot be empty"));
    }

    #[test]
    fn test_config_validate_empty_output_directory() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("output_directory cannot be empty"));
    }

    #[test]
    fn test_config_validate_same_input_output() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "same".to_string(),
            output_directory: "same".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be the same"));
    }

    #[test]
    fn test_config_with_namespace() {
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

        assert!(config.validate().is_ok());
        assert_eq!(config.namespace, Some("MyGame".to_string()));
    }

    #[test]
    fn test_override_config_defaults() {
        let override_config = OverrideConfig {
            enabled: false,
            file: default_override_file(),
        };

        assert!(!override_config.enabled);
        assert_eq!(override_config.file, "overrides.yaml");
    }

    #[test]
    fn test_analytics_config_defaults() {
        let analytics = AnalyticsConfig {
            enabled: false,
            track_missing: default_true(),
            track_usage: false,
            callback: None,
        };

        assert!(!analytics.enabled);
        assert!(analytics.track_missing);
        assert!(!analytics.track_usage);
        assert!(analytics.callback.is_none());
    }

    #[test]
    fn test_analytics_config_with_callback() {
        let analytics = AnalyticsConfig {
            enabled: true,
            track_missing: true,
            track_usage: true,
            callback: Some("game.Analytics.TrackTranslation".to_string()),
        };

        assert!(analytics.enabled);
        assert_eq!(
            analytics.callback,
            Some("game.Analytics.TrackTranslation".to_string())
        );
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_input_directory(), "translations");
        assert_eq!(default_output_directory(), "output");
        assert_eq!(default_override_file(), "overrides.yaml");
        assert!(default_true());
        assert_eq!(default_localization_mode(), "embedded");
    }

    #[test]
    fn test_localization_config_default() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        };

        assert!(config.localization.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_localization_config_embedded() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: Some(LocalizationConfig {
                mode: "embedded".to_string(),
            }),
        };

        assert!(config.validate().is_ok());
        assert_eq!(config.localization.unwrap().mode, "embedded");
    }

    #[test]
    fn test_localization_config_cloud() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: Some(LocalizationConfig {
                mode: "cloud".to_string(),
            }),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_localization_config_hybrid() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: Some(LocalizationConfig {
                mode: "hybrid".to_string(),
            }),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_localization_config_invalid_mode() {
        let config = Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: Some(LocalizationConfig {
                mode: "invalid".to_string(),
            }),
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid localization mode"));
    }
}
