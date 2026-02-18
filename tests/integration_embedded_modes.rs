use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a test project with translations
fn create_test_project(temp_dir: &Path, mode: &str) -> Result<()> {
    // Create translations directory first
    let translations_dir = temp_dir.join("translations");
    fs::create_dir_all(&translations_dir)?;

    // Create English translations
    let en_content = r#"{
  "ui": {
    "button": "Buy",
    "message": "Hello, {name}!"
  },
  "game": {
    "score": "Score: {score:int}"
  }
}"#;
    fs::write(translations_dir.join("en.json"), en_content)?;

    // Create Indonesian translations
    let id_content = r#"{
  "ui": {
    "button": "Beli",
    "message": "Halo, {name}!"
  },
  "game": {
    "score": "Skor: {score:int}"
  }
}"#;
    fs::write(translations_dir.join("id.json"), id_content)?;

    // Create config with absolute paths
    let translations_path = translations_dir.to_str().unwrap();
    let output_path = temp_dir.join("output").to_str().unwrap().to_string();

    let config_content = format!(
        r#"base_locale: en
supported_locales:
  - en
  - id

input_directory: {}
output_directory: {}

localization:
  mode: {}
"#,
        translations_path, output_path, mode
    );
    fs::write(temp_dir.join("slang-roblox.yaml"), config_content)?;

    Ok(())
}

#[test]
fn test_embedded_mode_end_to_end() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_project(temp_dir.path(), "embedded")?;

    // Run build
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;

    // Check output files exist
    let output_file = temp_dir.path().join("output/Translations.lua");
    assert!(output_file.exists(), "Translations.lua should be generated");

    let types_file = temp_dir.path().join("output/types/Translations.d.luau");
    assert!(types_file.exists(), "Type definitions should be generated");

    let csv_file = temp_dir.path().join("output/roblox_upload.csv");
    assert!(csv_file.exists(), "CSV file should be generated");

    // Check generated code contains embedded data
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains("EMBEDDED_TRANSLATIONS"),
        "Should contain embedded data"
    );
    assert!(
        code.contains(r#"["en"] = {"#),
        "Should contain English locale"
    );
    assert!(
        code.contains(r#"["id"] = {"#),
        "Should contain Indonesian locale"
    );
    assert!(
        code.contains(r#"["ui.button"] = "Buy""#),
        "Should contain translation"
    );

    // Should NOT contain LocalizationService
    assert!(
        !code.contains("LocalizationService"),
        "Should not contain LocalizationService in embedded mode"
    );
    assert!(
        !code.contains("FormatByKey"),
        "Should not contain FormatByKey in embedded mode"
    );

    Ok(())
}

#[test]
fn test_cloud_mode_end_to_end() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_project(temp_dir.path(), "cloud")?;

    // Run build
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;

    // Check output files exist
    let output_file = temp_dir.path().join("output/Translations.lua");
    assert!(output_file.exists());

    // Check generated code uses LocalizationService
    let code = fs::read_to_string(&output_file)?;
    assert!(
        !code.contains("EMBEDDED_TRANSLATIONS"),
        "Should NOT contain embedded data in cloud mode"
    );
    assert!(
        code.contains("LocalizationService"),
        "Should contain LocalizationService"
    );
    assert!(code.contains("FormatByKey"), "Should contain FormatByKey");

    Ok(())
}

#[test]
fn test_hybrid_mode_end_to_end() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_project(temp_dir.path(), "hybrid")?;

    // Run build
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;

    // Check output files exist
    let output_file = temp_dir.path().join("output/Translations.lua");
    assert!(output_file.exists());

    // Check generated code uses BOTH embedded and cloud
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains("EMBEDDED_TRANSLATIONS"),
        "Should contain embedded data"
    );
    assert!(
        code.contains("LocalizationService"),
        "Should contain LocalizationService"
    );
    assert!(code.contains("FormatByKey"), "Should contain FormatByKey");
    assert!(
        code.contains("pcall"),
        "Should use pcall for cloud fallback"
    );

    Ok(())
}

