use super::client::RobloxCloudClient;
use super::merge::{MergeEngine, MergeStrategy};
use super::types::{DownloadStats, LocalizationEntry, SyncStats, UploadStats};
use crate::config::Config;
use crate::parser::{self, Translation};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
fn merge_locale_translations(
    mut existing: Vec<Translation>,
    incoming: &[Translation],
) -> Vec<Translation> {
    for new_t in incoming {
        if let Some(entry) = existing.iter_mut().find(|t| t.key == new_t.key) {
            entry.value = new_t.value.clone();
            entry.context = new_t.context.clone();
        } else {
            existing.push(new_t.clone());
        }
    }
    existing
}
pub struct SyncOrchestrator {
    client: RobloxCloudClient,
    config: Config,
}

impl SyncOrchestrator {
    pub fn new(client: RobloxCloudClient, config: Config) -> Self {
        Self { client, config }
    }
    pub async fn upload(&self, table_id: &str, dry_run: bool) -> Result<UploadStats> {
        let start = Instant::now();
        let translations = self.read_local_translations()?;

        let entries_count = translations.len();
        let locales: std::collections::HashSet<_> =
            translations.iter().map(|t| t.locale.clone()).collect();
        let locales_count = locales.len();

        if !dry_run {
            let entries = self.translations_to_entries(&translations);
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
    pub async fn download(&self, table_id: &str, dry_run: bool) -> Result<DownloadStats> {
        let start = Instant::now();
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
        let translations = self.entries_to_translations(&entries);
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
    pub async fn sync(
        &self,
        table_id: &str,
        strategy: MergeStrategy,
        dry_run: bool,
    ) -> Result<SyncStats> {
        let start = Instant::now();
        let local_translations = self.read_local_translations()?;
        let local_map = self.translations_to_map(&local_translations);
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
        let diff = MergeEngine::compute_diff(&local_map, &cloud_map);
        let merge_result = MergeEngine::apply_strategy(&diff, strategy, &local_map);

        let mut entries_added = 0;
        let mut entries_updated = 0;
        let entries_deleted = 0;

        if !dry_run {
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
                let mut by_locale: HashMap<String, Vec<Translation>> = HashMap::new();
                for translation in download_translations {
                    by_locale
                        .entry(translation.locale.clone())
                        .or_default()
                        .push(translation);
                }

                for (locale, incoming) in &by_locale {
                    let file_path =
                        Path::new(&self.config.input_directory).join(format!("{}.json", locale));

                    let merged = if file_path.exists() {
                        let existing =
                            parser::parse_json_file(&file_path, locale).with_context(|| {
                                format!(
                                    "Failed to parse existing {} for merge",
                                    file_path.display()
                                )
                            })?;
                        merge_locale_translations(existing, incoming)
                    } else {
                        incoming.clone()
                    };

                    self.write_translation_file(&file_path, &merged)?;
                }

                entries_updated = merge_result.to_download.len();
            }
            if !merge_result.conflicts.is_empty() {
                self.write_conflicts_file(&merge_result.conflicts)?;
            }
        } else {
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
    fn translations_to_entries(&self, translations: &[Translation]) -> Vec<LocalizationEntry> {
        use super::types::{EntryMetadata, Identifier, Translation as ApiTranslation};
        let mut by_key: HashMap<String, Vec<&Translation>> = HashMap::new();
        for translation in translations {
            by_key
                .entry(translation.key.clone())
                .or_default()
                .push(translation);
        }
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
    fn entries_to_translations(&self, entries: &[LocalizationEntry]) -> Vec<Translation> {
        let mut translations = Vec::new();

        for entry in entries {
            translations.push(Translation {
                key: entry.identifier.key.clone(),
                locale: self.config.base_locale.clone(),
                value: entry.identifier.source.clone(),
                context: entry.identifier.context.clone(),
            });
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
    fn translations_to_map(
        &self,
        translations: &[Translation],
    ) -> HashMap<(String, String), String> {
        translations
            .iter()
            .map(|t| ((t.key.clone(), t.locale.clone()), t.value.clone()))
            .collect()
    }
    fn write_translation_file(&self, path: &Path, translations: &[Translation]) -> Result<()> {
        use crate::utils::flatten::unflatten_translations;
        use std::fs;
        let nested = unflatten_translations(translations);
        let json =
            serde_json::to_string_pretty(&nested).context("Failed to serialize translations")?;

        fs::write(path, json).context(format!("Failed to write {}", path.display()))?;

        Ok(())
    }
    fn write_conflicts_file(&self, conflicts: &[super::merge::Conflict]) -> Result<()> {
        use std::fs;
        let mut by_locale: HashMap<String, Vec<&super::merge::Conflict>> = HashMap::new();
        for conflict in conflicts {
            by_locale
                .entry(conflict.locale.clone())
                .or_default()
                .push(conflict);
        }
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
        assert_eq!(translations.len(), 2);
        assert_eq!(translations[0].key, "ui.button");
        assert_eq!(translations[0].locale, "en");
        assert_eq!(translations[0].value, "Buy");
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
        assert_eq!(translations.len(), 3);
        let en_translation = translations.iter().find(|t| t.locale == "en").unwrap();
        assert_eq!(en_translation.value, "Hello");
        let es_translation = translations.iter().find(|t| t.locale == "es").unwrap();
        assert_eq!(es_translation.value, "Hola");
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

    #[test]
    fn test_merge_locale_translations_preserves_local_keys() {
        let existing = vec![
            Translation {
                key: "greeting".to_string(),
                locale: "en".to_string(),
                value: "Hello".to_string(),
                context: None,
            },
            Translation {
                key: "farewell".to_string(),
                locale: "en".to_string(),
                value: "Goodbye".to_string(),
                context: None,
            },
        ];
        let incoming = vec![Translation {
            key: "greeting".to_string(),
            locale: "en".to_string(),
            value: "Hi there".to_string(),
            context: None,
        }];

        let merged = merge_locale_translations(existing, &incoming);

        assert_eq!(merged.len(), 2, "must preserve both keys");
        assert_eq!(
            merged.iter().find(|t| t.key == "greeting").unwrap().value,
            "Hi there",
            "greeting must be updated"
        );
        assert_eq!(
            merged.iter().find(|t| t.key == "farewell").unwrap().value,
            "Goodbye",
            "farewell must be preserved"
        );
    }

    #[test]
    fn test_merge_locale_translations_adds_new_keys() {
        let existing = vec![Translation {
            key: "greeting".to_string(),
            locale: "en".to_string(),
            value: "Hello".to_string(),
            context: None,
        }];

        let incoming = vec![Translation {
            key: "new_key".to_string(),
            locale: "en".to_string(),
            value: "New Value".to_string(),
            context: Some("added by cloud".to_string()),
        }];

        let merged = merge_locale_translations(existing, &incoming);

        assert_eq!(merged.len(), 2);
        let new = merged.iter().find(|t| t.key == "new_key").unwrap();
        assert_eq!(new.value, "New Value");
        assert_eq!(new.context, Some("added by cloud".to_string()));
    }

    #[test]
    fn test_merge_locale_translations_empty_existing() {
        let existing = vec![];

        let incoming = vec![Translation {
            key: "greeting".to_string(),
            locale: "en".to_string(),
            value: "Hello".to_string(),
            context: None,
        }];

        let merged = merge_locale_translations(existing, &incoming);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].key, "greeting");
    }

    #[test]
    fn test_merge_locale_translations_empty_incoming() {
        let existing = vec![Translation {
            key: "greeting".to_string(),
            locale: "en".to_string(),
            value: "Hello".to_string(),
            context: None,
        }];

        let incoming: Vec<Translation> = vec![];

        let merged = merge_locale_translations(existing, &incoming);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].value, "Hello");
    }
}
