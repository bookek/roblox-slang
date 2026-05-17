use anyhow::Result;
use std::fs;
use std::path::Path;
pub fn detect_unused_keys(translation_keys: &[String], source_dir: &Path) -> Result<Vec<String>> {
    if !source_dir.exists() {
        return Ok(Vec::new());
    }
    let lua_files = find_lua_files(source_dir)?;
    let mut all_content = String::new();
    for file in lua_files {
        if let Ok(content) = fs::read_to_string(&file) {
            all_content.push_str(&content);
            all_content.push('\n');
        }
    }
    let mut unused = Vec::new();
    for key in translation_keys {
        let key_variations = [
            format!("\"{}\"", key), // String literal
            format!("'{}'", key),   // Single quote string
            key.to_string(),        // Direct key
        ];

        let found = key_variations
            .iter()
            .any(|variation| all_content.contains(variation));

        if !found {
            unused.push(key.clone());
        }
    }

    Ok(unused)
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
        let unused = detect_unused_keys(&keys, Path::new("/nonexistent/path")).unwrap();
        assert_eq!(unused.len(), 0);
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
}
