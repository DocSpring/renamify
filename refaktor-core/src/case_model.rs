use bstr::ByteSlice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Style {
    Snake,
    Kebab,
    Camel,
    Pascal,
    ScreamingSnake,
    Title,
    Train,
    Dot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub text: String,
}

impl Token {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenModel {
    pub tokens: Vec<Token>,
}

impl TokenModel {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
}

pub fn detect_style(s: &str) -> Option<Style> {
    if s.is_empty() {
        return None;
    }

    let has_underscore = s.contains('_');
    let has_hyphen = s.contains('-');
    // Only consider it dot style if the dot is in the middle, not at the start
    let has_dot = s.contains('.') && !s.starts_with('.');
    let has_space = s.contains(' ');
    let has_upper = s.bytes().any(|b| b.is_ascii_uppercase());
    let has_lower = s.bytes().any(|b| b.is_ascii_lowercase());

    match (
        has_underscore,
        has_hyphen,
        has_dot,
        has_space,
        has_upper,
        has_lower,
    ) {
        (true, false, false, false, false, true) => Some(Style::Snake),
        (true, false, false, false, true, false) => Some(Style::ScreamingSnake),
        (false, true, false, false, false, true) => Some(Style::Kebab),
        (false, true, false, false, true, true) => {
            if is_train_case(s) {
                Some(Style::Train)
            } else {
                None
            }
        },
        (false, false, true, false, _, true) => Some(Style::Dot),
        (false, false, false, true, true, true) => {
            if is_title_case(s) {
                Some(Style::Title)
            } else {
                None
            }
        },
        (false, false, false, false, true, true) => {
            if s.bytes().next().map_or(false, |b| b.is_ascii_uppercase()) {
                Some(Style::Pascal)
            } else if s.bytes().next().map_or(false, |b| b.is_ascii_lowercase()) {
                Some(Style::Camel)
            } else {
                None
            }
        },
        _ => None,
    }
}

fn is_train_case(s: &str) -> bool {
    s.split('-').all(|word| {
        word.len() > 0
            && word
                .bytes()
                .next()
                .map_or(false, |b| b.is_ascii_uppercase())
            && word.bytes().skip(1).all(|b| b.is_ascii_lowercase())
    })
}

fn is_title_case(s: &str) -> bool {
    s.split(' ').all(|word| {
        word.len() > 0
            && word
                .bytes()
                .next()
                .map_or(false, |b| b.is_ascii_uppercase())
            && word.bytes().skip(1).all(|b| b.is_ascii_lowercase())
    })
}

pub fn parse_to_tokens(s: &str) -> TokenModel {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut current = Vec::new();

    for i in 0..bytes.len() {
        let b = bytes[i];

        if b == b'_' || b == b'-' || b == b'.' || b == b' ' {
            if !current.is_empty() {
                tokens.push(Token::new(
                    std::str::from_utf8(&current).unwrap_or_default(),
                ));
                current.clear();
            }
        } else if b.is_ascii_alphabetic() || b.is_ascii_digit() {
            if i > 0 && !current.is_empty() {
                let prev = bytes[i - 1];

                let should_split = (prev.is_ascii_lowercase() && b.is_ascii_uppercase())
                    || (prev.is_ascii_alphabetic() && b.is_ascii_digit())
                    || (prev.is_ascii_digit() && b.is_ascii_alphabetic())
                    || (i > 0
                        && prev.is_ascii_uppercase()
                        && b.is_ascii_uppercase()
                        && i + 1 < bytes.len()
                        && bytes[i + 1].is_ascii_lowercase());

                if should_split {
                    tokens.push(Token::new(
                        std::str::from_utf8(&current).unwrap_or_default(),
                    ));
                    current.clear();
                }
            }
            current.push(b);
        }
    }

    if !current.is_empty() {
        tokens.push(Token::new(
            std::str::from_utf8(&current).unwrap_or_default(),
        ));
    }

    TokenModel::new(tokens)
}

pub fn to_style(model: &TokenModel, style: Style) -> String {
    if model.tokens.is_empty() {
        return String::new();
    }

    match style {
        Style::Snake => model
            .tokens
            .iter()
            .map(|t| t.text.to_lowercase())
            .collect::<Vec<_>>()
            .join("_"),

        Style::Kebab => model
            .tokens
            .iter()
            .map(|t| t.text.to_lowercase())
            .collect::<Vec<_>>()
            .join("-"),

        Style::Camel => {
            let mut result = String::new();
            for (i, token) in model.tokens.iter().enumerate() {
                if i == 0 {
                    result.push_str(&token.text.to_lowercase());
                } else {
                    result.push_str(&capitalize_first(&token.text));
                }
            }
            result
        },

        Style::Pascal => model
            .tokens
            .iter()
            .map(|t| capitalize_first(&t.text))
            .collect::<Vec<_>>()
            .join(""),

        Style::ScreamingSnake => model
            .tokens
            .iter()
            .map(|t| t.text.to_uppercase())
            .collect::<Vec<_>>()
            .join("_"),

        Style::Title => model
            .tokens
            .iter()
            .map(|t| capitalize_first(&t.text))
            .collect::<Vec<_>>()
            .join(" "),

        Style::Train => model
            .tokens
            .iter()
            .map(|t| capitalize_first(&t.text))
            .collect::<Vec<_>>()
            .join("-"),

        Style::Dot => model
            .tokens
            .iter()
            .map(|t| t.text.to_lowercase())
            .collect::<Vec<_>>()
            .join("."),
    }
}

fn capitalize_first(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    if s.bytes().all(|b| b.is_ascii_uppercase()) && s.len() <= 2 {
        return s.to_string();
    }

    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

pub fn generate_variant_map(
    old: &str,
    new: &str,
    styles: Option<&[Style]>,
) -> BTreeMap<String, String> {
    let default_styles = [
        Style::Snake,
        Style::Kebab,
        Style::Camel,
        Style::Pascal,
        Style::ScreamingSnake,
    ];
    let styles = styles.unwrap_or(&default_styles);

    let old_tokens = parse_to_tokens(old);
    let new_tokens = parse_to_tokens(new);

    let mut map = BTreeMap::new();

    // Detect the original pattern's style
    let original_style = detect_style(old);

    // Check if the original pattern should be included
    let include_original = if let Some(orig_style) = original_style {
        // If we have explicit styles, only include original if its style is in the list
        styles.iter().any(|&s| s == orig_style)
    } else {
        // If original style can't be detected, include it anyway (backwards compat)
        true
    };

    // Add the original pattern if its style is not excluded
    if include_original {
        map.insert(old.to_string(), new.to_string());
    }

    for style in styles {
        let old_variant = to_style(&old_tokens, *style);
        let new_variant = to_style(&new_tokens, *style);

        // Skip if this variant is the same as the original (to avoid duplicates)
        if old_variant != old {
            map.insert(old_variant, new_variant);
        }
    }

    // Add case variants (lowercase and uppercase) but only if:
    // 1. They're different from the original
    // 2. They're not already in the map
    // 3. The original is included (or they're genuinely different)
    let lower_old = old.to_lowercase();
    let upper_old = old.to_uppercase();

    // Only add lowercase if it's different from original AND (original is included OR it's actually different)
    if lower_old != old && !map.contains_key(&lower_old) {
        map.insert(lower_old, new.to_lowercase());
    } else if lower_old == old && include_original && !map.contains_key(&lower_old) {
        // If lowercase IS the original, only add it if we're including the original
        map.insert(lower_old, new.to_lowercase());
    }

    // Only add uppercase if it's different from original AND not already in map
    if upper_old != old && !map.contains_key(&upper_old) {
        map.insert(upper_old, new.to_uppercase());
    } else if upper_old == old && include_original && !map.contains_key(&upper_old) {
        // If uppercase IS the original, only add it if we're including the original
        map.insert(upper_old, new.to_uppercase());
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_snake_case() {
        let tokens = parse_to_tokens("hello_world_test");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "hello");
        assert_eq!(tokens.tokens[1].text, "world");
        assert_eq!(tokens.tokens[2].text, "test");
    }

    #[test]
    fn test_parse_camel_case() {
        let tokens = parse_to_tokens("helloWorldTest");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "hello");
        assert_eq!(tokens.tokens[1].text, "World");
        assert_eq!(tokens.tokens[2].text, "Test");
    }

    #[test]
    fn test_parse_pascal_case() {
        let tokens = parse_to_tokens("HelloWorldTest");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "Hello");
        assert_eq!(tokens.tokens[1].text, "World");
        assert_eq!(tokens.tokens[2].text, "Test");
    }

