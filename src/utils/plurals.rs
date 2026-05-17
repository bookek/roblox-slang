pub fn is_plural_key(key: &str) -> bool {
    key.ends_with("(zero)")
        || key.ends_with("(one)")
        || key.ends_with("(two)")
        || key.ends_with("(few)")
        || key.ends_with("(many)")
        || key.ends_with("(other)")
}

pub fn extract_base_key(key: &str) -> String {
    if let Some(pos) = key.rfind('(') {
        key[..pos].to_string()
    } else {
        key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_plural_key() {
        assert!(is_plural_key("items(one)"));
        assert!(is_plural_key("items(other)"));
        assert!(!is_plural_key("items"));
    }

    #[test]
    fn test_is_plural_key_all_forms() {
        assert!(is_plural_key("count(zero)"));
        assert!(is_plural_key("count(one)"));
        assert!(is_plural_key("count(two)"));
        assert!(is_plural_key("count(few)"));
        assert!(is_plural_key("count(many)"));
        assert!(is_plural_key("count(other)"));
    }

    #[test]
    fn test_is_plural_key_nested() {
        assert!(is_plural_key("ui.messages.items(one)"));
        assert!(is_plural_key("shop.cart.products(other)"));
    }

    #[test]
    fn test_is_plural_key_false_cases() {
        assert!(!is_plural_key("items"));
        assert!(!is_plural_key("items()"));
        assert!(!is_plural_key("items(invalid)"));
    }

    #[test]
    fn test_extract_base_key() {
        assert_eq!(extract_base_key("items(one)"), "items");
        assert_eq!(
            extract_base_key("ui.messages.count(other)"),
            "ui.messages.count"
        );
    }

    #[test]
    fn test_extract_base_key_all_forms() {
        assert_eq!(extract_base_key("count(zero)"), "count");
        assert_eq!(extract_base_key("count(one)"), "count");
        assert_eq!(extract_base_key("count(two)"), "count");
        assert_eq!(extract_base_key("count(few)"), "count");
        assert_eq!(extract_base_key("count(many)"), "count");
        assert_eq!(extract_base_key("count(other)"), "count");
    }

    #[test]
    fn test_extract_base_key_no_plural() {
        assert_eq!(extract_base_key("items"), "items");
        assert_eq!(extract_base_key("ui.button.buy"), "ui.button.buy");
    }

    #[test]
    fn test_extract_base_key_nested_parentheses() {
        assert_eq!(extract_base_key("func(arg)(one)"), "func(arg)");
    }

    #[test]
    fn test_extract_base_key_empty() {
        assert_eq!(extract_base_key(""), "");
    }
}
