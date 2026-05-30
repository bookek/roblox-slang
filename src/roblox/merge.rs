use std::collections::HashMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    Overwrite,
    Merge,
    SkipConflicts,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub added_local: Vec<(String, String, String)>, // (key, locale, value)
    pub added_cloud: Vec<(String, String, String)>, // (key, locale, value)
    pub modified_both: Vec<(String, String, String, String)>, // (key, locale, local_value, cloud_value)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub key: String,
    pub locale: String,
    pub local_value: String,
    pub cloud_value: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResult {
    pub to_upload: Vec<(String, String, String)>, // (key, locale, value)
    pub to_download: Vec<(String, String, String)>, // (key, locale, value)
    pub conflicts: Vec<Conflict>,
}
pub struct MergeEngine;

impl MergeEngine {
    pub fn compute_diff(
        local: &HashMap<(String, String), String>,
        cloud: &HashMap<(String, String), String>,
    ) -> Diff {
        let mut added_local = Vec::new();
        let mut added_cloud = Vec::new();
        let mut modified_both = Vec::new();
        for ((key, locale), local_value) in local {
            if let Some(cloud_value) = cloud.get(&(key.clone(), locale.clone())) {
                if local_value != cloud_value {
                    modified_both.push((
                        key.clone(),
                        locale.clone(),
                        local_value.clone(),
                        cloud_value.clone(),
                    ));
                }
            } else {
                added_local.push((key.clone(), locale.clone(), local_value.clone()));
            }
        }
        for ((key, locale), cloud_value) in cloud {
            if !local.contains_key(&(key.clone(), locale.clone())) {
                added_cloud.push((key.clone(), locale.clone(), cloud_value.clone()));
            }
        }

        Diff {
            added_local,
            added_cloud,
            modified_both,
        }
    }
    pub fn apply_strategy(
        diff: &Diff,
        strategy: MergeStrategy,
        local: &HashMap<(String, String), String>,
    ) -> MergeResult {
        match strategy {
            MergeStrategy::Overwrite => Self::apply_overwrite(local),
            MergeStrategy::Merge => Self::apply_merge(diff),
            MergeStrategy::SkipConflicts => Self::apply_skip_conflicts(diff),
        }
    }
    fn apply_overwrite(local: &HashMap<(String, String), String>) -> MergeResult {
        let to_upload: Vec<(String, String, String)> = local
            .iter()
            .map(|((key, locale), value)| (key.clone(), locale.clone(), value.clone()))
            .collect();

        MergeResult {
            to_upload,
            to_download: Vec::new(),
            conflicts: Vec::new(),
        }
    }
    fn apply_merge(diff: &Diff) -> MergeResult {
        let to_upload = diff.added_local.clone();
        let mut to_download = diff.added_cloud.clone();
        for (key, locale, _local_value, cloud_value) in &diff.modified_both {
            to_download.push((key.clone(), locale.clone(), cloud_value.clone()));
        }

        MergeResult {
            to_upload,
            to_download,
            conflicts: Vec::new(),
        }
    }
    fn apply_skip_conflicts(diff: &Diff) -> MergeResult {
        let to_upload = diff.added_local.clone();
        let to_download = diff.added_cloud.clone();
        let conflicts: Vec<Conflict> = diff
            .modified_both
            .iter()
            .map(|(key, locale, local_value, cloud_value)| Conflict {
                key: key.clone(),
                locale: locale.clone(),
                local_value: local_value.clone(),
                cloud_value: cloud_value.clone(),
            })
            .collect();

        MergeResult {
            to_upload,
            to_download,
            conflicts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_local() -> HashMap<(String, String), String> {
        let mut local = HashMap::new();
        local.insert(
            ("ui.button.buy".to_string(), "en".to_string()),
            "Buy".to_string(),
        );
        local.insert(
            ("ui.button.sell".to_string(), "en".to_string()),
            "Sell".to_string(),
        );
        local.insert(
            ("ui.button.buy".to_string(), "id".to_string()),
            "Beli".to_string(),
        );
        local
    }

    fn create_test_cloud() -> HashMap<(String, String), String> {
        let mut cloud = HashMap::new();
        cloud.insert(
            ("ui.button.buy".to_string(), "en".to_string()),
            "Purchase".to_string(), // Conflict
        );
        cloud.insert(
            ("ui.button.cancel".to_string(), "en".to_string()),
            "Cancel".to_string(), // Only in cloud
        );
        cloud
    }

    #[test]
    fn test_compute_diff() {
        let local = create_test_local();
        let cloud = create_test_cloud();

        let diff = MergeEngine::compute_diff(&local, &cloud);
        assert_eq!(diff.added_local.len(), 2);
        assert!(diff.added_local.contains(&(
            "ui.button.sell".to_string(),
            "en".to_string(),
            "Sell".to_string()
        )));
        assert!(diff.added_local.contains(&(
            "ui.button.buy".to_string(),
            "id".to_string(),
            "Beli".to_string()
        )));
        assert_eq!(diff.added_cloud.len(), 1);
        assert!(diff.added_cloud.contains(&(
            "ui.button.cancel".to_string(),
            "en".to_string(),
            "Cancel".to_string()
        )));
        assert_eq!(diff.modified_both.len(), 1);
        assert_eq!(diff.modified_both[0].0, "ui.button.buy");
        assert_eq!(diff.modified_both[0].1, "en");
        assert_eq!(diff.modified_both[0].2, "Buy");
        assert_eq!(diff.modified_both[0].3, "Purchase");
    }

    #[test]
    fn test_overwrite_strategy() {
        let local = create_test_local();
        let cloud = create_test_cloud();

        let diff = MergeEngine::compute_diff(&local, &cloud);
        let result = MergeEngine::apply_strategy(&diff, MergeStrategy::Overwrite, &local);
        assert_eq!(result.to_upload.len(), 3);
        assert_eq!(result.to_download.len(), 0);
        assert_eq!(result.conflicts.len(), 0);
    }

    #[test]
    fn test_merge_strategy() {
        let local = create_test_local();
        let cloud = create_test_cloud();

        let diff = MergeEngine::compute_diff(&local, &cloud);
        let result = MergeEngine::apply_strategy(&diff, MergeStrategy::Merge, &local);
        assert_eq!(result.to_upload.len(), 2);
        assert_eq!(result.to_download.len(), 2);
        assert_eq!(result.conflicts.len(), 0);
    }

    #[test]
    fn test_skip_conflicts_strategy() {
        let local = create_test_local();
        let cloud = create_test_cloud();

        let diff = MergeEngine::compute_diff(&local, &cloud);
        let result = MergeEngine::apply_strategy(&diff, MergeStrategy::SkipConflicts, &local);
        assert_eq!(result.to_upload.len(), 2);
        assert_eq!(result.to_download.len(), 1);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].key, "ui.button.buy");
        assert_eq!(result.conflicts[0].locale, "en");
        assert_eq!(result.conflicts[0].local_value, "Buy");
        assert_eq!(result.conflicts[0].cloud_value, "Purchase");
    }

    #[test]
    fn test_empty_diff() {
        let local = HashMap::new();
        let cloud = HashMap::new();

        let diff = MergeEngine::compute_diff(&local, &cloud);

        assert_eq!(diff.added_local.len(), 0);
        assert_eq!(diff.added_cloud.len(), 0);
        assert_eq!(diff.modified_both.len(), 0);
    }

    #[test]
    fn test_identical_translations() {
        let mut local = HashMap::new();
        local.insert(
            ("ui.button".to_string(), "en".to_string()),
            "Buy".to_string(),
        );

        let cloud = local.clone();

        let diff = MergeEngine::compute_diff(&local, &cloud);
        assert_eq!(diff.added_local.len(), 0);
        assert_eq!(diff.added_cloud.len(), 0);
        assert_eq!(diff.modified_both.len(), 0);
    }
}
