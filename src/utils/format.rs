use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatSpecifier {
    Int,
    Fixed(usize),
    Num,
    DateTime,
    Translate,
    None,
}

pub fn parse_format_specifier(param: &str) -> (String, FormatSpecifier) {
    if let Some(colon_pos) = param.find(':') {
        let name = param[..colon_pos].trim().to_string();
        let spec = param[colon_pos + 1..].trim();

        let format = match spec {
            "int" => FormatSpecifier::Int,
            "num" => FormatSpecifier::Num,
            "datetime" => FormatSpecifier::DateTime,
            "translate" => FormatSpecifier::Translate,
            s if s.starts_with("fixed(") && s.ends_with(')') => {
                let digits_str = &s[6..s.len() - 1];
                let digits = digits_str.parse::<usize>().unwrap_or(2);
                FormatSpecifier::Fixed(digits)
            }
            _ => FormatSpecifier::None,
        };

        (name, format)
    } else {
        (param.trim().to_string(), FormatSpecifier::None)
    }
}

pub fn extract_parameters_with_format(text: &str) -> HashMap<String, FormatSpecifier> {
    let mut params = HashMap::new();
    let mut in_param = false;
    let mut current_param = String::new();

    for ch in text.chars() {
        match ch {
            '{' => {
                in_param = true;
                current_param.clear();
            }
            '}' => {
                if in_param && !current_param.is_empty() {
                    let (name, format) = parse_format_specifier(&current_param);
                    params.insert(name, format);
                }
                in_param = false;
            }
            _ => {
                if in_param {
                    current_param.push(ch);
                }
            }
        }
    }

    params
}

