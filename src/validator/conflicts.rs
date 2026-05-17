use crate::parser::Translation;
use std::collections::{HashMap, HashSet};
pub fn detect_conflicts(translations: &[Translation]) -> Vec<String> {
    let mut conflicts = Vec::new();
    let mut seen_keys: HashMap<(String, String), usize> = HashMap::new();
    for translation in translations {
        let key = (translation.locale.clone(), translation.key.clone());
        let count = seen_keys.entry(key.clone()).or_insert(0);
        *count += 1;

        if *count > 1 {
            conflicts.push(format!("Duplicate key '{}' in locale '{}'", key.1, key.0));
        }
    }
    let unique_conflicts: HashSet<String> = conflicts.into_iter().collect();
    unique_conflicts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_conflicts() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Purchase".to_string(),
                locale: "en".to_string(),
                context: None,
            },
        ];

        let conflicts = detect_conflicts(&translations);

        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("Duplicate key 'ui.button'"));
        assert!(conflicts[0].contains("locale 'en'"));
    }

    #[test]
    fn test_no_conflicts() {
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
        ];

        let conflicts = detect_conflicts(&translations);

        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_multiple_conflicts() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Purchase".to_string(),
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
                key: "ui.label".to_string(),
                value: "Hello".to_string(),
                locale: "en".to_string(),
                context: None,
            },
        ];

        let conflicts = detect_conflicts(&translations);

        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_same_key_different_locales_no_conflict() {
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

        let conflicts = detect_conflicts(&translations);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_triple_duplicate() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Purchase".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Get".to_string(),
                locale: "en".to_string(),
                context: None,
            },
        ];

        let conflicts = detect_conflicts(&translations);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_empty_translations() {
        let translations = vec![];
        let conflicts = detect_conflicts(&translations);

        assert_eq!(conflicts.len(), 0);
    }
}
