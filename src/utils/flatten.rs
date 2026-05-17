use std::collections::HashMap;
pub fn flatten_json(value: &serde_json::Value, prefix: String) -> HashMap<String, String> {
    let mut result = HashMap::new();

    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                result.extend(flatten_json(val, new_prefix));
            }
        }
        serde_json::Value::String(s) => {
            result.insert(prefix, s.clone());
        }
        _ => {
            log::warn!("Skipping non-string value at key: {}", prefix);
        }
    }

    result
}
pub fn unflatten_to_json(flat: &HashMap<String, String>) -> serde_json::Value {
    let mut root = serde_json::Map::new();

    for (key, value) in flat {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &mut root;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current.insert(part.to_string(), serde_json::Value::String(value.clone()));
            } else {
                let part_string = part.to_string();
                if !current.contains_key(&part_string) {
                    current.insert(
                        part_string.clone(),
                        serde_json::Value::Object(serde_json::Map::new()),
                    );
                }
                current = current
                    .get_mut(&part_string)
                    .and_then(|v| v.as_object_mut())
                    .expect("Expected object");
            }
        }
    }

    serde_json::Value::Object(root)
}
pub fn unflatten_translations(translations: &[crate::parser::Translation]) -> serde_json::Value {
    let mut flat = HashMap::new();

    for translation in translations {
        flat.insert(translation.key.clone(), translation.value.clone());
    }

    unflatten_to_json(&flat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_flatten_simple() {
        let json = json!({"key": "value"});
        let result = flatten_json(&json, String::new());
        assert_eq!(result.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_flatten_nested() {
        let json = json!({
            "ui": {
                "button": "Buy"
            }
        });
        let result = flatten_json(&json, String::new());
        assert_eq!(result.get("ui.button"), Some(&"Buy".to_string()));
    }

    #[test]
    fn test_flatten_deep_nested() {
        let json = json!({
            "ui": {
                "buttons": {
                    "buy": "Buy",
                    "sell": "Sell"
                }
            }
        });
        let result = flatten_json(&json, String::new());
        assert_eq!(result.get("ui.buttons.buy"), Some(&"Buy".to_string()));
        assert_eq!(result.get("ui.buttons.sell"), Some(&"Sell".to_string()));
    }

    #[test]
    fn test_unflatten() {
        let mut flat = HashMap::new();
        flat.insert("ui.button".to_string(), "Buy".to_string());

        let result = unflatten_to_json(&flat);
        assert_eq!(result["ui"]["button"], "Buy");
    }

    #[test]
    fn test_unflatten_deep() {
        let mut flat = HashMap::new();
        flat.insert("ui.buttons.buy".to_string(), "Buy".to_string());
        flat.insert("ui.buttons.sell".to_string(), "Sell".to_string());

        let result = unflatten_to_json(&flat);
        assert_eq!(result["ui"]["buttons"]["buy"], "Buy");
        assert_eq!(result["ui"]["buttons"]["sell"], "Sell");
    }

    #[test]
    fn test_roundtrip() {
        let original = json!({
            "ui": {
                "buttons": {
                    "buy": "Buy",
                    "sell": "Sell"
                }
            }
        });

        let flattened = flatten_json(&original, String::new());
        let unflattened = unflatten_to_json(&flattened);

        assert_eq!(unflattened, original);
    }

    #[test]
    fn test_flatten_with_prefix() {
        let json = json!({"key": "value"});
        let result = flatten_json(&json, "prefix".to_string());
        assert_eq!(result.get("prefix.key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_flatten_empty_object() {
        let json = json!({});
        let result = flatten_json(&json, String::new());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_flatten_non_string_values() {
        let json = json!({
            "number": 42,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3]
        });
        let result = flatten_json(&json, String::new());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_unflatten_single_key() {
        let mut flat = HashMap::new();
        flat.insert("key".to_string(), "value".to_string());

        let result = unflatten_to_json(&flat);
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn test_unflatten_empty() {
        let flat = HashMap::new();
        let result = unflatten_to_json(&flat);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_unflatten_multiple_roots() {
        let mut flat = HashMap::new();
        flat.insert("ui.button".to_string(), "Buy".to_string());
        flat.insert("shop.item".to_string(), "Item".to_string());

        let result = unflatten_to_json(&flat);
        assert_eq!(result["ui"]["button"], "Buy");
        assert_eq!(result["shop"]["item"], "Item");
    }

    #[test]
    fn test_unflatten_translations() {
        use crate::parser::Translation;

        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                value: "Buy".to_string(),
                locale: "en".to_string(),
                context: None,
            },
            Translation {
                key: "ui.label".to_string(),
                value: "Label".to_string(),
                locale: "en".to_string(),
                context: None,
            },
        ];

        let result = unflatten_translations(&translations);
        assert_eq!(result["ui"]["button"], "Buy");
        assert_eq!(result["ui"]["label"], "Label");
    }
}
