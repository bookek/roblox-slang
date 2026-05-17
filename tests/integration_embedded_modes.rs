use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
fn create_test_project(temp_dir: &Path, mode: &str) -> Result<()> {
    let translations_dir = temp_dir.join("translations");
    fs::create_dir_all(&translations_dir)?;
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
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;
    let output_file = temp_dir.path().join("output/Translations.lua");
    assert!(output_file.exists(), "Translations.lua must be generated");

    let types_file = temp_dir.path().join("output/types/Translations.d.luau");
    assert!(types_file.exists(), "Type definitions must be generated");

    let csv_file = temp_dir.path().join("output/roblox_upload.csv");
    assert!(csv_file.exists(), "CSV file must be generated");
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains("EMBEDDED_TRANSLATIONS"),
        "expected embedded data"
    );
    assert!(code.contains(r#"["en"] = {"#), "expected English locale");
    assert!(code.contains(r#"["id"] = {"#), "expected Indonesian locale");
    assert!(
        code.contains(r#"["ui.button"] = "Buy""#),
        "expected translation"
    );
    assert!(
        !code.contains("LocalizationService"),
        "expected no LocalizationService in embedded mode"
    );
    assert!(
        !code.contains("FormatByKey"),
        "expected no FormatByKey in embedded mode"
    );

    Ok(())
}

#[test]
fn test_cloud_mode_end_to_end() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_project(temp_dir.path(), "cloud")?;
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;
    let output_file = temp_dir.path().join("output/Translations.lua");
    assert!(output_file.exists());
    let code = fs::read_to_string(&output_file)?;
    assert!(
        !code.contains("EMBEDDED_TRANSLATIONS"),
        "expected no embedded data in cloud mode"
    );
    assert!(
        code.contains("LocalizationService"),
        "expected LocalizationService"
    );
    assert!(code.contains("FormatByKey"), "expected FormatByKey");

    Ok(())
}

#[test]
fn test_hybrid_mode_end_to_end() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_project(temp_dir.path(), "hybrid")?;
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;
    let output_file = temp_dir.path().join("output/Translations.lua");
    assert!(output_file.exists());
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains("EMBEDDED_TRANSLATIONS"),
        "expected embedded data"
    );
    assert!(
        code.contains("LocalizationService"),
        "expected LocalizationService"
    );
    assert!(code.contains("FormatByKey"), "expected FormatByKey");
    assert!(code.contains("pcall"), "expected pcall for cloud fallback");

    Ok(())
}

#[test]
fn test_default_mode_is_embedded() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let translations_dir = temp_dir.path().join("translations");
    fs::create_dir_all(&translations_dir)?;
    let en_content = r#"{"test": {"key": "value"}}"#;
    fs::write(translations_dir.join("en.json"), en_content)?;
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
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;
    let output_file = temp_dir.path().join("output/Translations.lua");
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains("EMBEDDED_TRANSLATIONS"),
        "expected embedded mode by default"
    );
    assert!(
        !code.contains("LocalizationService"),
        "expected no LocalizationService by default"
    );

    Ok(())
}

#[test]
fn test_multiple_locales_embedded() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let translations_dir = temp_dir.path().join("translations");
    fs::create_dir_all(&translations_dir)?;
    fs::write(translations_dir.join("en.json"), r#"{"test": "English"}"#)?;
    fs::write(
        translations_dir.join("id.json"),
        r#"{"test": "Indonesian"}"#,
    )?;
    fs::write(translations_dir.join("es.json"), r#"{"test": "Spanish"}"#)?;
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
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;
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
    let translations_dir = temp_dir.path().join("translations");
    fs::create_dir_all(&translations_dir)?;
    let special_content = r#"{
  "special": {
    "quotes": "He said \"Hello\"",
    "newline": "Line1\nLine2",
    "tab": "Col1\tCol2",
    "backslash": "Path\\to\\file"
  }
}"#;
    fs::write(translations_dir.join("en.json"), special_content)?;
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
    let config_path = temp_dir.path().join("slang-roblox.yaml");
    roblox_slang::cli::build::build(&config_path)?;
    let output_file = temp_dir.path().join("output/Translations.lua");
    let code = fs::read_to_string(&output_file)?;
    assert!(
        code.contains(r#"He said \"Hello\""#),
        "Quotes must be escaped"
    );
    assert!(code.contains(r#"Line1\nLine2"#), "Newlines must be escaped");
    assert!(code.contains(r#"Col1\tCol2"#), "Tabs must be escaped");
    assert!(
        code.contains(r#"Path\\to\\file"#),
        "Backslashes must be escaped"
    );

    Ok(())
}
