#![allow(dead_code)]

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
pub fn create_test_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join("translations")).unwrap();
    fs::create_dir(temp.path().join("output")).unwrap();
    let config = r#"base_locale: en
supported_locales:
  - en
  - id
input_directory: translations
output_directory: output
"#;
    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();

    temp
}
pub fn create_test_project_with_translations() -> TempDir {
    let temp = create_test_project();
    let en_json = r#"{
  "ui": {
    "buttons": {
      "buy": "Buy",
      "sell": "Sell"
    },
    "labels": {
      "welcome": "Welcome"
    }
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), en_json).unwrap();
    let id_json = r#"{
  "ui": {
    "buttons": {
      "buy": "Beli",
      "sell": "Jual"
    },
    "labels": {
      "welcome": "Selamat datang"
    }
  }
}"#;
    fs::write(temp.path().join("translations/id.json"), id_json).unwrap();

    temp
}
#[allow(dead_code)]
pub fn create_test_project_with_config(config: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    fs::create_dir(temp.path().join("translations")).unwrap();
    fs::create_dir(temp.path().join("output")).unwrap();
    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();

    temp
}
#[allow(dead_code)]
pub fn generate_translations(count: usize) -> serde_json::Map<String, serde_json::Value> {
    let mut translations = serde_json::Map::new();

    for i in 0..count {
        let key = format!("key_{}", i);
        let value = format!("Value {}", i);
        translations.insert(key, serde_json::Value::String(value));
    }

    translations
}
#[allow(dead_code)]
pub fn generate_nested_translations(depth: usize, breadth: usize) -> serde_json::Value {
    fn build_nested(current_depth: usize, max_depth: usize, breadth: usize) -> serde_json::Value {
        if current_depth >= max_depth {
            return serde_json::Value::String(format!("Value at depth {}", current_depth));
        }

        let mut map = serde_json::Map::new();
        for i in 0..breadth {
            let key = format!("level_{}_{}", current_depth, i);
            map.insert(key, build_nested(current_depth + 1, max_depth, breadth));
        }

        serde_json::Value::Object(map)
    }

    build_nested(0, depth, breadth)
}
pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "File not found: {}", path.display());
}
#[allow(dead_code)]
pub fn assert_file_not_exists(path: &Path) {
    assert!(!path.exists(), "File must not exist: {}", path.display());
}
pub fn assert_file_contains(path: &Path, content: &str) {
    let file_content = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));

    assert!(
        file_content.contains(content),
        "File {} does not contain '{}'",
        path.display(),
        content
    );
}
#[allow(dead_code)]
pub fn assert_file_not_contains(path: &Path, content: &str) {
    let file_content = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));

    assert!(
        !file_content.contains(content),
        "File {} must not contain '{}'",
        path.display(),
        content
    );
}
#[allow(dead_code)]
pub fn assert_dir_exists(path: &Path) {
    assert!(
        path.exists() && path.is_dir(),
        "Directory not found: {}",
        path.display()
    );
}
#[allow(dead_code)]
pub fn run_cli(args: &[&str]) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
#[allow(dead_code)]
pub fn get_file_size(path: &Path) -> u64 {
    fs::metadata(path)
        .unwrap_or_else(|_| panic!("Failed to get metadata for: {}", path.display()))
        .len()
}
#[allow(dead_code)]
pub fn count_lines(path: &Path) -> usize {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));

    content.lines().count()
}
#[allow(dead_code)]
pub fn create_test_csv(path: &Path, locales: &[&str], keys: &[&str]) {
    let mut csv = String::from("Source,Context,Key");
    for locale in locales {
        csv.push(',');
        csv.push_str(locale);
    }
    csv.push('\n');

    for key in keys {
        csv.push_str(&format!("\"{}\",\"\",\"{}\"", key, key));
        for locale in locales {
            csv.push_str(&format!(",\"{} in {}\"", key, locale));
        }
        csv.push('\n');
    }

    fs::write(path, csv).unwrap();
}
#[allow(dead_code)]
pub fn wait_for_file(path: &Path, timeout_secs: u64) -> bool {
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    false
}
#[allow(dead_code)]
pub fn assert_json_files_equal(path1: &Path, path2: &Path) {
    let content1 = fs::read_to_string(path1)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path1.display()));
    let content2 = fs::read_to_string(path2)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path2.display()));

    let json1: serde_json::Value = serde_json::from_str(&content1)
        .unwrap_or_else(|_| panic!("Failed to parse JSON: {}", path1.display()));
    let json2: serde_json::Value = serde_json::from_str(&content2)
        .unwrap_or_else(|_| panic!("Failed to parse JSON: {}", path2.display()));

    assert_eq!(json1, json2, "JSON files are not equal");
}