    #[test]
    fn test_parse_kebab_case() {
        let tokens = parse_to_tokens("hello-world-test");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "hello");
        assert_eq!(tokens.tokens[1].text, "world");
        assert_eq!(tokens.tokens[2].text, "test");
    }

    #[test]
    fn test_parse_acronym() {
        let tokens = parse_to_tokens("XMLHttpRequest");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "XML");
        assert_eq!(tokens.tokens[1].text, "Http");
        assert_eq!(tokens.tokens[2].text, "Request");
    }

    #[test]
    fn test_parse_with_digits() {
        let tokens = parse_to_tokens("user2FA");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "user");
        assert_eq!(tokens.tokens[1].text, "2");
        assert_eq!(tokens.tokens[2].text, "FA");
    }

    #[test]
    fn test_detect_snake_case() {
        assert_eq!(detect_style("hello_world"), Some(Style::Snake));
    }

    #[test]
    fn test_detect_camel_case() {
        assert_eq!(detect_style("helloWorld"), Some(Style::Camel));
    }

    #[test]
    fn test_detect_pascal_case() {
        assert_eq!(detect_style("HelloWorld"), Some(Style::Pascal));
    }

    #[test]
    fn test_detect_kebab_case() {
        assert_eq!(detect_style("hello-world"), Some(Style::Kebab));
    }

    #[test]
    fn test_detect_screaming_snake_case() {
        assert_eq!(detect_style("HELLO_WORLD"), Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_to_snake_case() {
        let tokens = parse_to_tokens("HelloWorld");
        assert_eq!(to_style(&tokens, Style::Snake), "hello_world");
    }

    #[test]
    fn test_to_camel_case() {
        let tokens = parse_to_tokens("hello_world");
        assert_eq!(to_style(&tokens, Style::Camel), "helloWorld");
    }

    #[test]
    fn test_to_pascal_case() {
        let tokens = parse_to_tokens("hello_world");
        assert_eq!(to_style(&tokens, Style::Pascal), "HelloWorld");
    }

    #[test]
    fn test_to_screaming_snake_case() {
        let tokens = parse_to_tokens("helloWorld");
        assert_eq!(to_style(&tokens, Style::ScreamingSnake), "HELLO_WORLD");
    }

    #[test]
    fn test_generate_variant_map() {
        let map = generate_variant_map("old_name", "new_name", None);
        assert_eq!(map.get("old_name"), Some(&"new_name".to_string()));
        assert_eq!(map.get("oldName"), Some(&"newName".to_string()));
        assert_eq!(map.get("OldName"), Some(&"NewName".to_string()));
        assert_eq!(map.get("old-name"), Some(&"new-name".to_string()));
        assert_eq!(map.get("OLD_NAME"), Some(&"NEW_NAME".to_string()));
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(detect_style(""), None);
        let tokens = parse_to_tokens("");
        assert_eq!(tokens.tokens.len(), 0);
        assert_eq!(to_style(&tokens, Style::Snake), "");
    }

    #[test]
    fn test_single_word() {
        let tokens = parse_to_tokens("hello");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "hello");
        assert_eq!(to_style(&tokens, Style::Snake), "hello");
        assert_eq!(to_style(&tokens, Style::Camel), "hello");
        assert_eq!(to_style(&tokens, Style::Pascal), "Hello");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(detect_style("Hello World"), Some(Style::Title));
        let tokens = parse_to_tokens("Hello World");
        assert_eq!(to_style(&tokens, Style::Title), "Hello World");
    }

    #[test]
    fn test_train_case() {
        assert_eq!(detect_style("Hello-World"), Some(Style::Train));
        let tokens = parse_to_tokens("Hello-World");
        assert_eq!(to_style(&tokens, Style::Train), "Hello-World");
    }

    #[test]
    fn test_dot_case() {
        assert_eq!(detect_style("hello.world"), Some(Style::Dot));
        let tokens = parse_to_tokens("hello.world");
        assert_eq!(to_style(&tokens, Style::Dot), "hello.world");
    }

    #[test]
    fn test_mixed_case_detection() {
        assert_eq!(detect_style("123"), None);
        assert_eq!(detect_style("hello world test"), None);
        assert_eq!(detect_style("hello-World"), None);
        assert_eq!(detect_style("HELLO"), None);
        assert_eq!(detect_style("hello"), None);
    }

    #[test]
    fn test_non_ascii_handling() {
        let tokens = parse_to_tokens("hello@world#test");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "helloworldtest");
    }

    #[test]
    fn test_consecutive_delimiters() {
        let tokens = parse_to_tokens("hello__world--test");
        assert_eq!(tokens.tokens.len(), 3);
        assert_eq!(tokens.tokens[0].text, "hello");
        assert_eq!(tokens.tokens[1].text, "world");
        assert_eq!(tokens.tokens[2].text, "test");
    }

    #[test]
    fn test_variant_map_with_custom_styles() {
        let styles = vec![Style::Snake, Style::Title, Style::Dot];
        let map = generate_variant_map("oldName", "newName", Some(&styles));
        assert_eq!(map.get("old_name"), Some(&"new_name".to_string()));
        assert_eq!(map.get("Old Name"), Some(&"New Name".to_string()));
        assert_eq!(map.get("old.name"), Some(&"new.name".to_string()));
        assert!(!map.contains_key("old-name"));
        // Original camelCase pattern should NOT be included since Camel is not in styles
        assert!(!map.contains_key("oldName"));
    }

    #[test]
    fn test_variant_map_excludes_original_style() {
        // Test excluding snake_case when the original is snake_case
        let styles = vec![Style::Camel, Style::Pascal, Style::Kebab];
        let map = generate_variant_map("old_name", "new_name", Some(&styles));

        // Debug: print the map
        eprintln!("Map with excluded Snake style:");
        for (k, v) in &map {
            eprintln!("  '{}' -> '{}'", k, v);
        }

        // Should NOT include the original snake_case since Snake is not in styles
        assert!(
            !map.contains_key("old_name"),
            "Map should not contain 'old_name' when Snake is excluded"
        );

        // Should include the other styles
        assert_eq!(map.get("oldName"), Some(&"newName".to_string()));
        assert_eq!(map.get("OldName"), Some(&"NewName".to_string()));
        assert_eq!(map.get("old-name"), Some(&"new-name".to_string()));

        // Case variants should still be there if different
        assert_eq!(map.get("OLD_NAME"), Some(&"NEW_NAME".to_string()));
    }

    #[test]
    fn test_variant_map_includes_original_when_style_present() {
        // Test that original is included when its style IS in the list
        let styles = vec![Style::Snake, Style::Camel];
        let map = generate_variant_map("old_name", "new_name", Some(&styles));

        // Should include the original since Snake is in styles
        assert_eq!(map.get("old_name"), Some(&"new_name".to_string()));
        assert_eq!(map.get("oldName"), Some(&"newName".to_string()));
    }

    #[test]
    fn test_all_caps_short_acronym() {
        let tokens = TokenModel::new(vec![Token::new("IO"), Token::new("Test")]);
        assert_eq!(to_style(&tokens, Style::Pascal), "IOTest");
    }

    #[test]
    fn test_capitalize_first_with_empty() {
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn test_edge_cases() {
        let tokens = parse_to_tokens("a");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "a");

        assert_eq!(detect_style("Hello World Test"), Some(Style::Title));
        assert_eq!(detect_style("hello.World"), Some(Style::Dot));
    }

    #[test]
    fn test_parse_all_caps_longer() {
        let tokens = parse_to_tokens("ALLCAPS");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "ALLCAPS");

        let tokens2 = TokenModel::new(vec![Token::new("ALLCAPS")]);
        assert_eq!(to_style(&tokens2, Style::Pascal), "Allcaps");
    }

    #[test]
    fn test_is_train_case_edge() {
        assert!(!is_train_case(""));
        assert!(!is_train_case("-"));
        assert!(!is_train_case("hello"));
    }

    #[test]
    fn test_is_title_case_edge() {
        assert!(!is_title_case(""));
        assert!(!is_title_case(" "));
        assert!(!is_title_case("hello"));
    }
}
