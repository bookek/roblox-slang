#![allow(deprecated)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::doc_lazy_continuation)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::Duration;
#[test]
fn test_init_creates_project_structure() {
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
    let config = fs::read_to_string(temp.path().join("slang-roblox.yaml")).unwrap();
    assert!(config.contains("base_locale"));
    assert!(config.contains("supported_locales"));
}
#[test]
fn test_init_with_overrides_flag() {
    let temp = tempfile::TempDir::new().unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("init")
        .arg("--with-overrides")
        .assert()
        .success();
    common::assert_file_exists(&temp.path().join("overrides.yaml"));
    let overrides = fs::read_to_string(temp.path().join("overrides.yaml")).unwrap();
    assert!(overrides.contains("overrides"));
}
#[test]
fn test_init_already_initialized() {
    let temp = tempfile::TempDir::new().unwrap();
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("init")
        .assert()
        .success();
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("init")
        .assert()
        .success();
}
#[test]
fn test_build_generates_all_files() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Building translations"));
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));
    common::assert_file_exists(&temp.path().join("output/types/Translations.d.luau"));
    common::assert_file_exists(&temp.path().join("output/roblox_upload.csv"));
}
#[test]
fn test_build_with_multiple_locales() {
    let temp = common::create_test_project();
    for locale in &["en", "id", "es", "fr", "de"] {
        let json = format!(r#"{{"ui": {{"button": "Buy in {}"}}}}"#, locale);
        fs::write(
            temp.path().join(format!("translations/{}.json", locale)),
            json,
        )
        .unwrap();
    }
    let config = r#"base_locale: en
supported_locales:
  - en
  - id
  - es
  - fr
  - de
input_directory: translations
output_directory: output
"#;
    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parsed en"))
        .stdout(predicate::str::contains("Parsed id"))
        .stdout(predicate::str::contains("Parsed es"));
}
#[test]
fn test_build_with_yaml_files() {
    let temp = common::create_test_project();
    let yaml = r#"ui:
  button: Buy
  label: Welcome
"#;
    fs::write(temp.path().join("translations/en.yaml"), yaml).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    common::assert_file_exists(&temp.path().join("output/Translations.lua"));
}
#[test]
fn test_build_with_mixed_formats() {
    let temp = common::create_test_project();
    let json = r#"{"ui": {"button": "Buy"}}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();
    let yaml = r#"ui:
  button: Beli
"#;
    fs::write(temp.path().join("translations/id.yaml"), yaml).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
}
#[test]
fn test_build_with_overrides() {
    let temp = common::create_test_project_with_translations();
    let config = r#"base_locale: en
supported_locales:
  - en
  - id
input_directory: translations
output_directory: output
overrides:
  enabled: true
  file: overrides.yaml
"#;
    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();
    let overrides = r#"en:
  ui.buttons.buy: "Purchase"
id:
  ui.buttons.buy: "Beli"
"#;
    fs::write(temp.path().join("overrides.yaml"), overrides).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();
    let csv = fs::read_to_string(temp.path().join("output/roblox_upload.csv")).unwrap();

    assert!(
        csv.contains("Purchase"),
        "Override 'Purchase' must be in generated CSV for upload to Roblox Cloud"
    );

    assert!(
        csv.contains("Beli"),
        "Override 'Beli' must be in generated CSV for Indonesian locale"
    );
}
#[test]
fn test_watch_mode_starts() {
    let temp = common::create_test_project_with_translations();
    let mut cmd = Command::cargo_bin("roblox-slang").unwrap();
    let assert = cmd
        .current_dir(&temp)
        .arg("build")
        .arg("--watch")
        .timeout(Duration::from_secs(2))
        .assert();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Starting watch mode") || stdout.contains("Watching for changes"),
        "Watch mode must start successfully. Output: {}",
        stdout
    );
}
#[test]
fn test_import_basic_csv() {
    let temp = common::create_test_project();
    let csv = r#"Source,Context,Key,en,id
"Buy","","ui.button.buy","Buy","Beli"
"Sell","","ui.button.sell","Sell","Jual"
"#;
    fs::write(temp.path().join("import.csv"), csv).unwrap();

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
    let en_json = fs::read_to_string(temp.path().join("translations/en.json")).unwrap();
    assert!(en_json.contains("Buy"));
}
#[test]
fn test_import_multiple_locales() {
    let temp = common::create_test_project();
    let csv = r#"Source,Context,Key,en,id,es,fr,de
"Buy","","ui.button","Buy","Beli","Comprar","Acheter","Kaufen"
"#;
    fs::write(temp.path().join("import.csv"), csv).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("import")
        .arg("import.csv")
        .assert()
        .success();
    for locale in &["en", "id", "es", "fr", "de"] {
        common::assert_file_exists(&temp.path().join(format!("translations/{}.json", locale)));
    }
}
#[test]
fn test_import_with_empty_cells() {
    let temp = common::create_test_project();
    let csv = r#"Source,Context,Key,en,id
"Buy","","ui.button.buy","Buy",""
"Sell","","ui.button.sell","","Jual"
"#;
    fs::write(temp.path().join("import.csv"), csv).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("import")
        .arg("import.csv")
        .assert()
        .success();
}
#[test]
fn test_import_with_special_characters() {
    let temp = common::create_test_project();
    let csv = r#"Source,Context,Key,en,id
"He said ""Hi""","","ui.greeting","He said ""Hi""","Dia bilang ""Hai"""
"Item 1, Item 2","","ui.list","Item 1, Item 2","Barang 1, Barang 2"
"#;
    fs::write(temp.path().join("import.csv"), csv).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("import")
        .arg("import.csv")
        .assert()
        .success();
}
#[test]
fn test_validate_missing_flag() {
    let temp = common::create_test_project();
    let en = r#"{"ui": {"button": "Buy", "label": "Welcome"}}"#;
    fs::write(temp.path().join("translations/en.json"), en).unwrap();
    let id = r#"{"ui": {"button": "Beli"}}"#;
    fs::write(temp.path().join("translations/id.json"), id).unwrap();

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
fn test_validate_conflicts_flag() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--conflicts")
        .assert()
        .success()
        .stdout(predicate::str::contains("Checking for conflicts"));
}
#[test]
fn test_validate_coverage_flag() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--coverage")
        .assert()
        .success()
        .stdout(predicate::str::contains("Translation Coverage Report"))
        .stdout(predicate::str::contains("100.0%"));
}
#[test]
fn test_validate_unused_flag() {
    let temp = common::create_test_project_with_translations();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/test.lua"),
        "local text = t.ui.buttons.buy()",
    )
    .unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--unused")
        .arg("--source")
        .arg("src")
        .assert()
        .success()
        .stdout(predicate::str::contains("Checking for unused keys"));
}
#[test]
fn test_validate_all_flag() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Checking for missing translations",
        ))
        .stdout(predicate::str::contains("Checking for conflicts"))
        .stdout(predicate::str::contains("Translation Coverage Report"));
}
#[test]
fn test_validate_multiple_flags() {
    let temp = common::create_test_project_with_translations();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("validate")
        .arg("--missing")
        .arg("--coverage")
        .arg("--conflicts")
        .assert()
        .success();
}
#[test]
fn test_migrate_custom_json_format() {
    let temp = common::create_test_project();
    let custom = r#"{
  "translations": {
    "en": {
      "UI_BUTTON": "Buy"
    }
  }
}"#;
    fs::write(temp.path().join("custom.json"), custom).unwrap();

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
fn test_migrate_gettext_format() {
    let temp = common::create_test_project();
    let po = r#"msgid "ui.button"
msgstr "Buy"

msgid "ui.label"
msgstr "Welcome"
"#;
    fs::write(temp.path().join("translations.po"), po).unwrap();

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
        .success();

    common::assert_file_exists(&temp.path().join("migrated.json"));
}
#[test]
fn test_migrate_with_snake_to_camel_transform() {
    let temp = common::create_test_project();

    let custom = r#"{
  "translations": {
    "en": {
      "ui_button_buy": "Buy"
    }
  }
}"#;
    fs::write(temp.path().join("custom.json"), custom).unwrap();

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
        .arg("--transform")
        .arg("snake-to-camel")
        .assert()
        .success();
}
#[test]
fn test_migrate_with_upper_to_lower_transform() {
    let temp = common::create_test_project();

    let custom = r#"{
  "translations": {
    "en": {
      "UI_BUTTON": "Buy"
    }
  }
}"#;
    fs::write(temp.path().join("custom.json"), custom).unwrap();

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
        .arg("--transform")
        .arg("upper-to-lower")
        .assert()
        .success();
}
#[test]
fn test_migrate_with_dot_to_nested_transform() {
    let temp = common::create_test_project();

    let custom = r#"{
  "translations": {
    "en": {
      "ui.button.buy": "Buy"
    }
  }
}"#;
    fs::write(temp.path().join("custom.json"), custom).unwrap();

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
        .arg("--transform")
        .arg("dot-to-nested")
        .assert()
        .success();
}
#[test]
fn test_migrate_with_no_transform() {
    let temp = common::create_test_project();

    let custom = r#"{
  "translations": {
    "en": {
      "ui_button": "Buy"
    }
  }
}"#;
    fs::write(temp.path().join("custom.json"), custom).unwrap();

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
        .arg("--transform")
        .arg("none")
        .assert()
        .success();
}
#[test]
fn test_error_missing_required_argument() {
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .arg("import")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
#[test]
fn test_error_invalid_command() {
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized"));
}
#[test]
fn test_error_invalid_flag() {
    let temp = common::create_test_project();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .arg("--invalid-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected"));
}
#[test]
fn test_help_flag() {
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("Commands"));
}
#[test]
fn test_version_flag() {
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("roblox-slang"));
}
#[test]
fn test_subcommand_help() {
    Command::cargo_bin("roblox-slang")
        .unwrap()
        .arg("build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Build translations"));
}
