pub mod custom_json;
pub mod gettext;
pub mod key_transform;

use anyhow::Result;
use std::path::Path;
#[derive(Debug, Clone, Copy)]
pub enum MigrationFormat {
    CustomJson,
    Gettext,
}
#[derive(Debug, Clone, Copy)]
pub enum KeyTransform {
    SnakeToCamel,
    UpperToLower,
    DotToNested,
    None,
}
pub fn migrate(
    format: MigrationFormat,
    input_path: &Path,
    output_path: &Path,
    transform: KeyTransform,
) -> Result<()> {
    match format {
        MigrationFormat::CustomJson => {
            custom_json::migrate_custom_json(input_path, output_path, transform)
        }
        MigrationFormat::Gettext => gettext::migrate_gettext(input_path, output_path, transform),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_migration_format_debug() {
        let format = MigrationFormat::CustomJson;
        assert!(format!("{:?}", format).contains("CustomJson"));

        let format = MigrationFormat::Gettext;
        assert!(format!("{:?}", format).contains("Gettext"));
    }

    #[test]
    fn test_key_transform_debug() {
        let transform = KeyTransform::SnakeToCamel;
        assert!(format!("{:?}", transform).contains("SnakeToCamel"));

        let transform = KeyTransform::None;
        assert!(format!("{:?}", transform).contains("None"));
    }

    #[test]
    fn test_migrate_custom_json() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.json");

        let json = r#"{
            "ui": {
                "button": "Buy"
            }
        }"#;
        fs::write(&input_path, json).unwrap();

        let result = migrate(
            MigrationFormat::CustomJson,
            &input_path,
            &output_path,
            KeyTransform::None,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_migrate_gettext() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.po");
        let output_path = temp_dir.path().join("output.json");

        let po = r#"
msgid "hello"
msgstr "Hello"

msgid "world"
msgstr "World"
"#;
        fs::write(&input_path, po).unwrap();

        let result = migrate(
            MigrationFormat::Gettext,
            &input_path,
            &output_path,
            KeyTransform::None,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_migrate_with_transform() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.json");

        let json = r#"{
            "snake_case_key": "Value"
        }"#;
        fs::write(&input_path, json).unwrap();

        let result = migrate(
            MigrationFormat::CustomJson,
            &input_path,
            &output_path,
            KeyTransform::SnakeToCamel,
        );

        assert!(result.is_ok());
    }
}
