use crate::parser::Translation;
use anyhow::{Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashMap;

pub fn generate_csv(
    translations: &[Translation],
    base_locale: &str,
    locales: &[String],
) -> Result<String> {
    let mut writer = WriterBuilder::new().from_writer(Vec::new());

    let mut header = vec![
        "Source".to_string(),
        "Context".to_string(),
        "Key".to_string(),
    ];
    header.extend(locales.iter().cloned());
    writer.write_record(&header)?;

    let mut translation_map: HashMap<String, (Option<String>, HashMap<String, String>)> =
        HashMap::new();

    for translation in translations {
        let entry = translation_map
            .entry(translation.key.clone())
            .or_insert_with(|| (translation.context.clone(), HashMap::new()));

        entry
            .1
            .insert(translation.locale.clone(), translation.value.clone());
    }
    let mut keys: Vec<_> = translation_map.keys().cloned().collect();
    keys.sort();
    for key in keys {
        let (context, locale_values) = translation_map.get(&key).unwrap();
        let mut record = Vec::with_capacity(3 + locales.len());
        record.push(locale_values.get(base_locale).cloned().unwrap_or_default());
        record.push(context.as_ref().cloned().unwrap_or_default());
        record.push(key);
        for locale in locales {
            record.push(locale_values.get(locale).cloned().unwrap_or_default());
        }

        writer.write_record(&record)?;
    }

    let bytes = writer.into_inner().context("Failed to write CSV")?;
    String::from_utf8(bytes).context("CSV output was not UTF-8")
}

pub fn parse_csv(content: &str) -> Result<Vec<Translation>> {
    let mut translations = Vec::new();
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(content.as_bytes());
    let mut records = reader.records();
    let header = records
        .next()
        .ok_or_else(|| anyhow::anyhow!("CSV file is empty"))??;
    let headers: Vec<String> = header
        .iter()
        .map(|value| value.trim().to_string())
        .collect();

    if headers.len() < 3 {
        anyhow::bail!("Invalid CSV header: expected at least Source,Context,Key columns");
    }
    let locales: Vec<String> = headers[3..].to_vec();
    for record in records {
        let values = record?;
        if values.iter().all(|value| value.trim().is_empty()) {
            continue;
        }

        if values.len() < 3 {
            log::warn!("Skipping invalid CSV row with {} column(s)", values.len());
            continue;
        }

        let key = values.get(2).unwrap_or("").trim().to_string();
        let context_value = values.get(1).unwrap_or("").trim();
        let context = if context_value.is_empty() {
            None
        } else {
            Some(context_value.to_string())
        };
        for (i, locale) in locales.iter().enumerate() {
            let value_index = 3 + i;
            if value_index < values.len() {
                let value = values.get(value_index).unwrap_or("");
                if !value.is_empty() {
                    translations.push(Translation {
                        key: key.clone(),
                        value: value.to_string(),
                        locale: locale.clone(),
                        context: context.clone(),
                    });
                }
            }
        }
    }

    Ok(translations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_csv() {
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

        let csv = generate_csv(&translations, "en", &["en".to_string(), "id".to_string()]).unwrap();

        assert!(csv.contains("Source,Context,Key,en,id"));
        assert!(csv.contains("Buy"));
        assert!(csv.contains("Beli"));
    }

    #[test]
    fn test_generate_csv_with_context() {
        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: Some("Purchase button".to_string()),
            },
            Translation {
                key: "ui.button".to_string(),
                value: "Beli".to_string(),
                locale: "id".to_string(),
                context: Some("Purchase button".to_string()),
            },
        ];

        let csv = generate_csv(&translations, "en", &["en".to_string(), "id".to_string()]).unwrap();

        assert!(csv.contains("Purchase button"));
    }

    #[test]
    fn test_generate_csv_missing_locale() {
        let translations = vec![Translation {
            key: "ui.button".to_string(),
            value: "Buy".to_string(),
            locale: "en".to_string(),
            context: None,
        }];

        let csv = generate_csv(
            &translations,
            "en",
            &["en".to_string(), "id".to_string(), "es".to_string()],
        )
        .unwrap();
        assert!(csv.contains("Buy"));
    }

    #[test]
    fn test_generate_csv_sorted_keys() {
        let translations = vec![
            Translation {
                key: "z.key".to_string(),
                value: "Z".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "a.key".to_string(),
                value: "A".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "m.key".to_string(),
                value: "M".to_string(),
                locale: "en".to_string(),
                context: None,
            },
        ];

        let csv = generate_csv(&translations, "en", &["en".to_string()]).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines[1].contains("a.key"));
        assert!(lines[2].contains("m.key"));
        assert!(lines[3].contains("z.key"));
    }

    #[test]
    fn test_generate_csv_special_characters() {
        let translations = vec![Translation {
            key: "ui.message".to_string(),
            value: "Hello, \"World\"!\nNew line".to_string(),
            locale: "en".to_string(),
            context: None,
        }];

        let csv = generate_csv(&translations, "en", &["en".to_string()]).unwrap();
        assert!(csv.contains("\"Hello, \"\"World\"\"!\nNew line\""));
    }

    #[test]
    fn test_parse_csv() {
        let csv_content = r#"Source,Context,Key,en,id
"Buy","","ui.button","Buy","Beli"
"Sell","","ui.sell","Sell","Jual"
"#;

        let translations = parse_csv(csv_content).unwrap();

        assert_eq!(translations.len(), 4); // 2 keys × 2 locales
        assert!(translations
            .iter()
            .any(|t| t.key == "ui.button" && t.locale == "en" && t.value == "Buy"));
        assert!(translations
            .iter()
            .any(|t| t.key == "ui.button" && t.locale == "id" && t.value == "Beli"));
    }

    #[test]
    fn test_parse_csv_empty_file() {
        let csv_content = "";
        let result = parse_csv(csv_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_invalid_header() {
        let csv_content = "Source,Context\n";
        let result = parse_csv(csv_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_empty_cells() {
        let csv_content = r#"Source,Context,Key,en,id
"Buy","","ui.button","Buy",""
"#;

        let translations = parse_csv(csv_content).unwrap();
        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0].locale, "en");
    }

    #[test]
    fn test_parse_csv_with_context() {
        let csv_content = r#"Source,Context,Key,en
"Buy","Purchase button","ui.button","Buy"
"#;

        let translations = parse_csv(csv_content).unwrap();

        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0].context, Some("Purchase button".to_string()));
    }

    #[test]
    fn test_parse_csv_skip_empty_lines() {
        let csv_content = r#"Source,Context,Key,en

"Buy","","ui.button","Buy"

"#;

        let translations = parse_csv(csv_content).unwrap();

        assert_eq!(translations.len(), 1);
    }

    #[test]
    fn test_parse_csv_multiline_field() {
        let csv_content =
            "Source,Context,Key,en\n\"Line 1\nLine 2\",\"\",\"ui.message\",\"Line 1\nLine 2\"\n";

        let translations = parse_csv(csv_content).unwrap();

        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0].value, "Line 1\nLine 2");
    }

    #[test]
    fn test_roundtrip_csv() {
        let original_translations = vec![
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
        let csv = generate_csv(
            &original_translations,
            "en",
            &["en".to_string(), "id".to_string()],
        )
        .unwrap();
        let parsed_translations = parse_csv(&csv).unwrap();
        assert_eq!(parsed_translations.len(), original_translations.len());
        for original in &original_translations {
            assert!(parsed_translations.iter().any(|parsed| {
                parsed.key == original.key
                    && parsed.value == original.value
                    && parsed.locale == original.locale
            }));
        }
    }
}
