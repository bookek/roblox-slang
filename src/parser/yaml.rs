use super::types::*;
use crate::utils::flatten;
use crate::utils::validation;
use anyhow::{bail, Result};
use serde_yaml::Value;
use std::path::Path;
pub fn parse_yaml_file(path: &Path, locale: &str) -> Result<Vec<Translation>> {
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
             ui:\n\
               button: Click me\n\
               label: Welcome",
            path.display()
        );
    }
    let yaml: Value = serde_yaml::from_str(&content).map_err(|e| {
        let location = e.location();
        let (line, column) = if let Some(loc) = location {
            (loc.line(), loc.column())
        } else {
            (0, 0)
        };
        let lines: Vec<&str> = content.lines().collect();
        let context_line = if line > 0 && line <= lines.len() {
            lines[line - 1]
        } else {
            ""
        };

        anyhow::anyhow!(
            "Failed to parse YAML in: {}\n\
             Error at line {}, column {}: {}\n\
             \n\
             Problematic line:\n\
             {}\n\
             {}^\n\
             \n\
             Common YAML errors:\n\
             - Incorrect indentation (use spaces, not tabs)\n\
             - Missing colon after key\n\
             - Unquoted special characters\n\
             - Inconsistent indentation levels\n\
             \n\
             Hint: YAML is indentation-sensitive. Use 2 or 4 spaces consistently.",
            path.display(),
            line,
            column,
            e,
            context_line,
            " ".repeat(column.saturating_sub(1))
        )
    })?;
    let json = yaml_to_json(&yaml)?;
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
fn yaml_to_json(yaml: &Value) -> Result<serde_json::Value> {
    match yaml {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(serde_json::Value::Number(i.into()))
            } else if let Some(f) = n.as_f64() {
                Ok(serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null))
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Sequence(seq) => {
            let arr: Result<Vec<_>> = seq.iter().map(yaml_to_json).collect();
            Ok(serde_json::Value::Array(arr?))
        }
        Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let Value::String(key) = k {
                    obj.insert(key.clone(), yaml_to_json(v)?);
                }
            }
            Ok(serde_json::Value::Object(obj))
        }
        Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_yaml_simple() {
        let yaml_content = r#"
key: value
another: test
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let translations = parse_yaml_file(temp_file.path(), "en").unwrap();

        assert_eq!(translations.len(), 2);
        assert!(translations
            .iter()
            .any(|t| t.key == "key" && t.value == "value"));
        assert!(translations
            .iter()
            .any(|t| t.key == "another" && t.value == "test"));
    }

    #[test]
    fn test_parse_yaml_nested() {
        let yaml_content = r#"
ui:
  buttons:
    buy: Buy
    sell: Sell
  labels:
    welcome: Welcome!
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let translations = parse_yaml_file(temp_file.path(), "en").unwrap();

        assert_eq!(translations.len(), 3);
        assert!(translations
            .iter()
            .any(|t| t.key == "ui.buttons.buy" && t.value == "Buy"));
        assert!(translations
            .iter()
            .any(|t| t.key == "ui.buttons.sell" && t.value == "Sell"));
        assert!(translations
            .iter()
            .any(|t| t.key == "ui.labels.welcome" && t.value == "Welcome!"));
    }

    #[test]
    fn test_yaml_to_json_conversion() {
        let yaml: Value = serde_yaml::from_str(
            r#"
test: value
number: 42
nested:
  key: data
"#,
        )
        .unwrap();

        let json = yaml_to_json(&yaml).unwrap();

        assert_eq!(json["test"], "value");
        assert_eq!(json["number"], 42);
        assert_eq!(json["nested"]["key"], "data");
    }
}