pub fn generate_format_code(param_name: &str, specifier: &FormatSpecifier) -> String {
    match specifier {
        FormatSpecifier::Int => {
            format!(
                "params.{} = math.floor(tonumber(params.{}) or 0)",
                param_name, param_name
            )
        }
        FormatSpecifier::Fixed(digits) => {
            format!(
                "params.{} = string.format(\"%.{}f\", tonumber(params.{}) or 0)",
                param_name, digits, param_name
            )
        }
        FormatSpecifier::Num => {
            format!(
                "params.{} = tostring(params.{} or 0)",
                param_name, param_name
            )
        }
        FormatSpecifier::DateTime => {
            format!(
                "if typeof(params.{}) == \"DateTime\" then\n        params.{} = params.{}:FormatLocalTime(\"L LT\", \"en-us\")\n    end",
                param_name, param_name, param_name
            )
        }
        FormatSpecifier::Translate => {
            format!(
                "if type(params.{}) == \"string\" then\n        params.{} = self._translator:FormatByKey(params.{})\n    end",
                param_name, param_name, param_name
            )
        }
        FormatSpecifier::None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_format_specifier() {
        let (name, spec) = parse_format_specifier("count:int");
        assert_eq!(name, "count");
        assert_eq!(spec, FormatSpecifier::Int);
    }

    #[test]
    fn test_parse_fixed_specifier() {
        let (name, spec) = parse_format_specifier("price:fixed(2)");
        assert_eq!(name, "price");
        assert_eq!(spec, FormatSpecifier::Fixed(2));
    }

    #[test]
    fn test_parse_no_specifier() {
        let (name, spec) = parse_format_specifier("name");
        assert_eq!(name, "name");
        assert_eq!(spec, FormatSpecifier::None);
    }

    #[test]
    fn test_parse_num_specifier() {
        let (name, spec) = parse_format_specifier("value:num");
        assert_eq!(name, "value");
        assert_eq!(spec, FormatSpecifier::Num);
    }

    #[test]
    fn test_parse_datetime_specifier() {
        let (name, spec) = parse_format_specifier("date:datetime");
        assert_eq!(name, "date");
        assert_eq!(spec, FormatSpecifier::DateTime);
    }

    #[test]
    fn test_parse_translate_specifier() {
        let (name, spec) = parse_format_specifier("label:translate");
        assert_eq!(name, "label");
        assert_eq!(spec, FormatSpecifier::Translate);
    }

    #[test]
    fn test_parse_fixed_with_different_digits() {
        let (name, spec) = parse_format_specifier("price:fixed(4)");
        assert_eq!(name, "price");
        assert_eq!(spec, FormatSpecifier::Fixed(4));

        let (name2, spec2) = parse_format_specifier("amount:fixed(0)");
        assert_eq!(name2, "amount");
        assert_eq!(spec2, FormatSpecifier::Fixed(0));
    }

    #[test]
    fn test_parse_invalid_fixed_defaults_to_2() {
        let (name, spec) = parse_format_specifier("price:fixed(abc)");
        assert_eq!(name, "price");
        assert_eq!(spec, FormatSpecifier::Fixed(2));
    }

    #[test]
    fn test_parse_unknown_specifier() {
        let (name, spec) = parse_format_specifier("value:unknown");
        assert_eq!(name, "value");
        assert_eq!(spec, FormatSpecifier::None);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let (name, spec) = parse_format_specifier("  count : int  ");
        assert_eq!(name, "count");
        assert_eq!(spec, FormatSpecifier::Int);
    }

    #[test]
    fn test_extract_parameters_with_format() {
        let text = "Price: {price:fixed(2)}, Count: {count:int}";
        let params = extract_parameters_with_format(text);

        assert_eq!(params.get("price"), Some(&FormatSpecifier::Fixed(2)));
        assert_eq!(params.get("count"), Some(&FormatSpecifier::Int));
    }

    #[test]
    fn test_extract_multiple_same_param() {
        let text = "Value: {count:int}, Again: {count:int}";
        let params = extract_parameters_with_format(text);
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("count"), Some(&FormatSpecifier::Int));
    }

    #[test]
    fn test_extract_no_parameters() {
        let text = "No parameters here";
        let params = extract_parameters_with_format(text);

        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_extract_empty_braces() {
        let text = "Empty: {}";
        let params = extract_parameters_with_format(text);

        assert_eq!(params.len(), 0); // Empty braces are ignored
    }

    #[test]
    fn test_extract_nested_braces() {
        let text = "Nested: {outer{inner}}";
        let params = extract_parameters_with_format(text);
        assert_eq!(params.len(), 1);
        assert!(params.contains_key("inner"));
        assert_eq!(params.get("inner"), Some(&FormatSpecifier::None));
    }

    #[test]
    fn test_extract_all_specifier_types() {
        let text = "Int: {a:int}, Fixed: {b:fixed(3)}, Num: {c:num}, DateTime: {d:datetime}, Translate: {e:translate}, None: {f}";
        let params = extract_parameters_with_format(text);

        assert_eq!(params.len(), 6);
        assert_eq!(params.get("a"), Some(&FormatSpecifier::Int));
        assert_eq!(params.get("b"), Some(&FormatSpecifier::Fixed(3)));
        assert_eq!(params.get("c"), Some(&FormatSpecifier::Num));
        assert_eq!(params.get("d"), Some(&FormatSpecifier::DateTime));
        assert_eq!(params.get("e"), Some(&FormatSpecifier::Translate));
        assert_eq!(params.get("f"), Some(&FormatSpecifier::None));
    }

    #[test]
    fn test_generate_format_code_int() {
        let code = generate_format_code("count", &FormatSpecifier::Int);
        assert!(code.contains("math.floor"));
        assert!(code.contains("tonumber"));
        assert!(code.contains("params.count"));
    }

    #[test]
    fn test_generate_format_code_fixed() {
        let code = generate_format_code("price", &FormatSpecifier::Fixed(2));
        assert!(code.contains("string.format"));
        assert!(code.contains("%.2f"));
        assert!(code.contains("params.price"));
    }

    #[test]
    fn test_generate_format_code_num() {
        let code = generate_format_code("value", &FormatSpecifier::Num);
        assert!(code.contains("tostring"));
        assert!(code.contains("params.value"));
    }

    #[test]
    fn test_generate_format_code_datetime() {
        let code = generate_format_code("date", &FormatSpecifier::DateTime);
        assert!(code.contains("DateTime"));
        assert!(code.contains("FormatLocalTime"));
        assert!(code.contains("params.date"));
    }

    #[test]
    fn test_generate_format_code_translate() {
        let code = generate_format_code("label", &FormatSpecifier::Translate);
        assert!(code.contains("FormatByKey"));
        assert!(code.contains("params.label"));
    }

    #[test]
    fn test_generate_format_code_none() {
        let code = generate_format_code("name", &FormatSpecifier::None);
        assert_eq!(code, "");
    }
}
