use crate::parser::Translation;
use anyhow::Result;
use std::collections::HashMap;
pub fn generate_csv(
    translations: &[Translation],
    base_locale: &str,
    locales: &[String],
) -> Result<String> {
    let mut csv = String::new();
    csv.push_str("Source,Context,Key");
    for locale in locales {
        csv.push(',');
        csv.push_str(locale);
    }
    csv.push('\n');
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
        let source = locale_values
            .get(base_locale)
            .map(|s| escape_csv_value(s))
            .unwrap_or_else(|| String::from("\"\""));

        csv.push_str(&source);
        csv.push(',');
        let context_str = context
            .as_ref()
            .map(|c| escape_csv_value(c))
            .unwrap_or_else(|| String::from("\"\""));
        csv.push_str(&context_str);
        csv.push(',');
        csv.push_str(&escape_csv_value(&key));
        for locale in locales {
            csv.push(',');
            let value = locale_values
                .get(locale)
                .map(|s| escape_csv_value(s))
                .unwrap_or_else(|| String::from("\"\""));
            csv.push_str(&value);
        }

        csv.push('\n');
    }

    Ok(csv)
}
fn escape_csv_value(value: &str) -> String {
    let needs_escape =
        value.contains('"') || value.contains(',') || value.contains('\n') || value.contains('\r');

    if needs_escape || !value.is_empty() {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        String::from("\"\"")
    }
}
pub fn parse_csv(content: &str) -> Result<Vec<Translation>> {
    let mut translations = Vec::new();
    let mut lines = content.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("CSV file is empty"))?;
    let headers = parse_csv_line(header);

    if headers.len() < 3 {
        anyhow::bail!("Invalid CSV header: expected at least Source,Context,Key columns");
    }
    let locales: Vec<String> = headers[3..].to_vec();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let values = parse_csv_line(line);

        if values.len() < 3 {
            log::warn!("Skipping invalid CSV row: {}", line);
            continue;
        }

        let key = values[2].clone();
        let context = if values[1].is_empty() {
            None
        } else {
            Some(values[1].clone())
        };
        for (i, locale) in locales.iter().enumerate() {
            let value_index = 3 + i;
            if value_index < values.len() {
                let value = &values[value_index];
                if !value.is_empty() {
                    translations.push(Translation {
                        key: key.clone(),
                        value: value.clone(),
                        locale: locale.clone(),
                        context: context.clone(),
                    });
                }
            }
        }
    }

    Ok(translations)
}
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' => {
                if in_quotes {
                    current.push(',');
                } else {
                    values.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }
    values.push(current.trim().to_string());

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_value() {
        assert_eq!(escape_csv_value("Hello"), "\"Hello\"");
        assert_eq!(escape_csv_value("Hello, World"), "\"Hello, World\"");
        assert_eq!(escape_csv_value("Say \"Hi\""), "\"Say \"\"Hi\"\"\"");
    }

    #[test]
    fn test_escape_csv_value_empty() {
        assert_eq!(escape_csv_value(""), "\"\"");
    }

    #[test]
    fn test_escape_csv_value_newline() {
        assert_eq!(escape_csv_value("Line1\nLine2"), "\"Line1\nLine2\"");
    }

    #[test]
    fn test_escape_csv_value_carriage_return() {
        assert_eq!(escape_csv_value("Line1\rLine2"), "\"Line1\rLine2\"");
    }

    #[test]
    fn test_escape_csv_value_multiple_quotes() {
        assert_eq!(
            escape_csv_value("\"Quote1\" and \"Quote2\""),
            "\"\"\"Quote1\"\" and \"\"Quote2\"\"\""
        );
    }

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
        assert!(csv.contains("\"Buy\""));
        assert!(csv.contains("\"Beli\""));
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

        assert!(csv.contains("\"Purchase button\""));
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
        assert!(csv.contains("\"Buy\",\"\",\"\""));
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
    fn test_parse_csv_line() {
        let line = "\"Hello\",\"World\",\"Test\"";
        let values = parse_csv_line(line);
        assert_eq!(values, vec!["Hello", "World", "Test"]);
    }

    #[test]
    fn test_parse_csv_line_with_quotes() {
        let line = "\"Say \"\"Hi\"\"\",\"World\"";
        let values = parse_csv_line(line);
        assert_eq!(values[0], "Say \"Hi\"");
    }

    #[test]
    fn test_parse_csv_line_with_commas() {
        let line = "\"Hello, World\",\"Test\"";
        let values = parse_csv_line(line);
        assert_eq!(values, vec!["Hello, World", "Test"]);
    }

    #[test]
    fn test_parse_csv_line_unquoted() {
        let line = "Hello,World,Test";
        let values = parse_csv_line(line);
        assert_eq!(values, vec!["Hello", "World", "Test"]);
    }

    #[test]
    fn test_parse_csv_line_mixed() {
        let line = "\"Quoted\",Unquoted,\"Mixed\"";
        let values = parse_csv_line(line);
        assert_eq!(values, vec!["Quoted", "Unquoted", "Mixed"]);
    }

    #[test]
    fn test_parse_csv_line_empty_values() {
        let line = "\"\",\"\",\"\"";
        let values = parse_csv_line(line);
        assert_eq!(values, vec!["", "", ""]);
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
