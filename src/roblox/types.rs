use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Roblox Open Cloud sync errors.
#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum CloudSyncError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded. Retrying in {retry_after}s (attempt {attempt}/3)")]
    RateLimitError { retry_after: u64, attempt: u32 },

    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Network error: {0}")]
    #[allow(dead_code)]
    NetworkError(String),

    #[error("Validation failed in {file}:{line} - {reason}")]
    #[allow(dead_code)]
    ValidationError {
        file: String,
        line: usize,
        reason: String,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("API error: {0}")]
    ApiError(String),
}

/// Table entries returned by Roblox Open Cloud.
#[derive(Debug, Deserialize)]
pub struct GetTableEntriesResponse {
    #[serde(alias = "entries", alias = "data")]
    pub entries: Vec<LocalizationEntry>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "nextPageCursor")]
    #[allow(dead_code)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct UpdateTableRequest {
    pub entries: Vec<LocalizationEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ListTablesResponse {
    pub data: Vec<TableInfo>,
}

/// Roblox localization table metadata.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TableInfo {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "ownerType")]
    pub owner_type: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<i64>,
    #[serde(rename = "assetId")]
    pub asset_id: Option<i64>,
}

/// Localization entry in Roblox Open Cloud format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationEntry {
    pub identifier: Identifier,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EntryMetadata>,

    pub translations: Vec<Translation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub key: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,

    #[serde(rename = "entryType", skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub locale: String,

    #[serde(rename = "translationText")]
    pub translation_text: String,
}

/// Local defaults for Roblox Open Cloud commands.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CloudConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,

    /// Prefer `ROBLOX_API_KEY` over storing secrets in config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

#[derive(Debug)]
pub struct UploadStats {
    pub entries_uploaded: usize,
    pub locales_processed: usize,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct DownloadStats {
    pub entries_downloaded: usize,
    pub locales_created: usize,
    pub locales_updated: usize,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct SyncStats {
    pub entries_added: usize,
    pub entries_updated: usize,
    pub entries_deleted: usize,
    pub conflicts_skipped: usize,
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_sync_error_display() {
        let auth_error = CloudSyncError::AuthenticationError("Invalid API key".to_string());
        assert!(auth_error.to_string().contains("Authentication failed"));

        let rate_limit_error = CloudSyncError::RateLimitError {
            retry_after: 60,
            attempt: 2,
        };
        assert!(rate_limit_error.to_string().contains("Rate limit exceeded"));
        assert!(rate_limit_error.to_string().contains("60s"));

        let server_error = CloudSyncError::ServerError {
            status: 500,
            message: "Internal Server Error".to_string(),
        };
        assert!(server_error.to_string().contains("500"));

        let config_error = CloudSyncError::ConfigError("Missing table_id".to_string());
        assert!(config_error.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_localization_entry_creation() {
        let entry = LocalizationEntry {
            identifier: Identifier {
                key: "ui.button".to_string(),
                context: None,
                source: "Buy".to_string(),
            },
            metadata: None,
            translations: vec![Translation {
                locale: "en".to_string(),
                translation_text: "Buy".to_string(),
            }],
        };

        assert_eq!(entry.identifier.key, "ui.button");
        assert_eq!(entry.translations.len(), 1);
        assert_eq!(entry.translations[0].locale, "en");
    }

    #[test]
    fn test_identifier_with_context() {
        let identifier = Identifier {
            key: "common.close".to_string(),
            context: Some("button".to_string()),
            source: "Close".to_string(),
        };

        assert_eq!(identifier.context, Some("button".to_string()));
    }

    #[test]
    fn test_entry_metadata() {
        let metadata = EntryMetadata {
            example: Some("Used in shop UI".to_string()),
            entry_type: Some("manual".to_string()),
        };

        assert_eq!(metadata.example, Some("Used in shop UI".to_string()));
        assert_eq!(metadata.entry_type, Some("manual".to_string()));
    }

    #[test]
    fn test_translation_serialization() {
        let translation = Translation {
            locale: "es".to_string(),
            translation_text: "Comprar".to_string(),
        };

        let json = serde_json::to_string(&translation).unwrap();
        assert!(json.contains("\"locale\":\"es\""));
        assert!(json.contains("\"translationText\":\"Comprar\""));
    }

    #[test]
    fn test_cloud_config_defaults() {
        let config = CloudConfig::default();
        assert!(config.table_id.is_none());
        assert!(config.game_id.is_none());
        assert!(config.api_key.is_none());
        assert!(config.strategy.is_none());
    }

    #[test]
    fn test_cloud_config_with_values() {
        let config = CloudConfig {
            table_id: Some("table-123".to_string()),
            game_id: Some("game-456".to_string()),
            api_key: Some("key-789".to_string()),
            strategy: Some("merge".to_string()),
        };

        assert_eq!(config.table_id, Some("table-123".to_string()));
        assert_eq!(config.game_id, Some("game-456".to_string()));
        assert_eq!(config.strategy, Some("merge".to_string()));
    }

    #[test]
    fn test_upload_stats() {
        let stats = UploadStats {
            entries_uploaded: 100,
            locales_processed: 3,
            duration: Duration::from_secs(5),
        };

        assert_eq!(stats.entries_uploaded, 100);
        assert_eq!(stats.locales_processed, 3);
        assert_eq!(stats.duration.as_secs(), 5);
    }

    #[test]
    fn test_download_stats() {
        let stats = DownloadStats {
            entries_downloaded: 50,
            locales_created: 2,
            locales_updated: 1,
            duration: Duration::from_secs(3),
        };

        assert_eq!(stats.entries_downloaded, 50);
        assert_eq!(stats.locales_created, 2);
        assert_eq!(stats.locales_updated, 1);
    }

    #[test]
    fn test_sync_stats() {
        let stats = SyncStats {
            entries_added: 10,
            entries_updated: 20,
            entries_deleted: 5,
            conflicts_skipped: 2,
            duration: Duration::from_secs(10),
        };

        assert_eq!(stats.entries_added, 10);
        assert_eq!(stats.entries_updated, 20);
        assert_eq!(stats.entries_deleted, 5);
        assert_eq!(stats.conflicts_skipped, 2);
    }

    #[test]
    fn test_get_table_entries_response_deserialization() {
        let json = r#"{
            "entries": [
                {
                    "identifier": {
                        "key": "test.key",
                        "source": "Test"
                    },
                    "translations": [
                        {
                            "locale": "en",
                            "translationText": "Test"
                        }
                    ]
                }
            ]
        }"#;

        let response: GetTableEntriesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].identifier.key, "test.key");
    }

    #[test]
    fn test_localization_entry_clone() {
        let entry = LocalizationEntry {
            identifier: Identifier {
                key: "key".to_string(),
                context: None,
                source: "Source".to_string(),
            },
            metadata: None,
            translations: vec![],
        };

        let cloned = entry.clone();
        assert_eq!(entry.identifier.key, cloned.identifier.key);
    }
}
