use crate::config::Config;
use crate::roblox::types::CloudSyncError;
use anyhow::Result;
#[derive(Debug)]
pub struct AuthConfig {
    pub api_key: String,
}

impl AuthConfig {
    pub fn load(config: &Config) -> Result<Self> {
        if let Ok(api_key) = std::env::var("ROBLOX_CLOUD_API_KEY") {
            let auth = Self { api_key };
            auth.validate()?;
            return Ok(auth);
        }
        if let Some(cloud_config) = &config.cloud {
            if let Some(api_key) = &cloud_config.api_key {
                let auth = Self {
                    api_key: api_key.clone(),
                };
                auth.validate()?;
                return Ok(auth);
            }
        }
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
