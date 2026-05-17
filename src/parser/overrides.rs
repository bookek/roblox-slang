use super::types::*;
use anyhow::{Context, Result};
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;
pub fn parse_overrides(path: &Path) -> Result<Vec<Translation>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read override file: {}", path.display()))?;

    let yaml: Value = serde_yaml::from_str(&content)
        .context(format!("Failed to parse override YAML: {}", path.display()))?;

    let mut translations = Vec::new();
    if let Value::Mapping(locales) = yaml {
        for (locale_key, locale_value) in locales {
            if let Value::String(locale) = locale_key {
                if let Value::Mapping(keys) = locale_value {
                    for (key, value) in keys {
                        if let (Value::String(k), Value::String(v)) = (key, value) {
                            translations.push(Translation {
                                key: k.clone(),
                                value: v.clone(),
                                locale: locale.clone(),
                                context: None,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(translations)
}
pub fn merge_translations(main: Vec<Translation>, overrides: Vec<Translation>) -> Vec<Translation> {
    if overrides.is_empty() {
        return main;
    }
    let mut override_map: HashMap<(String, String), Translation> = HashMap::new();
    for override_trans in overrides {
        let key = (override_trans.locale.clone(), override_trans.key.clone());
        override_map.insert(key, override_trans);
    }
    let mut result = Vec::new();
    let mut used_overrides: HashMap<(String, String), bool> = HashMap::new();

    for main_trans in main {
        let key = (main_trans.locale.clone(), main_trans.key.clone());

        if let Some(override_trans) = override_map.get(&key) {
            result.push(override_trans.clone());
            used_overrides.insert(key, true);
        } else {
            result.push(main_trans);
        }
    }
    for (key, override_trans) in override_map {
        if !used_overrides.contains_key(&key) {
            result.push(override_trans);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_overrides() {
        let yaml_content = r#"
en:
  ui.buttons.buy: "Purchase Now!"
  ui.messages.greeting: "Hey, {name}!"
id:
  ui.buttons.buy: "Beli Sekarang!"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let overrides = parse_overrides(temp_file.path()).unwrap();

        assert_eq!(overrides.len(), 3);
        assert!(overrides
            .iter()
            .any(|t| t.locale == "en" && t.key == "ui.buttons.buy" && t.value == "Purchase Now!"));
        assert!(overrides.iter().any(|t| t.locale == "en"
            && t.key == "ui.messages.greeting"
            && t.value == "Hey, {name}!"));
        assert!(overrides
            .iter()
            .any(|t| t.locale == "id" && t.key == "ui.buttons.buy" && t.value == "Beli Sekarang!"));
    }

    #[test]
    fn test_merge_translations() {
        let main = vec![
            Translation {
                key: "ui.buttons.buy".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.buttons.sell".to_string(),
                value: "Sell".to_string(),
                locale: "en".to_string(),
                context: None,
            },
        ];

        let overrides = vec![Translation {
            key: "ui.buttons.buy".to_string(),
            value: "Purchase Now!".to_string(),
            locale: "en".to_string(),
            context: None,
        }];

        let merged = merge_translations(main, overrides);

        assert_eq!(merged.len(), 2);
        let buy_trans = merged.iter().find(|t| t.key == "ui.buttons.buy").unwrap();
        assert_eq!(buy_trans.value, "Purchase Now!");
        let sell_trans = merged.iter().find(|t| t.key == "ui.buttons.sell").unwrap();
        assert_eq!(sell_trans.value, "Sell");
    }

    #[test]
    fn test_parse_overrides_nonexistent_file() {
        let result = parse_overrides(Path::new("nonexistent.yaml")).unwrap();
        assert_eq!(result.len(), 0);
    }
}
