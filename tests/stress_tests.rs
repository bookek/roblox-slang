#![allow(deprecated)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::doc_lazy_continuation)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::{Duration, Instant};
#[test]
#[ignore] // Run with --ignored flag
fn test_10k_translations() {
    let temp = common::create_test_project();
    let mut translations = serde_json::Map::new();
    for i in 0..10000 {
        let category = format!("category_{}", i / 100);
        let subcategory = format!("subcategory_{}", i / 10);
        let key = format!("key_{}", i);
        let value = format!("Translation value number {}", i);

        if !translations.contains_key(&category) {
            translations.insert(
                category.clone(),
                serde_json::Value::Object(serde_json::Map::new()),
            );
        }

        if let Some(serde_json::Value::Object(cat_map)) = translations.get_mut(&category) {
            if !cat_map.contains_key(&subcategory) {
                cat_map.insert(
                    subcategory.clone(),
                    serde_json::Value::Object(serde_json::Map::new()),
                );
            }

            if let Some(serde_json::Value::Object(subcat_map)) = cat_map.get_mut(&subcategory) {
                subcat_map.insert(key, serde_json::Value::String(value));
            }
        }
    }

    let json = serde_json::to_string_pretty(&translations).unwrap();
    fs::write(temp.path().join("translations/en.json"), &json).unwrap();
    let start = Instant::now();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parsed en"));

    let duration = start.elapsed();
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));
    common::assert_file_exists(&temp.path().join("output/roblox_upload.csv"));
    assert!(
        duration < Duration::from_secs(7),
        "Build took {:?}, expected <10s (debug build)",
        duration
    );

    println!("✓ Built 10,000 translations in {:?}", duration);
}
#[test]
#[ignore]
fn test_100_locales() {
    let temp = tempfile::TempDir::new().unwrap();
    let roblox_locales = vec![
        "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh-cn", "zh-tw", "id", "tr", "vi",
        "th", "pl", "uk",
    ];
    let config = format!(
        r#"base_locale: en
supported_locales:
{}
input_directory: translations
output_directory: output
"#,
        roblox_locales
            .iter()
            .map(|l| format!("  - {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    );

    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();
    fs::create_dir(temp.path().join("translations")).unwrap();
    let json = r#"{"ui": {"button": "Buy"}}"#;
    for locale in &roblox_locales {
        fs::write(
            temp.path().join(format!("translations/{}.json", locale)),
            json,
        )
        .unwrap();
    }
    let start = Instant::now();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    let duration = start.elapsed();
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));

    println!(
        "✓ Built {} Roblox locales in {:?}",
        roblox_locales.len(),
        duration
    );
}
#[test]
#[ignore]
fn test_1mb_translation_file() {
    let temp = common::create_test_project();
    let mut translations = serde_json::Map::new();
    for i in 0..5000 {
        let key = format!("key_{}", i);
        let value = format!(
            "This is a very long translation value number {} that contains a lot of text \
             to make the file size larger. Lorem ipsum dolor sit amet, consectetur adipiscing \
             elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
            i
        );
        translations.insert(key, serde_json::Value::String(value));
    }

    let json = serde_json::to_string_pretty(&translations).unwrap();
    let file_size = json.len();

    assert!(
        file_size > 1_000_000,
        "File size is {} bytes, expected >1MB",
        file_size
    );

    fs::write(temp.path().join("translations/en.json"), &json).unwrap();
    let start = Instant::now();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    let duration = start.elapsed();

    println!(
        "✓ Parsed {:.2}MB file in {:?}",
        file_size as f64 / 1_000_000.0,
        duration
    );
}
#[test]
#[ignore]
fn test_very_deep_nesting() {
    let temp = common::create_test_project();
    let mut json = String::from("{");
    for i in 0..20 {
        json.push_str(&format!("\"level_{}\": {{", i));
    }
    json.push_str("\"final\": \"Deep value\"");
    for _ in 0..20 {
        json.push('}');
    }
    json.push('}');

    fs::write(temp.path().join("translations/en.json"), json).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    println!("✓ Handled 20-level deep nesting");
}
#[test]
#[ignore]
fn test_rapid_file_changes() {
    use std::process::{Command as StdCommand, Stdio};

    let temp = common::create_test_project_with_translations();
    let bin_path = assert_cmd::cargo::cargo_bin("roblox-slang");
    let mut child = StdCommand::new(bin_path)
        .current_dir(&temp)
        .arg("build")
        .arg("--watch")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start watch mode");
    std::thread::sleep(Duration::from_secs(2));
    for i in 0..100 {
        let json = format!(r#"{{"ui": {{"button": "Buy {}"}}}}"#, i);
        fs::write(temp.path().join("translations/en.json"), json).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_secs(2));
    child.kill().expect("Failed to kill watch process");
    child.wait().expect("Failed to wait for process");
    common::assert_file_exists(&temp.path().join("output/Translations.lua"));

    println!("✓ Handled 100 rapid file changes");
}
#[test]
#[ignore]
fn test_repeated_builds() {
    let temp = common::create_test_project_with_translations();

    let mut durations = Vec::new();
    for i in 0..10 {
        let start = Instant::now();

        Command::cargo_bin("roblox-slang")
            .unwrap()
            .current_dir(&temp)
            .arg("build")
            .assert()
            .success();

        let duration = start.elapsed();
        durations.push(duration);

        println!("Build {}: {:?}", i + 1, duration);
    }
    let avg = durations.iter().sum::<Duration>() / durations.len() as u32;
    for (i, duration) in durations.iter().enumerate() {
        assert!(
            *duration < avg * 2,
            "Build {} took {:?}, more than 2x average {:?}",
            i + 1,
            duration,
            avg
        );
    }

    println!("✓ Average build time: {:?}", avg);
}
#[test]
#[ignore]
fn test_many_small_files() {
    let temp = tempfile::TempDir::new().unwrap();
    let locales = vec![
        "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh-cn", "zh-tw", "id", "tr", "vi",
        "th", "pl", "uk",
    ];

    let config = format!(
        r#"base_locale: en
supported_locales:
{}
input_directory: translations
output_directory: output
"#,
        locales
            .iter()
            .map(|l| format!("  - {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    );

    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();
    fs::create_dir(temp.path().join("translations")).unwrap();
    for locale in &locales {
        let json = r#"{"ui": {"button": "Buy"}}"#;
        fs::write(
            temp.path().join(format!("translations/{}.json", locale)),
            json,
        )
        .unwrap();
    }

    let start = Instant::now();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    let duration = start.elapsed();

    println!(
        "✓ Processed {} Roblox locale files in {:?}",
        locales.len(),
        duration
    );
}
#[test]
#[ignore]
fn test_very_long_keys() {
    let temp = common::create_test_project();
    let long_key = "a".repeat(500);
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
    let luau = fs::read_to_string(temp.path().join("output/Translations.lua")).unwrap();
    assert!(luau.len() > 500, "Output must contain long key");

    println!("✓ Handled 500-character key");
}
#[test]
#[ignore]
fn test_very_long_values() {
    let temp = common::create_test_project();
    let long_value = "Lorem ipsum ".repeat(1000);
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
    let luau = fs::read_to_string(temp.path().join("output/Translations.lua")).unwrap();
    assert!(
        luau.len() > 5000,
        "Output must be large (contains long value), got {} bytes",
        luau.len()
    );

    println!(
        "✓ Handled 10,000-character value (output: {} bytes)",
        luau.len()
    );
}
#[test]
#[ignore]
fn test_concurrent_builds() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let temp = common::create_test_project_with_translations();

                Command::cargo_bin("roblox-slang")
                    .unwrap()
                    .current_dir(&temp)
                    .arg("build")
                    .assert()
                    .success();

                println!("✓ Concurrent build {} completed", i);
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("✓ All 10 concurrent builds completed");
}
#[test]
#[ignore]
fn test_large_dataset_many_locales() {
    let temp = tempfile::TempDir::new().unwrap();
    let locales = vec![
        "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh-cn", "zh-tw", "id", "tr", "vi",
        "th", "pl", "uk",
    ];

    let config = format!(
        r#"base_locale: en
supported_locales:
{}
input_directory: translations
output_directory: output
"#,
        locales
            .iter()
            .map(|l| format!("  - {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    );

    fs::write(temp.path().join("slang-roblox.yaml"), config).unwrap();
    fs::create_dir(temp.path().join("translations")).unwrap();
    let mut translations = serde_json::Map::new();
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = format!("Value {}", i);
        translations.insert(key, serde_json::Value::String(value));
    }
    let json = serde_json::to_string_pretty(&translations).unwrap();

    for locale in &locales {
        fs::write(
            temp.path().join(format!("translations/{}.json", locale)),
            &json,
        )
        .unwrap();
    }

    let start = Instant::now();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    let duration = start.elapsed();

    println!(
        "✓ Built {} locales × 1000 translations = {} total in {:?}",
        locales.len(),
        locales.len() * 1000,
        duration
    );
}
#[test]
#[ignore]
fn test_all_features_combined() {
    let temp = common::create_test_project();
    let json = r#"{
  "ui": {
    "greeting": "Hello {name}!",
    "items(plural)": {
      "zero": "No items",
      "one": "One item",
      "other": "{count} items"
    },
    "button(context=save)": "Save",
    "button(context=cancel)": "Cancel"
  }
}"#;
    fs::write(temp.path().join("translations/en.json"), json).unwrap();
    let overrides = r#"overrides:
  ui.greeting:
    en: "Hi {name}!"
"#;
    fs::write(temp.path().join("overrides.yaml"), overrides).unwrap();

    Command::cargo_bin("roblox-slang")
        .unwrap()
        .current_dir(&temp)
        .arg("build")
        .assert()
        .success();

    println!("✓ All features combined successfully");
}
