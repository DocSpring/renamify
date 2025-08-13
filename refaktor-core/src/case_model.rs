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
    ScreamingTrain, // ALL-CAPS-WITH-HYPHENS
    Dot,
    Original, // Matches the exact original string regardless of case style
}

impl Style {
    /// Returns the default styles used by refaktor
    pub fn default_styles() -> Vec<Style> {
        vec![
            Style::Original,
            Style::Snake,
            Style::Kebab,
            Style::Camel,
            Style::Pascal,
            Style::ScreamingSnake,
            Style::Train,          // Include Train-Case in defaults
            Style::ScreamingTrain, // Include ScreamingTrain for ALL-CAPS-PATTERNS
        ]
    }
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
        (false, true, false, false, true, false) => Some(Style::ScreamingTrain), // ALL-CAPS-WITH-HYPHENS
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
            if s.bytes().next().is_some_and(|b| b.is_ascii_uppercase()) {
                Some(Style::Pascal)
            } else if s.bytes().next().is_some_and(|b| b.is_ascii_lowercase()) {
                Some(Style::Camel)
            } else {
                None
            }
        },
        _ => None,
    }
}

fn is_train_case(s: &str) -> bool {
    let acronym_set = crate::acronym::get_default_acronym_set();

    s.split('-').all(|word| {
        if word.is_empty() {
            return false;
        }

        // Check if it's Title case (First upper, rest lower)
        let is_title = word.bytes().next().is_some_and(|b| b.is_ascii_uppercase())
            && word.bytes().skip(1).all(|b| b.is_ascii_lowercase());

        // Check if it's a known acronym (all uppercase)
        let is_acronym = word.len() >= 2
            && word.bytes().all(|b| b.is_ascii_uppercase())
            && acronym_set.is_acronym(word);

        // Accept either Title case or known acronym
        is_title || is_acronym
    })
}

fn is_title_case(s: &str) -> bool {
    s.split(' ').all(|word| {
        !word.is_empty()
            && word.bytes().next().is_some_and(|b| b.is_ascii_uppercase())
            && word.bytes().skip(1).all(|b| b.is_ascii_lowercase())
    })
}

pub fn parse_to_tokens(s: &str) -> TokenModel {
    parse_to_tokens_with_acronyms(s, crate::acronym::get_default_acronym_set())
}

