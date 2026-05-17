use super::CoverageInfo;
use crate::parser::Translation;
use std::collections::{HashMap, HashSet};
pub fn generate_coverage_report(
    translations: &[Translation],
    base_locale: &str,
    supported_locales: &[String],
) -> HashMap<String, CoverageInfo> {
    let base_keys: HashSet<String> = translations
        .iter()
        .filter(|t| t.locale == base_locale)
        .map(|t| t.key.clone())
        .collect();

    let total_keys = base_keys.len();
    let mut coverage: HashMap<String, CoverageInfo> = HashMap::new();

    for locale in supported_locales {
        let locale_keys: HashSet<String> = translations
            .iter()
            .filter(|t| t.locale == locale.as_str())
            .map(|t| t.key.clone())
            .collect();

        let translated_keys = locale_keys.len();
        let missing: Vec<String> = base_keys.difference(&locale_keys).cloned().collect();

        let coverage_percent = if total_keys > 0 {
            (translated_keys as f64 / total_keys as f64) * 100.0
        } else {
            100.0
        };

        coverage.insert(
            locale.clone(),
            CoverageInfo {
                total_keys,
                translated_keys,
                missing_keys: missing,
                coverage_percent,
            },
        );
    }

    coverage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_coverage_report() {
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
        let coverage = generate_coverage_report(&translations, "en", &supported_locales);

        assert_eq!(coverage.len(), 2);
        assert_eq!(coverage["en"].coverage_percent, 100.0);
        assert_eq!(coverage["en"].translated_keys, 2);
        assert_eq!(coverage["en"].missing_keys.len(), 0);
        assert_eq!(coverage["id"].coverage_percent, 50.0);
        assert_eq!(coverage["id"].translated_keys, 1);
        assert_eq!(coverage["id"].missing_keys.len(), 1);
    }
}
