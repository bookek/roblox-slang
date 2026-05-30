use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::generator::luau::{sanitize_luau_identifier, sanitize_segment};

pub fn detect_unused_keys(translation_keys: &[String], source_dir: &Path) -> Result<Vec<String>> {
    if !source_dir.exists() {
        anyhow::bail!("Source directory not found: {}", source_dir.display());
    }
    let lua_files = find_lua_files(source_dir)?;
    let mut all_content = String::new();
    for file in lua_files {
        if let Ok(content) = fs::read_to_string(&file) {
            all_content.push_str(&content);
            all_content.push('\n');
        }
    }
    let content = strip_lua_comments(&all_content);
    let string_literals = extract_string_literals(&content);
    let mut unused = Vec::new();
    for key in translation_keys {
        let found = string_literals.iter().any(|literal| literal == key)
            || contains_namespace_usage(&content, key);

        if !found {
            unused.push(key.clone());
        }
    }

    Ok(unused)
}

fn contains_namespace_usage(content: &str, key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return false;
    }

    let flat_method = sanitize_luau_identifier(key);
    if contains_access_pattern(content, &format!(":{}", flat_method))
        || contains_access_pattern(content, &format!(".{}", flat_method))
    {
        return true;
    }

    if parts.len() < 2 {
        return false;
    }

    let namespace = parts[..parts.len() - 1]
        .iter()
        .map(|part| sanitize_segment(part))
        .collect::<Vec<_>>()
        .join(".");
    let method = sanitize_segment(parts[parts.len() - 1]);
    contains_access_pattern(content, &format!(".{}.{}", namespace, method))
        || contains_access_pattern(content, &format!(".{}:{}", namespace, method))
}

fn contains_access_pattern(content: &str, pattern: &str) -> bool {
    let mut search_start = 0;
    while let Some(index) = content[search_start..].find(pattern) {
        let start = search_start + index;
        let end = start + pattern.len();
        let after = content[end..].chars().next();

        if !is_identifier_char(after) {
            return true;
        }

        search_start = end;
    }

    false
}

fn is_identifier_char(ch: Option<char>) -> bool {
    ch.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn strip_lua_comments(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string: Option<char> = None;

    while let Some(ch) = chars.next() {
        if let Some(quote) = in_string {
            output.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    output.push(next);
                }
            } else if ch == quote {
                in_string = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            output.push(ch);
            continue;
        }

        if ch == '-' && chars.peek() == Some(&'-') {
            chars.next();
            if chars.peek() == Some(&'[') {
                let mut lookahead = chars.clone();
                lookahead.next();
                if lookahead.peek() == Some(&'[') {
                    chars.next();
                    chars.next();
                    while let Some(block_ch) = chars.next() {
                        if block_ch == ']' && chars.peek() == Some(&']') {
                            chars.next();
                            break;
                        }
                        if block_ch == '\n' {
                            output.push('\n');
                        }
                    }
                    continue;
                }
            }

            for line_ch in chars.by_ref() {
                if line_ch == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }

        output.push(ch);
    }

    output
}

fn extract_string_literals(content: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '"' && ch != '\'' {
            continue;
        }

        let quote = ch;
        let mut literal = String::new();
        while let Some(value_ch) = chars.next() {
            if value_ch == '\\' {
                if let Some(escaped) = chars.next() {
                    literal.push(escaped);
                }
                continue;
            }

            if value_ch == quote {
                break;
            }

            literal.push(value_ch);
        }

        literals.push(literal);
    }

    literals
}

