pub mod conflicts;
pub mod coverage;
pub mod missing;
pub mod unused;

#[derive(Debug, Clone)]
pub struct CoverageInfo {
    pub total_keys: usize,
    pub translated_keys: usize,
    pub missing_keys: Vec<String>,
    pub coverage_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_info_creation() {
        let info = CoverageInfo {
            total_keys: 100,
            translated_keys: 80,
            missing_keys: vec!["key1".to_string(), "key2".to_string()],
            coverage_percent: 80.0,
        };

        assert_eq!(info.total_keys, 100);
        assert_eq!(info.translated_keys, 80);
        assert_eq!(info.missing_keys.len(), 2);
        assert_eq!(info.coverage_percent, 80.0);
    }

    #[test]
    fn test_coverage_info_clone() {
        let info = CoverageInfo {
            total_keys: 50,
            translated_keys: 50,
            missing_keys: vec![],
            coverage_percent: 100.0,
        };

        let cloned = info.clone();
        assert_eq!(info.total_keys, cloned.total_keys);
        assert_eq!(info.coverage_percent, cloned.coverage_percent);
    }

    #[test]
    fn test_coverage_info_debug() {
        let info = CoverageInfo {
            total_keys: 10,
            translated_keys: 5,
            missing_keys: vec!["test".to_string()],
            coverage_percent: 50.0,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("CoverageInfo"));
        assert!(debug_str.contains("total_keys: 10"));
    }
}
