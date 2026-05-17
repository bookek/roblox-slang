use super::types::*;
use crate::utils::flatten;
use crate::utils::validation;
use anyhow::{bail, Result};
use serde_json::Value;
use std::path::Path;
pub fn parse_json_file(path: &Path, locale: &str) -> Result<Vec<Translation>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read translation file: {}\n\
             Error: {}\n\
             \n\
             Hint: Make sure the file exists and you have read permissions.",
            path.display(),
            e
        )
    })?;
    if content.trim().is_empty() {
        bail!(
            "Translation file is empty: {}\n\
             \n\
             Hint: Add at least one translation key-value pair.\n\
             Example:\n\
             {{\n\
               \"ui\": {{\n\
                 \"button\": \"Click me\"\n\
               }}\n\
             }}",
            path.display()
        );
    }
    let json: Value = serde_json::from_str(&content).map_err(|e| {
        let line = e.line();
        let column = e.column();
        let lines: Vec<&str> = content.lines().collect();
        let context_line = if line > 0 && line <= lines.len() {
            lines[line - 1]
        } else {
            ""
        };

        anyhow::anyhow!(
            "Failed to parse JSON in: {}\n\
             Error at line {}, column {}: {}\n\
             \n\
             Problematic line:\n\
             {}\n\
             {}^\n\
             \n\
             Common JSON errors:\n\
             - Missing or extra commas\n\
             - Missing closing brackets or braces\n\
             - Unquoted keys or values\n\
             - Trailing commas (not allowed in strict JSON)\n\
             \n\
             Hint: Use a JSON validator or linter to check your file.",
            path.display(),
            line,
            column,
            e,
            context_line,
            " ".repeat(column.saturating_sub(1))
        )
    })?;

    let flattened = flatten::flatten_json(&json, String::new());
    if flattened.is_empty() {
        log::warn!("No translations found in: {}", path.display());
        return Ok(Vec::new());
    }

    let translations = flattened
        .into_iter()
        .map(|(key, value)| {
            validation::validate_translation_key(&key).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid translation key in: {}\n\
                     {}\n\
                     \n\
                     Hint: Use dot notation for translation keys (e.g., 'ui.button.buy')",
                    path.display(),
                    e
                )
            })?;

            Ok(Translation {
                key,
                value,
                locale: locale.to_string(),
                context: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(translations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_file_structure() {
        let json = json!({
            "ui": {
                "button": "Buy"
            }
        });
        let flattened = flatten::flatten_json(&json, String::new());
        assert_eq!(flattened.get("ui.button"), Some(&"Buy".to_string()));
    }

    #[test]
    fn test_flatten_simple() {
        let json = json!({"key": "value"});
        let result = flatten::flatten_json(&json, String::new());
        assert_eq!(result.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_flatten_nested() {
        let json = json!({
            "ui": {
                "button": "Buy"
            }
        });
        let result = flatten::flatten_json(&json, String::new());
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
        let result = flatten::flatten_json(&json, String::new());
        assert_eq!(result.get("ui.buttons.buy"), Some(&"Buy".to_string()));
        assert_eq!(result.get("ui.buttons.sell"), Some(&"Sell".to_string()));
    }
}
