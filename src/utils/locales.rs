/// Roblox supported locales
/// Based on: <https://create.roblox.com/docs/production/localization/language-codes>
/// Roblox supported locale information
#[derive(Debug, Clone, PartialEq)]
pub struct LocaleInfo {
    pub code: &'static str,
    pub name: &'static str,
    pub native_name: &'static str,
}

/// Get all Roblox supported locales
pub fn get_roblox_locales() -> Vec<LocaleInfo> {
    vec![
        LocaleInfo {
            code: "en",
            name: "English",
            native_name: "English",
        },
        LocaleInfo {
            code: "es",
            name: "Spanish",
            native_name: "Español",
        },
        LocaleInfo {
            code: "fr",
            name: "French",
            native_name: "Français",
        },
        LocaleInfo {
            code: "de",
            name: "German",
            native_name: "Deutsch",
        },
        LocaleInfo {
            code: "pt",
            name: "Portuguese",
            native_name: "Português",
        },
        LocaleInfo {
            code: "id",
            name: "Indonesian",
            native_name: "Bahasa Indonesia",
        },
        LocaleInfo {
            code: "it",
            name: "Italian",
            native_name: "Italiano",
        },
        LocaleInfo {
            code: "ja",
            name: "Japanese",
            native_name: "日本語",
        },
        LocaleInfo {
            code: "ko",
            name: "Korean",
            native_name: "한국어",
        },
        LocaleInfo {
            code: "ru",
            name: "Russian",
            native_name: "Русский",
        },
        LocaleInfo {
            code: "th",
            name: "Thai",
            native_name: "ไทย",
        },
        LocaleInfo {
            code: "tr",
            name: "Turkish",
            native_name: "Türkçe",
        },
        LocaleInfo {
            code: "vi",
            name: "Vietnamese",
            native_name: "Tiếng Việt",
        },
        LocaleInfo {
            code: "pl",
            name: "Polish",
            native_name: "Polski",
        },
        LocaleInfo {
            code: "zh-cn",
            name: "Chinese (Simplified)",
            native_name: "简体中文",
        },
        LocaleInfo {
            code: "zh-tw",
            name: "Chinese (Traditional)",
            native_name: "繁體中文",
        },
        LocaleInfo {
            code: "uk",
            name: "Ukrainian",
            native_name: "Українська",
        },
    ]
}

/// Check if a locale is supported by Roblox
pub fn is_roblox_locale(code: &str) -> bool {
    get_roblox_locales()
        .iter()
        .any(|locale| locale.code == code)
}

/// Get all supported locale codes
pub fn get_supported_locale_codes() -> Vec<&'static str> {
    get_roblox_locales()
        .iter()
        .map(|locale| locale.code)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_roblox_locales() {
        let locales = get_roblox_locales();
        assert_eq!(locales.len(), 17);
    }

    #[test]
    fn test_is_roblox_locale() {
        assert!(is_roblox_locale("en"));
        assert!(is_roblox_locale("id"));
        assert!(is_roblox_locale("zh-cn"));
        assert!(!is_roblox_locale("xx"));
    }

    #[test]
    fn test_get_supported_locale_codes() {
        let codes = get_supported_locale_codes();
        assert_eq!(codes.len(), 17);
        assert!(codes.contains(&"en"));
        assert!(codes.contains(&"id"));
    }
}