pub fn parse_to_tokens_with_acronyms(
    s: &str,
    acronym_set: &crate::acronym::AcronymSet,
) -> TokenModel {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut current = Vec::new();

    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        if b == b'_' || b == b'-' || b == b'.' || b == b' ' {
            // Delimiter: finish current token and continue
            if !current.is_empty() {
                tokens.push(Token::new(
                    std::str::from_utf8(&current).unwrap_or_default(),
                ));
                current.clear();
            }
            i += 1;
        } else if b.is_ascii_alphabetic() || b.is_ascii_digit() {
            // Check for known acronyms at the start of a new token
            if current.is_empty() {
                // Use trie to find longest matching acronym
                if let Some(acronym) = acronym_set.find_longest_match(s, i) {
                    tokens.push(Token::new(acronym));
                    i += acronym.len();
                    continue;
                }

                // Handle uppercase sequences that might be acronyms
                if b.is_ascii_uppercase() {
                    // Look ahead to find the end of consecutive uppercase letters
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_uppercase() {
                        j += 1;
                    }

                    // If followed by lowercase and multiple uppercase, split appropriately
                    // This handles cases like "URLParser" -> "URL" + "Parser"
                    if j < bytes.len() && bytes[j].is_ascii_lowercase() && j > i + 1 {
                        // Multiple uppercase letters followed by lowercase
                        // Take all but the last uppercase letter as one token
                        if j > i + 2 {
                            let acronym_part = std::str::from_utf8(&bytes[i..j - 1]).unwrap_or("");
                            tokens.push(Token::new(acronym_part));
                            i = j - 1;
                            continue;
                        }
                    }
                }
            }

            // Standard case boundary detection
            if i > 0 && !current.is_empty() {
                let prev = bytes[i - 1];
                let should_split = (prev.is_ascii_lowercase() && b.is_ascii_uppercase())
                    || (prev.is_ascii_alphabetic() && b.is_ascii_digit())
                    || (prev.is_ascii_digit() && b.is_ascii_alphabetic());

                if should_split {
                    tokens.push(Token::new(
                        std::str::from_utf8(&current).unwrap_or_default(),
                    ));
                    current.clear();
                }
            }

            current.push(b);
            i += 1;
        } else {
            // Skip non-alphanumeric, non-delimiter characters (treat as if they don't exist)
            i += 1;
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
            .collect::<String>(),

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

        Style::Train => {
            let acronym_set = crate::acronym::get_default_acronym_set();
            model
                .tokens
                .iter()
                .map(|t| {
                    // If the token is a known acronym in uppercase, preserve it
                    if t.text.bytes().all(|b| b.is_ascii_uppercase())
                        && acronym_set.is_acronym(&t.text)
                    {
                        t.text.clone()
                    } else {
                        capitalize_first(&t.text)
                    }
                })
                .collect::<Vec<_>>()
                .join("-")
        },

        Style::ScreamingTrain => model
            .tokens
            .iter()
            .map(|t| t.text.to_uppercase())
            .collect::<Vec<_>>()
            .join("-"),

        Style::Dot => model
            .tokens
            .iter()
            .map(|t| t.text.to_lowercase())
            .collect::<Vec<_>>()
            .join("."),

        Style::Original => {
            // Original style should be handled separately in generate_variant_map
            // This case should never be reached
            panic!("Original style should not be processed through to_style function")
        },
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
        Style::Original, // Always include the exact original string
        Style::Snake,
        Style::Kebab,
        Style::Camel,
        Style::Pascal,
        Style::ScreamingSnake,
        Style::Train, // Include Train-Case for patterns like Refaktor-Core-Engine
        Style::ScreamingTrain, // Include ScreamingTrain for patterns like REFAKTOR-DEBUG
    ];
    let styles = styles.unwrap_or(&default_styles);

    let old_tokens = parse_to_tokens(old);
    let new_tokens = parse_to_tokens(new);

    let mut map = BTreeMap::new();

    // Process styles in order to prioritize Original style
    for style in styles {
        if *style == Style::Original {
            // Add the original pattern directly
            map.insert(old.to_string(), new.to_string());
        } else {
            let old_variant = to_style(&old_tokens, *style);
            let new_variant = to_style(&new_tokens, *style);

            // Only add if not already in map (Original takes priority)
            if !map.contains_key(&old_variant) {
                map.insert(old_variant, new_variant);
            }
        }
    }

    // Add case variants (lowercase and uppercase) but only if not already in map
    let lower_old = old.to_lowercase();
    let upper_old = old.to_uppercase();

    if lower_old != old && !map.contains_key(&lower_old) {
        map.insert(lower_old, new.to_lowercase());
    }

    if upper_old != old && !map.contains_key(&upper_old) {
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
        // With acronym-aware tokenization, XML stays together but Http gets split
        // because HTTP is an acronym but "Http" in mixed case is not
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
    fn test_detect_train_case() {
        assert_eq!(detect_style("Hello-World"), Some(Style::Train));
        assert_eq!(detect_style("Refaktor-Core-Engine"), Some(Style::Train));
    }

    #[test]
    fn test_detect_screaming_train_case() {
        assert_eq!(detect_style("HELLO-WORLD"), Some(Style::ScreamingTrain));
        assert_eq!(detect_style("REFAKTOR-DEBUG"), Some(Style::ScreamingTrain));
        assert_eq!(
            detect_style("ALL-CAPS-HYPHENATED"),
            Some(Style::ScreamingTrain)
        );
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
    fn test_to_train_case() {
        let tokens = parse_to_tokens("hello_world");
        assert_eq!(to_style(&tokens, Style::Train), "Hello-World");

        let tokens = parse_to_tokens("refaktor_core_engine");
        assert_eq!(to_style(&tokens, Style::Train), "Refaktor-Core-Engine");
    }

    #[test]
    fn test_to_screaming_train_case() {
        let tokens = parse_to_tokens("hello_world");
        assert_eq!(to_style(&tokens, Style::ScreamingTrain), "HELLO-WORLD");

        let tokens = parse_to_tokens("smart_search_and_replace");
        assert_eq!(
            to_style(&tokens, Style::ScreamingTrain),
            "SMART-SEARCH-AND-REPLACE"
        );
    }

    #[test]
    fn test_generate_variant_map() {
        let map = generate_variant_map("old_name", "new_name", None);
        assert_eq!(map.get("old_name"), Some(&"new_name".to_string()));
        assert_eq!(map.get("oldName"), Some(&"newName".to_string()));
        assert_eq!(map.get("OldName"), Some(&"NewName".to_string()));
        assert_eq!(map.get("old-name"), Some(&"new-name".to_string()));
        assert_eq!(map.get("OLD_NAME"), Some(&"NEW_NAME".to_string()));
        // Check Train and ScreamingTrain are included
        assert_eq!(map.get("Old-Name"), Some(&"New-Name".to_string()));
        assert_eq!(map.get("OLD-NAME"), Some(&"NEW-NAME".to_string()));
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