fn find_lua_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut lua_files = Vec::new();

    if dir.is_file() {
        if let Some(ext) = dir.extension() {
            if ext == "lua" || ext == "luau" {
                lua_files.push(dir.to_path_buf());
            }
        }
        return Ok(lua_files);
    }

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                lua_files.extend(find_lua_files(&path)?);
            } else if let Some(ext) = path.extension() {
                if ext == "lua" || ext == "luau" {
                    lua_files.push(path);
                }
            }
        }
    }

    Ok(lua_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_detect_unused_keys() {
        let temp_dir = TempDir::new().unwrap();
        let lua_file = temp_dir.path().join("test.lua");
        let mut file = fs::File::create(&lua_file).unwrap();
        writeln!(file, "local text = t.ui.buttons.buy()").unwrap();
        writeln!(file, "print(\"ui.labels.welcome\")").unwrap();

        let keys = vec![
            "ui.buttons.buy".to_string(),
            "ui.labels.welcome".to_string(),
            "ui.unused.key".to_string(),
        ];

        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0], "ui.unused.key");
    }

    #[test]
    fn test_find_lua_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::File::create(temp_dir.path().join("test1.lua")).unwrap();
        fs::File::create(temp_dir.path().join("test2.luau")).unwrap();
        fs::File::create(temp_dir.path().join("test.txt")).unwrap();

        let lua_files = find_lua_files(temp_dir.path()).unwrap();

        assert_eq!(lua_files.len(), 2);
    }

    #[test]
    fn test_find_lua_files_nested() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        fs::File::create(temp_dir.path().join("test1.lua")).unwrap();
        fs::File::create(subdir.join("test2.lua")).unwrap();
        fs::File::create(subdir.join("test3.luau")).unwrap();

        let lua_files = find_lua_files(temp_dir.path()).unwrap();

        assert_eq!(lua_files.len(), 3);
    }

    #[test]
    fn test_detect_unused_keys_single_quotes() {
        let temp_dir = TempDir::new().unwrap();

        let lua_file = temp_dir.path().join("test.lua");
        let mut file = fs::File::create(&lua_file).unwrap();
        writeln!(file, "local text = 'ui.button'").unwrap();

        let keys = vec!["ui.button".to_string(), "ui.unused".to_string()];

        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0], "ui.unused");
    }

    #[test]
    fn test_detect_unused_keys_all_used() {
        let temp_dir = TempDir::new().unwrap();

        let lua_file = temp_dir.path().join("test.lua");
        let mut file = fs::File::create(&lua_file).unwrap();
        writeln!(file, "local text1 = t.ui.button()").unwrap();
        writeln!(file, "local text2 = \"ui.label\"").unwrap();

        let keys = vec!["ui.button".to_string(), "ui.label".to_string()];

        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert_eq!(unused.len(), 0);
    }

    #[test]
    fn test_detect_unused_keys_nonexistent_dir() {
        let keys = vec!["ui.button".to_string()];
        let result = detect_unused_keys(&keys, Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_unused_keys_empty_keys() {
        let temp_dir = TempDir::new().unwrap();

        let lua_file = temp_dir.path().join("test.lua");
        fs::File::create(&lua_file).unwrap();

        let keys = vec![];
        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert_eq!(unused.len(), 0);
    }

    #[test]
    fn test_find_lua_files_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let lua_file = temp_dir.path().join("test.lua");
        fs::File::create(&lua_file).unwrap();
        let lua_files = find_lua_files(&lua_file).unwrap();

        assert_eq!(lua_files.len(), 1);
        assert_eq!(lua_files[0], lua_file);
    }

    #[test]
    fn test_find_lua_files_non_lua_file() {
        let temp_dir = TempDir::new().unwrap();
        let txt_file = temp_dir.path().join("test.txt");
        fs::File::create(&txt_file).unwrap();
        let lua_files = find_lua_files(&txt_file).unwrap();

        assert_eq!(lua_files.len(), 0);
    }

    #[test]
    fn test_detect_unused_keys_ignores_comments() {
        let temp_dir = TempDir::new().unwrap();
        let lua_file = temp_dir.path().join("test.lua");
        let mut file = fs::File::create(&lua_file).unwrap();
        writeln!(file, "-- ui.comment.only").unwrap();
        writeln!(file, "local text = \"ui.used\"").unwrap();

        let keys = vec!["ui.comment.only".to_string(), "ui.used".to_string()];
        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert_eq!(unused, vec!["ui.comment.only"]);
    }

    #[test]
    fn test_detect_unused_keys_does_not_match_substrings() {
        let temp_dir = TempDir::new().unwrap();
        let lua_file = temp_dir.path().join("test.lua");
        let mut file = fs::File::create(&lua_file).unwrap();
        writeln!(file, "local text = \"ui.button.extra\"").unwrap();

        let keys = vec!["ui.button".to_string()];
        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert_eq!(unused, vec!["ui.button"]);
    }

    #[test]
    fn test_detect_unused_keys_matches_sanitized_namespace() {
        let temp_dir = TempDir::new().unwrap();
        let lua_file = temp_dir.path().join("test.lua");
        let mut file = fs::File::create(&lua_file).unwrap();
        writeln!(file, "local text = t.ui.buttons:buy_now()").unwrap();

        let keys = vec!["ui.buttons.buy-now".to_string()];
        let unused = detect_unused_keys(&keys, temp_dir.path()).unwrap();

        assert!(unused.is_empty());
    }
}
