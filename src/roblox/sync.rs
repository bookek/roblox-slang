use super::client::RobloxCloudClient;
use super::merge::{MergeEngine, MergeStrategy};
use super::types::{DownloadStats, LocalizationEntry, SyncStats, UploadStats};
use crate::config::Config;
use crate::parser::{self, Translation};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Orchestrates synchronization between local translation files and Roblox Cloud
///
/// The `SyncOrchestrator` provides high-level operations for syncing translations:
/// - `upload()` - Push local translations to cloud
/// - `download()` - Pull translations from cloud
/// - `sync()` - Bidirectional sync with merge strategies
///
/// # Example
///
/// ```no_run
/// use roblox_slang::roblox::{RobloxCloudClient, SyncOrchestrator};
/// use roblox_slang::config::Config;
///
/// # async fn example() -> anyhow::Result<()> {
/// let client = RobloxCloudClient::new("api_key".to_string())?;
/// let config = Config::default();
/// let orchestrator = SyncOrchestrator::new(client, config);
///
/// // Upload translations
/// let stats = orchestrator.upload("table_id", false).await?;
/// println!("Uploaded {} entries", stats.entries_uploaded);
/// # Ok(())
/// # }
/// ```
pub struct SyncOrchestrator {
    /// HTTP client for Roblox Cloud API
    client: RobloxCloudClient,
    /// Configuration
    config: Config,
}

impl SyncOrchestrator {
    /// Create a new sync orchestrator
    ///
    /// # Arguments
    ///
    /// * `client` - Authenticated Roblox Cloud API client
    /// * `config` - Project configuration
    pub fn new(client: RobloxCloudClient, config: Config) -> Self {
        Self { client, config }
    }

    /// Upload local translations to Roblox Cloud
    ///
    /// Reads all translation files from the input directory, converts them to
    /// Roblox Cloud format, and uploads to the specified localization table.
    ///
    /// # Arguments
    ///
    /// * `table_id` - Roblox localization table ID
    /// * `dry_run` - If true, skip actual upload (preview only)
    ///
    /// # Returns
    ///
    /// Statistics about the upload operation including entries uploaded,
    /// locales processed, and duration.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Translation files cannot be read
    /// - API request fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use roblox_slang::roblox::{RobloxCloudClient, SyncOrchestrator};
    /// # use roblox_slang::config::Config;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = RobloxCloudClient::new("api_key".to_string())?;
    /// # let config = Config::default();
    /// # let orchestrator = SyncOrchestrator::new(client, config);
    /// let stats = orchestrator.upload("table_id", false).await?;
    /// println!("Uploaded {} entries in {:?}", stats.entries_uploaded, stats.duration);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload(&self, table_id: &str, dry_run: bool) -> Result<UploadStats> {
        let start = Instant::now();

        // Read all local translation files
        let translations = self.read_local_translations()?;

        let entries_count = translations.len();
        let locales: std::collections::HashSet<_> =
            translations.iter().map(|t| t.locale.clone()).collect();
        let locales_count = locales.len();

        if !dry_run {
            // Convert to LocalizationEntry format
            let entries = self.translations_to_entries(&translations);

            // Upload to cloud in batches (Roblox API has a limit per request)
            // Based on testing, the API fails with 500 error when uploading too many entries at once
            const BATCH_SIZE: usize = 20;

            let game_id = self
                .config
                .cloud
                .as_ref()
                .and_then(|c| c.game_id.as_deref());

            for chunk in entries.chunks(BATCH_SIZE) {
                self.client
                    .update_table_entries(table_id, chunk, game_id)
                    .await
                    .context("Failed to upload translations")?;
            }
        }

        let duration = start.elapsed();

