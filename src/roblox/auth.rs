//! Authentication module for Roblox Cloud API
//!
//! Handles loading and validation of Roblox Cloud API keys from environment
//! variables or configuration files.

use crate::config::Config;
use crate::roblox::types::CloudSyncError;
use anyhow::Result;

/// Authentication configuration for Roblox Cloud API
///
/// Stores the API key used to authenticate requests to Roblox Cloud.
/// The key can be loaded from environment variables or configuration files.
///
/// # Example
///
/// ```no_run
/// use roblox_slang::roblox::AuthConfig;
/// use roblox_slang::config::Config;
///
/// # fn example() -> anyhow::Result<()> {
/// let config = Config::default();
/// let auth = AuthConfig::load(&config)?;
/// println!("Loaded API key");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct AuthConfig {
    /// Roblox Cloud API key
    pub api_key: String,
}

impl AuthConfig {
    /// Load API key from environment or config file
    ///
    /// Attempts to load the API key in the following priority order:
    /// 1. Environment variable: `ROBLOX_CLOUD_API_KEY`
    /// 2. Config file: `cloud.api_key` field
    /// 3. Returns error if neither found
    ///
    /// The loaded key is automatically validated for format correctness.
    ///
    /// # Arguments
    ///
    /// * `config` - Project configuration
    ///
    /// # Returns
    ///
    /// Authenticated configuration with validated API key.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No API key found in environment or config
    /// - API key format is invalid (empty or too short)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use roblox_slang::roblox::AuthConfig;
    /// use roblox_slang::config::Config;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// // Set environment variable
    /// std::env::set_var("ROBLOX_CLOUD_API_KEY", "your_api_key_here");
    ///
    /// let config = Config::default();
    /// let auth = AuthConfig::load(&config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load(config: &Config) -> Result<Self> {
        // Try environment variable first
        if let Ok(api_key) = std::env::var("ROBLOX_CLOUD_API_KEY") {
            let auth = Self { api_key };
            auth.validate()?;
            return Ok(auth);
        }

        // Try config file
        if let Some(cloud_config) = &config.cloud {
            if let Some(api_key) = &cloud_config.api_key {
                let auth = Self {
                    api_key: api_key.clone(),
                };
                auth.validate()?;
                return Ok(auth);
            }
        }

        // Neither found
        Err(CloudSyncError::ConfigError(
            "Roblox Cloud API key not found\n\
             \n\
             Please set your API key using one of these methods:\n\
             \n\
             1. Environment variable (recommended):\n\
                export ROBLOX_CLOUD_API_KEY=your_api_key_here\n\
             \n\
             2. Configuration file (slang-roblox.yaml):\n\
                cloud:\n\
                  api_key: your_api_key_here\n\
             \n\
             Get your API key from:\n\
             https://create.roblox.com/credentials"
                .to_string(),
        )
        .into())
    }

    /// Validate API key format
    ///
    /// Checks that the API key meets basic format requirements:
    /// - Not empty
    /// - At least 10 characters long
    ///
    /// # Errors
    ///
    /// Returns error if the API key is empty or too short.
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(CloudSyncError::ConfigError("API key cannot be empty".to_string()).into());
        }

        if self.api_key.len() < 10 {
            return Err(CloudSyncError::ConfigError(
                "API key appears to be invalid (too short)".to_string(),
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roblox::types::CloudConfig;

    fn create_test_config_with_api_key(api_key: Option<String>) -> Config {
        Config {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: Some(CloudConfig {
                table_id: None,
                game_id: None,
                api_key,
                strategy: None,
            }),
            localization: None,
        }
    }

    #[test]
    fn test_load_from_config() {
        let config = create_test_config_with_api_key(Some("test_api_key_12345".to_string()));
        let auth = AuthConfig::load(&config).unwrap();
        assert_eq!(auth.api_key, "test_api_key_12345");
    }

    #[test]
    fn test_load_missing_api_key() {
        let config = create_test_config_with_api_key(None);
        let result = AuthConfig::load(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("API key not found"));
    }

    #[test]
    fn test_validate_empty_key() {
        let auth = AuthConfig {
            api_key: String::new(),
        };
        let result = auth.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_short_key() {
        let auth = AuthConfig {
            api_key: "short".to_string(),
        };
        let result = auth.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_validate_valid_key() {
        let auth = AuthConfig {
            api_key: "valid_api_key_12345".to_string(),
        };
        assert!(auth.validate().is_ok());
    }
}
