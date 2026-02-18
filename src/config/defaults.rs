use super::Config;

impl Default for Config {
    fn default() -> Self {
        Self {
            base_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
            input_directory: "translations".to_string(),
            output_directory: "output".to_string(),
            namespace: None,
            overrides: None,
            analytics: None,
            cloud: None,
            localization: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.base_locale, "en");
        assert_eq!(config.supported_locales, vec!["en"]);
        assert_eq!(config.input_directory, "translations");
        assert_eq!(config.output_directory, "output");
        assert!(config.namespace.is_none());
        assert!(config.overrides.is_none());
        assert!(config.analytics.is_none());
        assert!(config.cloud.is_none());
    }

    #[test]
    fn test_config_default_validates() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
}
