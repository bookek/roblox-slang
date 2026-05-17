use super::KeyTransform;
use crate::utils::flatten::unflatten_to_json;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
pub fn migrate_gettext(
    input_path: &Path,
    output_path: &Path,
    transform: KeyTransform,
) -> Result<()> {
    let content = fs::read_to_string(input_path)
        .context(format!("Failed to read {}", input_path.display()))?;
    let translations = parse_po_file(&content)?;
    let transformed: HashMap<String, String> = translations
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
fn parse_po_file(content: &str) -> Result<HashMap<String, String>> {
    let mut translations = HashMap::new();
    let mut current_msgid: Option<String> = None;
    let mut current_msgstr: Option<String> = None;
    let mut in_msgid = false;
    let mut in_msgstr = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("msgid ") {
            if let (Some(id), Some(str)) = (current_msgid.take(), current_msgstr.take()) {
                if !id.is_empty() && !str.is_empty() {
                    translations.insert(id, str);
                }
            }

            current_msgid = Some(extract_quoted_string(line));
            in_msgid = true;
            in_msgstr = false;
        } else if line.starts_with("msgstr ") {
            current_msgstr = Some(extract_quoted_string(line));
            in_msgid = false;
            in_msgstr = true;
        } else if line.starts_with('"') {
            let text = extract_quoted_string(line);
            if in_msgstr {
                if let Some(ref mut msgstr) = current_msgstr {
                    msgstr.push_str(&text);
                }
            } else if in_msgid {
                if let Some(ref mut msgid) = current_msgid {
                    msgid.push_str(&text);
                }
            }
        }
    }
    if let (Some(id), Some(str)) = (current_msgid, current_msgstr) {
        if !id.is_empty() && !str.is_empty() {
            translations.insert(id, str);
        }
    }

    Ok(translations)
}
fn extract_quoted_string(line: &str) -> String {
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if start < end {
                return line[start + 1..end].to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::TempDir;

    #[test]
    fn test_extract_quoted_string() {
        assert_eq!(extract_quoted_string("msgid \"hello\""), "hello");
        assert_eq!(extract_quoted_string("msgstr \"world\""), "world");
        assert_eq!(extract_quoted_string("\"text\""), "text");
    }

    #[test]
    fn test_parse_po_file_simple() {
        let content = r#"
msgid "ui.button.buy"
msgstr "Buy"

msgid "ui.button.sell"
msgstr "Sell"
"#;

        let result = parse_po_file(content).unwrap();
        assert_eq!(result.get("ui.button.buy"), Some(&"Buy".to_string()));
        assert_eq!(result.get("ui.button.sell"), Some(&"Sell".to_string()));
    }

    #[test]
    fn test_parse_po_file_with_comments() {
        let content = r#"
# This is a comment
msgid "ui.button"
msgstr "Button"

# Another comment
msgid "ui.label"
msgstr "Label"
"#;

        let result = parse_po_file(content).unwrap();
        assert_eq!(result.get("ui.button"), Some(&"Button".to_string()));
        assert_eq!(result.get("ui.label"), Some(&"Label".to_string()));
    }

    #[test]
    fn test_parse_po_file_multiline() {
        let content = r#"
msgid "ui.message"
msgstr "This is a long "
"message"
"#;

        let result = parse_po_file(content).unwrap();
        assert_eq!(
            result.get("ui.message"),
            Some(&"This is a long message".to_string())
        );
    }

    #[test]
    fn test_migrate_gettext() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.po");
        let output_path = temp_dir.path().join("output.json");
        let input_content = r#"
msgid "ui.button"
msgstr "Buy"
"#;
        fs::write(&input_path, input_content).unwrap();
        migrate_gettext(&input_path, &output_path, KeyTransform::None).unwrap();
        let output_content = fs::read_to_string(&output_path).unwrap();
        let output_json: Value = serde_json::from_str(&output_content).unwrap();

        assert_eq!(output_json["ui"]["button"], "Buy");
    }
}