#[test]
fn test_default_mode_is_embedded() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create translations directory first
    let translations_dir = temp_dir.path().join("translations");
    fs::create_dir_all(&translations_dir)?;

    // Create translations
    let en_content = r#"{"test": {"key": "value"}}"#;
    fs::write(translations_dir.join("en.json"), en_content)?;

    // Create config WITHOUT localization section (using absolute paths)
    let translations_path = translations_dir.to_str().unwrap();
    let output_path = temp_dir.path().join("output").to_str().unwrap().to_string();

    let config_content = format!(
        r#"base_locale: en
supported_locales:
  - en

input_directory: {}
output_directory: {}
"#,
        translations_path, output_path
    );
    fs::write(temp_dir.path().join("slang-roblox.yaml"), config_content)?;

    // Run build
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;

    // Check generated code defaults to embedded mode
    let output_file = temp_dir.path().join("output/Translations.lua");
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains("EMBEDDED_TRANSLATIONS"),
        "Should default to embedded mode"
    );
    assert!(
        !code.contains("LocalizationService"),
        "Should not contain LocalizationService by default"
    );

    Ok(())
}

#[test]
fn test_multiple_locales_embedded() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create translations directory first
    let translations_dir = temp_dir.path().join("translations");
    fs::create_dir_all(&translations_dir)?;

    // Create translations
    fs::write(translations_dir.join("en.json"), r#"{"test": "English"}"#)?;
    fs::write(
        translations_dir.join("id.json"),
        r#"{"test": "Indonesian"}"#,
    )?;
    fs::write(translations_dir.join("es.json"), r#"{"test": "Spanish"}"#)?;

    // Create config with 3 locales (using absolute paths)
    let translations_path = translations_dir.to_str().unwrap();
    let output_path = temp_dir.path().join("output").to_str().unwrap().to_string();

    let config_content = format!(
        r#"base_locale: en
supported_locales:
  - en
  - id
  - es

input_directory: {}
output_directory: {}

localization:
  mode: embedded
"#,
        translations_path, output_path
    );
    fs::write(temp_dir.path().join("slang-roblox.yaml"), config_content)?;

    // Run build
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;

    // Check all locales are embedded
    let output_file = temp_dir.path().join("output/Translations.lua");
    let code = fs::read_to_string(&output_file)?;
    assert!(code.contains(r#"["en"] = {"#));
    assert!(code.contains(r#"["id"] = {"#));
    assert!(code.contains(r#"["es"] = {"#));
    assert!(code.contains(r#""English""#));
    assert!(code.contains(r#""Indonesian""#));
    assert!(code.contains(r#""Spanish""#));

    Ok(())
}

#[test]
fn test_special_characters_embedded() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create translations directory first
    let translations_dir = temp_dir.path().join("translations");
    fs::create_dir_all(&translations_dir)?;

    // Add translations with special characters
    let special_content = r#"{
  "special": {
    "quotes": "He said \"Hello\"",
    "newline": "Line1\nLine2",
    "tab": "Col1\tCol2",
    "backslash": "Path\\to\\file"
  }
}"#;
    fs::write(translations_dir.join("en.json"), special_content)?;

    // Create config (using absolute paths)
    let translations_path = translations_dir.to_str().unwrap();
    let output_path = temp_dir.path().join("output").to_str().unwrap().to_string();

    let config_content = format!(
        r#"base_locale: en
supported_locales:
  - en

input_directory: {}
output_directory: {}

localization:
  mode: embedded
"#,
        translations_path, output_path
    );
    fs::write(temp_dir.path().join("slang-roblox.yaml"), config_content)?;

    // Run build
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;

    // Check special characters are properly escaped
    let output_file = temp_dir.path().join("output/Translations.lua");
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains(r#"He said \"Hello\""#),
        "Quotes should be escaped"
    );
    assert!(
        code.contains(r#"Line1\nLine2"#),
        "Newlines should be escaped"
    );
    assert!(code.contains(r#"Col1\tCol2"#), "Tabs should be escaped");
    assert!(
        code.contains(r#"Path\\to\\file"#),
        "Backslashes should be escaped"
    );

    Ok(())
}
