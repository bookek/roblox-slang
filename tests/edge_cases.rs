#![allow(deprecated)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::doc_lazy_continuation)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
#[test]
fn test_empty_json_file() {
    let temp = common::create_test_project();
    fs::write(temp.path().join("translations/en.json"), "{}").unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_empty_yaml_file() {
    let temp = common::create_test_project();
    fs::write(temp.path().join("translations/en.yaml"), "---").unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_completely_empty_file() {
    let temp = common::create_test_project();
    fs::write(temp.path().join("translations/en.json"), "").unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse"));
}
#[test]
fn test_malformed_json() {
    let temp = common::create_test_project();
    fs::write(temp.path().join("translations/en.json"), "{ invalid json }").unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse"));
}
#[test]
fn test_invalid_yaml_syntax() {
    let temp = common::create_test_project();
    let invalid_yaml = r#"
ui:
  button: "Buy"
    invalid_indent: "Test"
"#;
    fs::write(temp.path().join("translations/en.yaml"), invalid_yaml).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse"));
}
#[test]
fn test_json_with_trailing_comma() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "button": "Buy",
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .failure();
}
#[test]
fn test_unicode_in_keys() {
    let temp = common::create_test_project();
    let json = r#"{
  "你好": "Hello",
  "مرحبا": "Welcome"
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_emojis_in_values() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "greeting": "Hello 👋",
    "celebration": "Party! 🎉🎊"
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success(); // Just verify it doesn't crash
}
#[test]
fn test_escape_sequences() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "quote": "He said \"Hi\"",
    "newline": "Line 1\nLine 2",
    "tab": "Col1\tCol2"
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_special_lua_characters() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "brackets": "[[nested]]",
    "quotes": "It's \"quoted\"",
    "backslash": "Path\\to\\file"
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_many_translations() {
    let temp = common::create_test_project();
    let mut translations = serde_json::Map::new();
    for i in 0..1000 {
        translations.insert(
            format!("key_{}", i),
            serde_json::Value::String(format!("Value {}", i)),
        );
    }

    let json = serde_json::to_string_pretty(&translations).unwrap();
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));
}
#[test]
fn test_deep_nesting() {
    let temp = common::create_test_project();
    let json = r#"{
  "level1": {
    "level2": {
      "level3": {
        "level4": {
          "level5": {
            "level6": {
              "level7": {
                "level8": {
                  "level9": {
                    "level10": "Deep value"
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
    let luau_path = temp.path().join("output/Translations.lua");
    common::assert_file_contains(
        &luau_path,
        "level1_level2_level3_level4_level5_level6_level7_level8_level9_level10",
    );
}
#[test]
fn test_very_long_keys() {
    let temp = common::create_test_project();
    let long_key = "a".repeat(200);
    let json = format!(
        r#"{{
  "{}": "Value"
}}"#,
        long_key
    );
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_very_long_values() {
    let temp = common::create_test_project();
    let long_value = "Lorem ipsum ".repeat(100);
    let json = format!(
        r#"{{
  "ui": {{
    "long_text": "{}"
  }}
}}"#,
        long_value
    );
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_missing_config_file() {
    let temp = tempfile::TempDir::new().unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to load config"));
}
#[test]
fn test_missing_translation_file() {
    let temp = common::create_test_project();
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_missing_output_directory() {
    let temp = common::create_test_project();
    let json = r#"{"ui": {"button": "Buy"}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();
    fs::remove_dir_all(temp.path().join("output")).ok();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    common::assert_file_exists(&temp.path().join("output"));
}
#[test]
fn test_duplicate_keys_in_same_file() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "button": "Buy",
    "button": "Sell"
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success(); // Parser handles this
}
#[test]
fn test_mixed_types_in_json() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "button": "Buy",
    "count": 123,
    "enabled": true,
    "items": ["a", "b", "c"]
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_single_character_key() {
    let temp = common::create_test_project();

    let json = r#"{"a": "Value"}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_single_character_value() {
    let temp = common::create_test_project();

    let json = r#"{"ui": {"button": "X"}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_empty_string_value() {
    let temp = common::create_test_project();

    let json = r#"{"ui": {"button": ""}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_whitespace_only_value() {
    let temp = common::create_test_project();

    let json = r#"{"ui": {"button": "   "}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_unsupported_locale_in_config() {
    let temp = tempfile::TempDir::new().unwrap();
    let config = r#"base_locale: en
supported_locales:
  - en
  - xx
input_directory: translations
output_directory: output
"#;
    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();
    fs::create_dir(temp.path().join("translations")).unwrap();

    let json = r#"{"ui": {"button": "Buy"}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported locale"));
}
#[test]
fn test_all_roblox_locales() {
    let temp = tempfile::TempDir::new().unwrap();
    let config = r#"base_locale: en
supported_locales:
  - en
  - es
  - fr
  - de
  - it
  - pt
  - ru
  - ja
  - ko
  - zh-cn
  - zh-tw
  - id
  - tr
  - vi
  - th
  - pl
  - uk
input_directory: translations
output_directory: output
"#;
    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();
    fs::create_dir(temp.path().join("translations")).unwrap();
    let json = r#"{"ui": {"button": "Buy"}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