        Ok(UploadStats {
            entries_uploaded: entries_count,
            locales_processed: locales_count,
            duration,
        })
    }

    /// Download translations from Roblox Cloud
    ///
    /// Fetches all translations from the specified localization table and writes
    /// them to local translation files (one file per locale).
    ///
    /// # Arguments
    ///
    /// * `table_id` - Roblox localization table ID
    /// * `dry_run` - If true, skip file writes (preview only)
    ///
    /// # Returns
    ///
    /// Statistics about the download operation including entries downloaded,
    /// locales created/updated, and duration.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - API request fails
    /// - Files cannot be written
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use roblox_slang::roblox::{RobloxCloudClient, SyncOrchestrator};
    /// # use roblox_slang::config::Config;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = RobloxCloudClient::new("api_key".to_string())?;
    /// # let config = Config::default();
    /// # let orchestrator = SyncOrchestrator::new(client, config);
    /// let stats = orchestrator.download("table_id", false).await?;
    /// println!("Downloaded {} entries", stats.entries_downloaded);
    /// println!("Created {} locales, updated {} locales",
    ///          stats.locales_created, stats.locales_updated);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download(&self, table_id: &str, dry_run: bool) -> Result<DownloadStats> {
        let start = Instant::now();

        // Download from cloud
        let game_id = self
            .config
            .cloud
            .as_ref()
            .and_then(|c| c.game_id.as_deref());

        let entries = self
            .client
            .get_table_entries(table_id, game_id)
            .await
            .context("Failed to download translations")?;

        let entries_count = entries.len();

        // Convert to Translation format
        let translations = self.entries_to_translations(&entries);

        // Group by locale
        let mut by_locale: HashMap<String, Vec<Translation>> = HashMap::new();
        for translation in translations {
            by_locale
                .entry(translation.locale.clone())
                .or_default()
                .push(translation);
        }

        let mut locales_created = 0;
        let mut locales_updated = 0;

        if !dry_run {
            // Write translation files for each locale
            for (locale, locale_translations) in &by_locale {
                let file_path =
                    Path::new(&self.config.input_directory).join(format!("{}.json", locale));

                let existed = file_path.exists();

                self.write_translation_file(&file_path, locale_translations)?;

                if existed {
                    locales_updated += 1;
                } else {
                    locales_created += 1;
                }
            }
        } else {
            // In dry-run, count what would be created/updated
            for locale in by_locale.keys() {
                let file_path =
                    Path::new(&self.config.input_directory).join(format!("{}.json", locale));

                if file_path.exists() {
                    locales_updated += 1;
                } else {
                    locales_created += 1;
                }
            }
        }

        let duration = start.elapsed();

        Ok(DownloadStats {
            entries_downloaded: entries_count,
            locales_created,
            locales_updated,
            duration,
        })
    }

    /// Synchronize translations between local and cloud with merge strategy
    ///
    /// Performs bidirectional sync by comparing local and cloud translations,
    /// computing differences, and applying the specified merge strategy.
    ///
    /// # Merge Strategies
    ///
    /// - `Overwrite` - Upload all local translations (cloud is overwritten)
    /// - `Merge` - Upload local-only, download cloud-only, prefer cloud for conflicts
    /// - `SkipConflicts` - Upload local-only, download cloud-only, skip conflicts
    ///
    /// # Arguments
    ///
    /// * `table_id` - Roblox localization table ID
    /// * `strategy` - Merge strategy to use
    /// * `dry_run` - If true, skip all writes (preview only)
    ///
    /// # Returns
    ///
    /// Statistics about the sync operation including entries added/updated/deleted,
    /// conflicts skipped, and duration.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - API requests fail
    /// - Files cannot be read/written
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use roblox_slang::roblox::{RobloxCloudClient, SyncOrchestrator, MergeStrategy};
    /// # use roblox_slang::config::Config;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = RobloxCloudClient::new("api_key".to_string())?;
    /// # let config = Config::default();
    /// # let orchestrator = SyncOrchestrator::new(client, config);
    /// let stats = orchestrator.sync("table_id", MergeStrategy::Merge, false).await?;
    /// println!("Added: {}, Updated: {}, Conflicts: {}",
    ///          stats.entries_added, stats.entries_updated, stats.conflicts_skipped);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sync(
        &self,
        table_id: &str,
        strategy: MergeStrategy,
        dry_run: bool,
    ) -> Result<SyncStats> {
        let start = Instant::now();

        // Get local translations
        let local_translations = self.read_local_translations()?;
        let local_map = self.translations_to_map(&local_translations);

        // Get cloud translations
        let game_id = self
            .config
            .cloud
            .as_ref()
            .and_then(|c| c.game_id.as_deref());

        let cloud_entries = self
            .client
            .get_table_entries(table_id, game_id)
            .await
            .context("Failed to download translations")?;
        let cloud_translations = self.entries_to_translations(&cloud_entries);
        let cloud_map = self.translations_to_map(&cloud_translations);

        // Compute diff
        let diff = MergeEngine::compute_diff(&local_map, &cloud_map);

        // Apply strategy
        let merge_result = MergeEngine::apply_strategy(&diff, strategy, &local_map);

        let mut entries_added = 0;
        let mut entries_updated = 0;
        let entries_deleted = 0;

        if !dry_run {
            // Upload to_upload entries
            if !merge_result.to_upload.is_empty() {
                let upload_translations: Vec<Translation> = merge_result
                    .to_upload
                    .iter()
                    .map(|(key, locale, value)| Translation {
                        key: key.clone(),
                        locale: locale.clone(),
                        value: value.clone(),
                        context: None,
                    })
                    .collect();

                let entries = self.translations_to_entries(&upload_translations);

                let game_id = self
                    .config
                    .cloud
                    .as_ref()
                    .and_then(|c| c.game_id.as_deref());

                self.client
                    .update_table_entries(table_id, &entries, game_id)
                    .await
                    .context("Failed to upload translations")?;

                entries_added = merge_result.to_upload.len();
            }

            // Download to_download entries
            if !merge_result.to_download.is_empty() {
                let download_translations: Vec<Translation> = merge_result
                    .to_download
                    .iter()
                    .map(|(key, locale, value)| Translation {
                        key: key.clone(),
                        locale: locale.clone(),
                        value: value.clone(),
                        context: None,
                    })
                    .collect();

                // Group by locale and write files
                let mut by_locale: HashMap<String, Vec<Translation>> = HashMap::new();
                for translation in download_translations {
                    by_locale
                        .entry(translation.locale.clone())
                        .or_default()
                        .push(translation);
                }

                for (locale, locale_translations) in &by_locale {
                    let file_path =
                        Path::new(&self.config.input_directory).join(format!("{}.json", locale));

                    self.write_translation_file(&file_path, locale_translations)?;
                }

                entries_updated = merge_result.to_download.len();
            }

            // Write conflicts file if any
            if !merge_result.conflicts.is_empty() {
                self.write_conflicts_file(&merge_result.conflicts)?;
            }
        } else {
            // In dry-run, just count
            entries_added = merge_result.to_upload.len();
            entries_updated = merge_result.to_download.len();
        }

        let duration = start.elapsed();

        Ok(SyncStats {
            entries_added,
            entries_updated,
            entries_deleted,
            conflicts_skipped: merge_result.conflicts.len(),
            duration,
        })
    }

    // Helper methods

    /// Read all local translation files
    fn read_local_translations(&self) -> Result<Vec<Translation>> {
        let mut all_translations = Vec::new();

        for locale in &self.config.supported_locales {
            let file_path =
                Path::new(&self.config.input_directory).join(format!("{}.json", locale));

            if !file_path.exists() {
                continue;
            }

            let translations = parser::parse_json_file(&file_path, locale)
                .context(format!("Failed to parse {}", file_path.display()))?;

            all_translations.extend(translations);
        }

        Ok(all_translations)
    }

    /// Convert translations to LocalizationEntry format
    fn translations_to_entries(&self, translations: &[Translation]) -> Vec<LocalizationEntry> {
        use super::types::{EntryMetadata, Identifier, Translation as ApiTranslation};

        // Group by key
        let mut by_key: HashMap<String, Vec<&Translation>> = HashMap::new();
        for translation in translations {
            by_key
                .entry(translation.key.clone())
                .or_default()
                .push(translation);
        }

        // Convert to LocalizationEntry
        by_key
            .into_iter()
            .map(|(key, translations)| {
                let source = translations
                    .iter()
                    .find(|t| t.locale == self.config.base_locale)
                    .map(|t| t.value.clone())
                    .unwrap_or_else(|| translations[0].value.clone());

                LocalizationEntry {
                    identifier: Identifier {
                        key: key.clone(),
                        context: translations[0].context.clone(),
                        source,
                    },
                    metadata: Some(EntryMetadata {
                        example: None,
                        entry_type: Some("manual".to_string()),
                    }),
                    translations: translations
                        .iter()
                        .map(|t| ApiTranslation {
                            locale: t.locale.clone(),
                            translation_text: t.value.clone(),
                        })
                        .collect(),
                }
            })
            .collect()
    }

    /// Convert LocalizationEntry to Translation format
    fn entries_to_translations(&self, entries: &[LocalizationEntry]) -> Vec<Translation> {
        let mut translations = Vec::new();

        for entry in entries {
            // Add source text as base locale translation
            // Roblox Cloud doesn't return base locale translations when they match source text
            translations.push(Translation {
                key: entry.identifier.key.clone(),
                locale: self.config.base_locale.clone(),
                value: entry.identifier.source.clone(),
                context: entry.identifier.context.clone(),
            });

            // Add all other translations from API response
            for api_translation in &entry.translations {
                translations.push(Translation {
                    key: entry.identifier.key.clone(),
                    locale: api_translation.locale.clone(),
                    value: api_translation.translation_text.clone(),
                    context: entry.identifier.context.clone(),
                });
            }
        }

        translations
    }

    /// Convert translations to HashMap for merge engine
    fn translations_to_map(
        &self,
        translations: &[Translation],
    ) -> HashMap<(String, String), String> {
        translations
            .iter()
            .map(|t| ((t.key.clone(), t.locale.clone()), t.value.clone()))
            .collect()
    }

    /// Write translation file in nested JSON format
    fn write_translation_file(&self, path: &Path, translations: &[Translation]) -> Result<()> {
        use crate::utils::flatten::unflatten_translations;
        use std::fs;

        // Convert flat translations to nested structure
        let nested = unflatten_translations(translations);

        // Write to file
        let json =
            serde_json::to_string_pretty(&nested).context("Failed to serialize translations")?;

        fs::write(path, json).context(format!("Failed to write {}", path.display()))?;

        Ok(())
    }

    /// Write conflicts to YAML file
    fn write_conflicts_file(&self, conflicts: &[super::merge::Conflict]) -> Result<()> {
        use std::fs;

        // Group conflicts by locale
        let mut by_locale: HashMap<String, Vec<&super::merge::Conflict>> = HashMap::new();
        for conflict in conflicts {
            by_locale
                .entry(conflict.locale.clone())
                .or_default()
                .push(conflict);
        }

        // Create YAML structure
        let mut yaml_content = String::from("# Translation Conflicts\n");
        yaml_content.push_str("# Resolve these conflicts manually\n\n");

        for (locale, locale_conflicts) in by_locale {
            yaml_content.push_str(&format!("{}:\n", locale));
            for conflict in locale_conflicts {
                yaml_content.push_str(&format!("  {}:\n", conflict.key));
                yaml_content.push_str(&format!("    local: \"{}\"\n", conflict.local_value));
                yaml_content.push_str(&format!("    cloud: \"{}\"\n", conflict.cloud_value));
            }
            yaml_content.push('\n');
        }

        let conflicts_path = Path::new(&self.config.output_directory).join("conflicts.yaml");

        fs::write(&conflicts_path, yaml_content)
            .context(format!("Failed to write {}", conflicts_path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_orchestrator_new() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let _orchestrator = SyncOrchestrator::new(client, config);
    }

    #[test]
    fn test_translations_to_map() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                locale: "en".to_string(),
                value: "Buy".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                locale: "id".to_string(),
                value: "Beli".to_string(),
                context: None,
            },
        ];

        let map = orchestrator.translations_to_map(&translations);

        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get(&("ui.button".to_string(), "en".to_string())),
            Some(&"Buy".to_string())
        );
        assert_eq!(
            map.get(&("ui.button".to_string(), "id".to_string())),
            Some(&"Beli".to_string())
        );
    }

    #[test]
    fn test_translations_to_entries() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                locale: "en".to_string(),
                value: "Buy".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                locale: "es".to_string(),
                value: "Comprar".to_string(),
                context: None,
            },
        ];

        let entries = orchestrator.translations_to_entries(&translations);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].identifier.key, "ui.button");
        assert_eq!(entries[0].identifier.source, "Buy");
        assert_eq!(entries[0].translations.len(), 2);
    }

    #[test]
    fn test_translations_to_entries_with_context() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        let translations = vec![Translation {
            key: "ui.button".to_string(),
            locale: "en".to_string(),
            value: "Buy".to_string(),
            context: Some("shop".to_string()),
        }];

        let entries = orchestrator.translations_to_entries(&translations);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].identifier.context, Some("shop".to_string()));
    }

    #[test]
    fn test_entries_to_translations() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        use crate::roblox::types::{Identifier, Translation as ApiTranslation};

        let entries = vec![LocalizationEntry {
            identifier: Identifier {
                key: "ui.button".to_string(),
                context: None,
                source: "Buy".to_string(),
            },
            metadata: None,
            translations: vec![ApiTranslation {
                locale: "es".to_string(),
                translation_text: "Comprar".to_string(),
            }],
        }];

        let translations = orchestrator.entries_to_translations(&entries);

        // Should have 2 translations: base locale (from source) + es
        assert_eq!(translations.len(), 2);

        // First translation should be base locale from source
        assert_eq!(translations[0].key, "ui.button");
        assert_eq!(translations[0].locale, "en");
        assert_eq!(translations[0].value, "Buy");

        // Second translation should be from API response
        assert_eq!(translations[1].key, "ui.button");
        assert_eq!(translations[1].locale, "es");
        assert_eq!(translations[1].value, "Comprar");
    }

    #[test]
    fn test_entries_to_translations_multiple_locales() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        use crate::roblox::types::{Identifier, Translation as ApiTranslation};

        let entries = vec![LocalizationEntry {
            identifier: Identifier {
                key: "greeting".to_string(),
                context: None,
                source: "Hello".to_string(),
            },
            metadata: None,
            translations: vec![
                ApiTranslation {
                    locale: "es".to_string(),
                    translation_text: "Hola".to_string(),
                },
                ApiTranslation {
                    locale: "id".to_string(),
                    translation_text: "Halo".to_string(),
                },
            ],
        }];

        let translations = orchestrator.entries_to_translations(&entries);

        // Should have 3 translations: en (source) + es + id
        assert_eq!(translations.len(), 3);

        // Check base locale
        let en_translation = translations.iter().find(|t| t.locale == "en").unwrap();
        assert_eq!(en_translation.value, "Hello");

        // Check Spanish
        let es_translation = translations.iter().find(|t| t.locale == "es").unwrap();
        assert_eq!(es_translation.value, "Hola");

        // Check Indonesian
        let id_translation = translations.iter().find(|t| t.locale == "id").unwrap();
        assert_eq!(id_translation.value, "Halo");
    }

    #[test]
    fn test_entries_to_translations_with_context() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        use crate::roblox::types::{Identifier, Translation as ApiTranslation};

        let entries = vec![LocalizationEntry {
            identifier: Identifier {
                key: "ui.button".to_string(),
                context: Some("shop".to_string()),
                source: "Buy".to_string(),
            },
            metadata: None,
            translations: vec![ApiTranslation {
                locale: "es".to_string(),
                translation_text: "Comprar".to_string(),
            }],
        }];

        let translations = orchestrator.entries_to_translations(&entries);

        // All translations should have the same context
        for translation in &translations {
            assert_eq!(translation.context, Some("shop".to_string()));
        }
    }

    #[test]
    fn test_translations_to_map_empty() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        let translations = vec![];
        let map = orchestrator.translations_to_map(&translations);

        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_translations_to_map_multiple_keys() {
        let client = RobloxCloudClient::new("test_key".to_string()).unwrap();
        let config = Config::default();
        let orchestrator = SyncOrchestrator::new(client, config);

        let translations = vec![
            Translation {
                key: "ui.button".to_string(),
                locale: "en".to_string(),
                value: "Buy".to_string(),
                context: None,
            },
            Translation {
                key: "ui.title".to_string(),
                locale: "en".to_string(),
                value: "Shop".to_string(),
                context: None,
            },
            Translation {
                key: "ui.button".to_string(),
                locale: "es".to_string(),
                value: "Comprar".to_string(),
                context: None,
            },
        ];

        let map = orchestrator.translations_to_map(&translations);

        assert_eq!(map.len(), 3);
        assert_eq!(
            map.get(&("ui.button".to_string(), "en".to_string())),
            Some(&"Buy".to_string())
        );
        assert_eq!(
            map.get(&("ui.title".to_string(), "en".to_string())),
            Some(&"Shop".to_string())
        );
        assert_eq!(
            map.get(&("ui.button".to_string(), "es".to_string())),
            Some(&"Comprar".to_string())
        );
    }
}
