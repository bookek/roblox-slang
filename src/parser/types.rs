use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Translation {
    pub key: String,
    pub value: String,
    pub locale: String,
    pub context: Option<String>,
}

#[allow(dead_code)]
pub type TranslationMap = HashMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_creation() {
        let translation = Translation {
            key: "ui.button".to_string(),
            value: "Buy".to_string(),
            locale: "en".to_string(),
            context: None,
        };

        assert_eq!(translation.key, "ui.button");
        assert_eq!(translation.value, "Buy");
        assert_eq!(translation.locale, "en");
        assert!(translation.context.is_none());
    }

    #[test]
    fn test_translation_with_context() {
        let translation = Translation {
            key: "common.close".to_string(),
            value: "Close".to_string(),
            locale: "en".to_string(),
            context: Some("button".to_string()),
        };

        assert_eq!(translation.context, Some("button".to_string()));
    }

    #[test]
    fn test_translation_clone() {
        let translation = Translation {
            key: "test.key".to_string(),
            value: "Test Value".to_string(),
            locale: "en".to_string(),
            context: None,
        };

        let cloned = translation.clone();
        assert_eq!(translation, cloned);
    }

    #[test]
    fn test_translation_equality() {
        let t1 = Translation {
            key: "key".to_string(),
            value: "value".to_string(),
            locale: "en".to_string(),
            context: None,
        };

        let t2 = Translation {
            key: "key".to_string(),
            value: "value".to_string(),
            locale: "en".to_string(),
            context: None,
        };

        assert_eq!(t1, t2);
    }

    #[test]
    fn test_translation_map() {
        let mut map: TranslationMap = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    }
}
