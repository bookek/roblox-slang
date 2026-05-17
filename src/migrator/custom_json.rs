use super::KeyTransform;
use crate::utils::flatten::unflatten_to_json;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
pub fn migrate_custom_json(
    input_path: &Path,
    output_path: &Path,
    transform: KeyTransform,
) -> Result<()> {
    let content = fs::read_to_string(input_path)
        .context(format!("Failed to read {}", input_path.display()))?;

    let json: Value = serde_json::from_str(&content).context("Failed to parse JSON")?;
    let flat_translations = extract_translations(&json)?;
    let transformed: HashMap<String, String> = flat_translations
        .into_iter()
        .map(|(k, v)| {
            let new_key = super::key_transform::transform_key(&k, transform);
            (new_key, v)
        })
        .collect();
    let nested = unflatten_to_json(&transformed);
    fs::create_dir_all(output_path.parent().unwrap())?;
    let output_json = serde_json::to_string_pretty(&nested)?;
    fs::write(output_path, output_json)?;

    Ok(())
}
fn extract_translations(json: &Value) -> Result<HashMap<String, String>> {
    let mut translations = HashMap::new();
    if let Some(trans_obj) = json.get("translations") {
        if let Some(locale_obj) = trans_obj.as_object() {
            if let Some((_, locale_data)) = locale_obj.iter().next() {
                extract_flat(locale_data, String::new(), &mut translations);
            }
        }
    } else if let Some(locales_obj) = json.get("locales") {
        if let Some(locale_obj) = locales_obj.as_object() {
            if let Some((_, locale_data)) = locale_obj.iter().next() {
                extract_flat(locale_data, String::new(), &mut translations);
            }
        }
    } else if let Some(strings_obj) = json.get("strings") {
        extract_flat(strings_obj, String::new(), &mut translations);
    } else {
        extract_flat(json, String::new(), &mut translations);
    }

    Ok(translations)
}
fn extract_flat(value: &Value, prefix: String, result: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                extract_flat(val, new_prefix, result);
            }
        }
        Value::String(s) => {
            result.insert(prefix, s.clone());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::TempDir;

    #[test]
    fn test_extract_translations_pattern1() {
        let json = serde_json::json!({
            "translations": {
                "en": {
                    "ui": {
                        "button": "Buy"
                    }
                }
            }
        });

        let result = extract_translations(&json).unwrap();
        assert_eq!(result.get("ui.button"), Some(&"Buy".to_string()));
    }

    #[test]
    fn test_extract_translations_pattern2() {
        let json = serde_json::json!({
            "locales": {
                "en": {
                    "ui": {
                        "button": "Buy"
                    }
                }
            }
        });

        let result = extract_translations(&json).unwrap();
        assert_eq!(result.get("ui.button"), Some(&"Buy".to_string()));
    }

    #[test]
    fn test_extract_translations_pattern3() {
        let json = serde_json::json!({
            "strings": {
                "ui": {
                    "button": "Buy"
                }
            }
        });

        let result = extract_translations(&json).unwrap();
        assert_eq!(result.get("ui.button"), Some(&"Buy".to_string()));
    }

    #[test]
    fn test_extract_translations_direct() {
        let json = serde_json::json!({
            "ui": {
                "button": "Buy"
            }
        });

        let result = extract_translations(&json).unwrap();
        assert_eq!(result.get("ui.button"), Some(&"Buy".to_string()));
    }

    #[test]
    fn test_migrate_custom_json() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.json");
        let input_json = serde_json::json!({
            "translations": {
                "en": {
                    "ui_button": "Buy"
                }
            }
        });
        fs::write(&input_path, serde_json::to_string(&input_json).unwrap()).unwrap();
        migrate_custom_json(&input_path, &output_path, KeyTransform::SnakeToCamel).unwrap();
        let output_content = fs::read_to_string(&output_path).unwrap();
        let output_json: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(output_json["uiButton"], "Buy");
    }
}
