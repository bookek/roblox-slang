use super::KeyTransform;
pub fn transform_key(key: &str, strategy: KeyTransform) -> String {
    match strategy {
        KeyTransform::SnakeToCamel => snake_to_camel(key),
        KeyTransform::UpperToLower => key.to_lowercase(),
        KeyTransform::DotToNested => key.to_string(), // Already handled by unflatten
        KeyTransform::None => key.to_string(),
    }
}
fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(snake_to_camel("ui_button_buy"), "uiButtonBuy");
        assert_eq!(snake_to_camel("simple"), "simple");
    }

    #[test]
    fn test_snake_to_camel_multiple_underscores() {
        assert_eq!(snake_to_camel("hello__world"), "helloWorld");
        assert_eq!(snake_to_camel("a_b_c_d"), "aBCD");
    }

    #[test]
    fn test_snake_to_camel_leading_underscore() {
        assert_eq!(snake_to_camel("_hello"), "Hello");
        assert_eq!(snake_to_camel("__hello"), "Hello");
    }

    #[test]
    fn test_snake_to_camel_trailing_underscore() {
        assert_eq!(snake_to_camel("hello_"), "hello");
        assert_eq!(snake_to_camel("hello__"), "hello");
    }

    #[test]
    fn test_snake_to_camel_empty() {
        assert_eq!(snake_to_camel(""), "");
    }

    #[test]
    fn test_snake_to_camel_no_underscores() {
        assert_eq!(snake_to_camel("hello"), "hello");
        assert_eq!(snake_to_camel("HELLO"), "HELLO");
    }

    #[test]
    fn test_transform_key_snake_to_camel() {
        assert_eq!(
            transform_key("hello_world", KeyTransform::SnakeToCamel),
            "helloWorld"
        );
    }

    #[test]
    fn test_transform_key_upper_to_lower() {
        assert_eq!(
            transform_key("HELLO_WORLD", KeyTransform::UpperToLower),
            "hello_world"
        );
        assert_eq!(
            transform_key("MixedCase", KeyTransform::UpperToLower),
            "mixedcase"
        );
    }

    #[test]
    fn test_transform_key_dot_to_nested() {
        assert_eq!(
            transform_key("ui.button.buy", KeyTransform::DotToNested),
            "ui.button.buy"
        );
    }

    #[test]
    fn test_transform_key_none() {
        assert_eq!(
            transform_key("hello_world", KeyTransform::None),
            "hello_world"
        );
        assert_eq!(
            transform_key("HELLO_WORLD", KeyTransform::None),
            "HELLO_WORLD"
        );
    }

    #[test]
    fn test_transform_key_empty() {
        assert_eq!(transform_key("", KeyTransform::SnakeToCamel), "");
        assert_eq!(transform_key("", KeyTransform::UpperToLower), "");
        assert_eq!(transform_key("", KeyTransform::None), "");
    }

    #[test]
    fn test_transform_key_special_chars() {
        assert_eq!(
            transform_key("hello-world", KeyTransform::SnakeToCamel),
            "hello-world"
        );
        assert_eq!(
            transform_key("hello.world", KeyTransform::SnakeToCamel),
            "hello.world"
        );
    }
}
