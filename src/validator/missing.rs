use crate::parser::Translation;
use std::collections::{HashMap, HashSet};
pub fn detect_missing_keys(
    translations: &[Translation],
    base_locale: &str,
    supported_locales: &[String],
) -> HashMap<String, Vec<String>> {
    let base_keys: HashSet<String> = translations
        .iter()
        .filter(|t| t.locale == base_locale)
        .map(|t| t.key.clone())
        .collect();
    let mut missing_by_locale: HashMap<String, Vec<String>> = HashMap::new();

    for locale in supported_locales {
        if locale == base_locale {
            continue;
        }
        let locale_keys: HashSet<String> = translations
            .iter()
            .filter(|t| t.locale == locale.as_str())
            .map(|t| t.key.clone())
            .collect();
        let missing: Vec<String> = base_keys.difference(&locale_keys).cloned().collect();

        if !missing.is_empty() {
            missing_by_locale.insert(locale.clone(), missing);
        }
    }

    missing_by_locale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_missing_keys() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.label".to_string(),
                value: "Welcome".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Beli".to_string(),
                locale: "id".to_string(),
                context: None,
            },
        ];

        let supported_locales = vec!["en".to_string(), "id".to_string()];
        let missing = detect_missing_keys(&translations, "en", &supported_locales);

        assert_eq!(missing.len(), 1);
        assert!(missing.contains_key("id"));
        assert_eq!(missing["id"], vec!["ui.label"]);
    }

    #[test]
    fn test_no_missing_keys() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Beli".to_string(),
                locale: "id".to_string(),
                context: None,
            },
        ];

        let supported_locales = vec!["en".to_string(), "id".to_string()];
        let missing = detect_missing_keys(&translations, "en", &supported_locales);

        assert_eq!(missing.len(), 0);
    }

    #[test]
    fn test_multiple_locales_missing_keys() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.label".to_string(),
                value: "Welcome".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.message".to_string(),
                value: "Hello".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Beli".to_string(),
                locale: "id".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Comprar".to_string(),
                locale: "es".to_string(),
                context: None,
            },
            Translation {
                key: "ui.label".to_string(),
                value: "Bienvenido".to_string(),
                locale: "es".to_string(),
                context: None,
            },
        ];

        let supported_locales = vec!["en".to_string(), "id".to_string(), "es".to_string()];
        let missing = detect_missing_keys(&translations, "en", &supported_locales);

        assert_eq!(missing.len(), 2);
        assert!(missing.contains_key("id"));
        assert!(missing.contains_key("es"));
        assert_eq!(missing["id"].len(), 2);
        assert_eq!(missing["es"].len(), 1);
    }

    #[test]
    fn test_empty_translations() {
        let translations = vec![];
        let supported_locales = vec!["en".to_string(), "id".to_string()];
        let missing = detect_missing_keys(&translations, "en", &supported_locales);

        assert_eq!(missing.len(), 0);
    }

    #[test]
    fn test_base_locale_only() {
        let translations = vec![Translation {
            key: "ui.button".to_string(),
            value: "Buy".to_string(),
            locale: "en".to_string(),
            context: None,
        }];

        let supported_locales = vec!["en".to_string()];
        let missing = detect_missing_keys(&translations, "en", &supported_locales);

        assert_eq!(missing.len(), 0);
    }

    #[test]
    fn test_extra_keys_in_non_base_locale() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Beli".to_string(),
                locale: "id".to_string(),
                context: None,
            },
            Translation {
                key: "ui.extra".to_string(),
                value: "Extra".to_string(),
                locale: "id".to_string(),
                context: None,
            },
        ];

        let supported_locales = vec!["en".to_string(), "id".to_string()];
        let missing = detect_missing_keys(&translations, "en", &supported_locales);
        assert_eq!(missing.len(), 0);
    }
}
