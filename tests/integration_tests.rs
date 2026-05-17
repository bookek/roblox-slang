#![allow(deprecated)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::doc_lazy_continuation)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
#[test]
fn test_init_command() {
    let temp = tempfile::TempDir::new().unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));
    common::assert_file_exists(&temp.path().join("slang-roblox.yaml"));
    common::assert_file_exists(&temp.path().join("translations"));
}
#[test]
fn test_build_command_basic() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Building translations"))
        .stdout(predicate::str::contains("Parsed en"))
        .stdout(predicate::str::contains("Parsed id"));
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));
    common::assert_file_exists(&temp.path().join("output/types/Translations.d.luau"));
    common::assert_file_exists(&temp.path().join("output/roblox_upload.csv"));
}
#[test]
fn test_build_generates_correct_luau() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    let luau_path = temp.path().join("output/Translations.lua");
    common::assert_file_contains(&luau_path, "function Translations.new");
    common::assert_file_contains(&luau_path, "function Translations:ui_buttons_buy");
    common::assert_file_contains(&luau_path, "function Translations:ui_buttons_sell");
    common::assert_file_contains(&luau_path, "function Translations:ui_labels_welcome");
}
#[test]
fn test_build_generates_type_definitions() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    let types_path = temp.path().join("output/types/Translations.d.luau");
    common::assert_file_contains(&types_path, "export type Translations");
    common::assert_file_contains(&types_path, "ui_buttons_buy");
}
#[test]
fn test_validate_command_all() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating translations"))
        .stdout(predicate::str::contains(
            "Checking for missing translations",
        ))
        .stdout(predicate::str::contains("Checking for conflicts"))
        .stdout(predicate::str::contains("Translation Coverage Report"));
}
#[test]
fn test_validate_missing_keys() {
    let temp = common::create_test_project();
    let en_json = r#"{
  "ui": {
    "button": "Buy",
    "label": "Welcome"
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), en_json).unwrap();
    let id_json = r#"{
  "ui": {
    "button": "Beli"
  }
}"#;
    fs::write(temp.path().join("translations/id.json"), id_json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--missing")
        .assert()
        .success()
        .stdout(predicate::str::contains("Missing in 'id'"))
        .stdout(predicate::str::contains("ui.label"));
}
#[test]
fn test_validate_coverage() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--coverage")
        .assert()
        .success()
        .stdout(predicate::str::contains("Translation Coverage Report"))
        .stdout(predicate::str::contains("Locale"))
        .stdout(predicate::str::contains("Coverage"))
        .stdout(predicate::str::contains("100.0%"));
}
#[test]
fn test_import_csv() {
    let temp = common::create_test_project();
    let csv_content = r#"Source,Context,Key,en,id
"Buy","","ui.button.buy","Buy","Beli"
"Sell","","ui.button.sell","Sell","Jual"
"#;
    fs::write(temp.path().join("import.csv"), csv_content).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("import")
        .arg("import.csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("Importing translations"));
    common::assert_file_exists(&temp.path().join("translations/en.json"));
    common::assert_file_exists(&temp.path().join("translations/id.json"));
}
#[test]
fn test_migrate_custom_json() {
    let temp = common::create_test_project();
    let custom_json = r#"{
  "translations": {
    "en": {
      "ui_button": "Buy"
    }
  }
}"#;
    fs::write(temp.path().join("custom.json"), custom_json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("migrate")
        .arg("--from")
        .arg("custom-json")
        .arg("--input")
        .arg("custom.json")
        .arg("--output")
        .arg("migrated.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Migration completed"));
    common::assert_file_exists(&temp.path().join("migrated.json"));
}
#[test]
fn test_migrate_gettext() {
    let temp = common::create_test_project();
    let po_content = r#"msgid "ui.button"
msgstr "Buy"
"#;
    fs::write(temp.path().join("translations.po"), po_content).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("migrate")
        .arg("--from")
        .arg("gettext")
        .arg("--input")
        .arg("translations.po")
        .arg("--output")
        .arg("migrated.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Migration completed"));
    common::assert_file_exists(&temp.path().join("migrated.json"));
}
#[test]
fn test_complete_workflow() {
    let temp = tempfile::TempDir::new().unwrap();
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("init")
        .assert()
        .success();
    let en_json = r#"{"ui": {"button": "Buy"}}"#;
    fs::write(temp.path().join("translations/en.json"), en_json).unwrap();

    let id_json = r#"{"ui": {"button": "Beli"}}"#;
    fs::write(temp.path().join("translations/id.json"), id_json).unwrap();
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--all")
        .assert()
        .success();
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));
    common::assert_file_exists(&temp.path().join("output/types/Translations.d.luau"));
    common::assert_file_exists(&temp.path().join("output/roblox_upload.csv"));
}
#[test]
fn test_error_handling_missing_config() {
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
fn test_error_handling_invalid_json() {
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
