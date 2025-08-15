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
    /// Returns the default styles used by renamify
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

    let debug = std::env::var("DEBUG_TOKENIZE").is_ok();
    if debug {
        eprintln!("=== Tokenizing: '{}' ===", s);
    }

    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if debug {
            eprintln!(
                "  [{}] Processing '{}', current = '{}'",
                i,
                b as char,
                std::str::from_utf8(&current).unwrap_or("?")
            );
        }

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
                // This handles both letter-starting acronyms (API, URL) and digit-starting ones (2FA, 3D)
                if let Some(acronym) = acronym_set.find_longest_match(s, i) {
                    let next_pos = i + acronym.len();
                    let mut should_skip_acronym = false;

                    // Check what comes after the potential acronym
                    if next_pos < bytes.len() {
                        let next_byte = bytes[next_pos];

                        // Skip if followed by a digit (e.g., "ARM" in "ARM64")
                        if next_byte.is_ascii_digit()
                            && !acronym.bytes().any(|b| b.is_ascii_digit())
                        {
                            should_skip_acronym = true;
                        }

                        // Skip if the acronym match is part of a longer word
                        // For example, "IDE" in "IDENTIFIERS" should not be matched
                        // Check if we're in the same case style and continuing the word
                        if bytes[i].is_ascii_uppercase() && next_byte.is_ascii_uppercase() {
                            // Both uppercase - might be part of same word like "IDENTIFIERS"
                            // Only accept the acronym if it's the complete uppercase sequence
                            let mut j = next_pos;
                            while j < bytes.len() && bytes[j].is_ascii_uppercase() {
                                j += 1;
                            }
                            // If there are more uppercase letters, this might be a longer word
                            // Only accept known acronyms if they're followed by a clear boundary
                            if j > next_pos {
                                // There are more uppercase letters - likely part of a longer word
                                should_skip_acronym = true;
                            }
                        } else if bytes[i].is_ascii_lowercase() && next_byte.is_ascii_lowercase() {
                            // Both lowercase - definitely part of same word
                            should_skip_acronym = true;
                        }
                    }

                    if !should_skip_acronym {
                        tokens.push(Token::new(acronym));
                        i += acronym.len();
                        continue;
                    }
                    // Otherwise, fall through to normal processing
                }

                // Handle uppercase sequences that might be acronyms
                if b.is_ascii_uppercase() {
                    // Look ahead to find consecutive uppercase letters (potential acronym)
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_uppercase() {
                        j += 1;
                    }

                    // If we have multiple uppercase letters followed by lowercase,
                    // this might be an acronym followed by a word (e.g., "URLParser" -> "URL" + "Parser")
                    if j > i + 1 && j < bytes.len() && bytes[j].is_ascii_lowercase() {
                        // Take all but the last uppercase letter as the acronym
                        let acronym_part = std::str::from_utf8(&bytes[i..j - 1]).unwrap_or("");
                        tokens.push(Token::new(acronym_part));
                        i = j - 1; // Continue from the last uppercase letter
                        continue;
                    }
                }
            }

            // Standard case boundary detection
            if i > 0 && !current.is_empty() {
                let prev = bytes[i - 1];

                // Check for various split conditions
                let mut should_split = false;

                // 1. lowercase to uppercase (e.g., "camelCase" -> "camel", "Case")
                if prev.is_ascii_lowercase() && b.is_ascii_uppercase() {
                    should_split = true;
                }
                // 2. letter to digit - DON'T split (e.g., "project1" -> "project1")
                //    UNLESS the digit starts a known acronym like "2FA"
                else if prev.is_ascii_alphabetic() && b.is_ascii_digit() {
                    // Look ahead to see if this digit starts a known acronym
                    let mut potential = vec![b];
                    let mut j = i + 1;
                    // Collect following characters that could be part of an acronym
                    while j < bytes.len()
                        && (bytes[j].is_ascii_uppercase() || bytes[j].is_ascii_digit())
                    {
                        potential.push(bytes[j]);
                        j += 1;
                    }
                    if let Ok(potential_str) = std::str::from_utf8(&potential) {
                        // Only split if this IS a known acronym (like "2FA")
                        should_split = acronym_set.is_acronym(potential_str);
                    } else {
                        should_split = false;
                    }
                }
                // 3. digit to uppercase letter (e.g., "arm64Arch" -> "arm64", "Arch")
                //    BUT don't split if the digit is part of a known acronym like "2FA"
                else if prev.is_ascii_digit() && b.is_ascii_uppercase() {
                    // Check if we're in the middle of a known acronym
                    // Look back to find where digits started
                    let mut digit_start = current.len();
                    while digit_start > 0 && current[digit_start - 1].is_ascii_digit() {
                        digit_start -= 1;
                    }

                    // Build the potential acronym from the digits we have + upcoming uppercase
                    let mut potential = Vec::new();
                    potential.extend_from_slice(&current[digit_start..]);
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_uppercase() {
                        potential.push(bytes[j]);
                        j += 1;
                    }

                    if let Ok(potential_str) = std::str::from_utf8(&potential) {
                        // Only split if this is NOT a known acronym
                        should_split = !acronym_set.is_acronym(potential_str);
                    } else {
                        should_split = true;
                    }
                }

                if should_split {
                    if debug {
                        eprintln!(
                            "    -> SPLIT! Pushing token: '{}'",
                            std::str::from_utf8(&current).unwrap_or("?")
                        );
                    }
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
        Style::Train, // Include Train-Case for patterns like Renamify-Core-Engine
        Style::ScreamingTrain, // Include ScreamingTrain for patterns like RENAMIFY-DEBUG
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
        // "2FA" is a known acronym, so it should be kept together
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "user");
        assert_eq!(tokens.tokens[1].text, "2FA");
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
        assert_eq!(detect_style("Renamify-Core-Engine"), Some(Style::Train));
    }

    #[test]
    fn test_detect_screaming_train_case() {
        assert_eq!(detect_style("HELLO-WORLD"), Some(Style::ScreamingTrain));
        assert_eq!(detect_style("RENAMIFY-DEBUG"), Some(Style::ScreamingTrain));
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

        let tokens = parse_to_tokens("renamify_core_engine");
        assert_eq!(to_style(&tokens, Style::Train), "Renamify-Core-Engine");
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
    fn test_mixed_alphanumeric_tokens() {
        // Test that tokens with mixed letters and numbers stay together
        let tokens = parse_to_tokens("amd64");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "amd64");

        let tokens = parse_to_tokens("arm64");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "arm64");

        let tokens = parse_to_tokens("project1");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "project1");

        let tokens = parse_to_tokens("project2");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "project2");
    }

    #[test]
    fn test_alphanumeric_in_all_styles() {
        // Test SCREAMING_SNAKE_CASE with alphanumeric
        let tokens = parse_to_tokens("ARM64_ARCH");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "ARM64");
        assert_eq!(tokens.tokens[1].text, "ARCH");

        let tokens = parse_to_tokens("ARCH_ARM64");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "ARCH");
        assert_eq!(tokens.tokens[1].text, "ARM64");

        // Test PascalCase with alphanumeric
        let tokens = parse_to_tokens("Arm64Arch");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "Arm64");
        assert_eq!(tokens.tokens[1].text, "Arch");

        let tokens = parse_to_tokens("ArchArm64");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "Arch");
        assert_eq!(tokens.tokens[1].text, "Arm64");

        // Test camelCase with alphanumeric
        let tokens = parse_to_tokens("arm64Arch");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "arm64");
        assert_eq!(tokens.tokens[1].text, "Arch");

        let tokens = parse_to_tokens("archArm64");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "arch");
        assert_eq!(tokens.tokens[1].text, "Arm64");

        // Test Train-Case with alphanumeric
        let tokens = parse_to_tokens("Arch-Arm64");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "Arch");
        assert_eq!(tokens.tokens[1].text, "Arm64");

        let tokens = parse_to_tokens("Arm64-Arch");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "Arm64");
        assert_eq!(tokens.tokens[1].text, "Arch");
    }

    #[test]
    fn test_style_conversion_preserves_alphanumeric() {
        // Test that converting between styles preserves alphanumeric tokens
        let tokens = TokenModel::new(vec![Token::new("arch"), Token::new("arm64")]);

        assert_eq!(to_style(&tokens, Style::Snake), "arch_arm64");
        assert_eq!(to_style(&tokens, Style::Kebab), "arch-arm64");
        assert_eq!(to_style(&tokens, Style::Pascal), "ArchArm64");
        assert_eq!(to_style(&tokens, Style::Camel), "archArm64");
        assert_eq!(to_style(&tokens, Style::ScreamingSnake), "ARCH_ARM64");
        assert_eq!(to_style(&tokens, Style::Train), "Arch-Arm64");
        assert_eq!(to_style(&tokens, Style::ScreamingTrain), "ARCH-ARM64");
        assert_eq!(to_style(&tokens, Style::Dot), "arch.arm64");

        // Test the reverse direction
        let tokens = TokenModel::new(vec![Token::new("arm64"), Token::new("arch")]);

        assert_eq!(to_style(&tokens, Style::Snake), "arm64_arch");
        assert_eq!(to_style(&tokens, Style::Kebab), "arm64-arch");
        assert_eq!(to_style(&tokens, Style::Pascal), "Arm64Arch");
        assert_eq!(to_style(&tokens, Style::Camel), "arm64Arch");
        assert_eq!(to_style(&tokens, Style::ScreamingSnake), "ARM64_ARCH");
        assert_eq!(to_style(&tokens, Style::Train), "Arm64-Arch");
        assert_eq!(to_style(&tokens, Style::ScreamingTrain), "ARM64-ARCH");
        assert_eq!(to_style(&tokens, Style::Dot), "arm64.arch");
    }

    #[test]
    fn test_kebab_case_preserves_alphanumeric() {
        // Test that kebab-case variants preserve alphanumeric tokens
        let tokens = parse_to_tokens("oldname-amd64");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "oldname");
        assert_eq!(tokens.tokens[1].text, "amd64");

        // Converting to kebab case should preserve "amd64" as one token
        let new_tokens = TokenModel::new(vec![Token::new("newname"), Token::new("amd64")]);
        assert_eq!(to_style(&new_tokens, Style::Kebab), "newname-amd64");

        // Test with project numbers
        let tokens = parse_to_tokens("oldname-project1");
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "oldname");
        assert_eq!(tokens.tokens[1].text, "project1");

        let new_tokens = TokenModel::new(vec![Token::new("newname"), Token::new("project1")]);
        assert_eq!(to_style(&new_tokens, Style::Kebab), "newname-project1");
    }

    #[test]
    fn test_variant_generation_preserves_alphanumeric() {
        // Test that variant generation preserves alphanumeric tokens correctly
        let map = generate_variant_map("oldname-amd64", "newname-amd64", Some(&[Style::Kebab]));
        assert_eq!(map.get("oldname-amd64"), Some(&"newname-amd64".to_string()));

        let map = generate_variant_map(
            "oldname-project1",
            "newname-project1",
            Some(&[Style::Kebab]),
        );
        assert_eq!(
            map.get("oldname-project1"),
            Some(&"newname-project1".to_string())
        );

        // Test with different separators
        let map = generate_variant_map("oldname_amd64", "newname_amd64", Some(&[Style::Snake]));
        assert_eq!(map.get("oldname_amd64"), Some(&"newname_amd64".to_string()));
    }

    #[test]
    fn test_file_extension_with_alphanumeric() {
        // Test file names with alphanumeric tokens
        let tokens = parse_to_tokens("oldname-linux-amd64.tar.gz");
        // Should parse as: oldname, linux, amd64, tar, gz (extension handling is separate)
        assert!(tokens.tokens.iter().any(|t| t.text == "amd64"));

        // Ensure amd64 stays as one token, not split into amd and 64
        assert!(!tokens.tokens.iter().any(|t| t.text == "amd"));
        assert!(!tokens.tokens.iter().any(|t| t.text == "64"));
    }

    #[test]
    fn test_round_trip_alphanumeric_conversion() {
        // Test that we can convert from one style to another and back without losing information

        // Start with kebab-case
        let original = "oldname-linux-amd64";
        let tokens = parse_to_tokens(original);
        let snake = to_style(&tokens, Style::Snake);
        assert_eq!(snake, "oldname_linux_amd64");
        let back_to_kebab = to_style(&parse_to_tokens(&snake), Style::Kebab);
        assert_eq!(back_to_kebab, original);

        // Start with PascalCase
        let original = "OldnameLinuxAmd64";
        let tokens = parse_to_tokens(original);
        let kebab = to_style(&tokens, Style::Kebab);
        assert_eq!(kebab, "oldname-linux-amd64");
        let back_to_pascal = to_style(&parse_to_tokens(&kebab), Style::Pascal);
        assert_eq!(back_to_pascal, original);

        // Start with SCREAMING_SNAKE_CASE
        let original = "OLDNAME_LINUX_AMD64";
        let tokens = parse_to_tokens(original);
        let camel = to_style(&tokens, Style::Camel);
        assert_eq!(camel, "oldnameLinuxAmd64");
        let back_to_screaming = to_style(&parse_to_tokens(&camel), Style::ScreamingSnake);
        assert_eq!(back_to_screaming, original);
    }

    #[test]
    fn test_round_trip_project_number_preservation() {
        // Critical test: project1 must round-trip correctly through all case styles
        // This ensures that project1 => workspace1 => project1 works correctly

        // Test with simple numbered project
        let original = "project1";
        let tokens = parse_to_tokens(original);
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "project1");

        // Round trip through all styles
        let pascal = to_style(&tokens, Style::Pascal);
        assert_eq!(pascal, "Project1");
        let kebab = to_style(&parse_to_tokens(&pascal), Style::Kebab);
        assert_eq!(kebab, "project1");

        // Test with compound identifier
        let original = "project1-db";
        let tokens = parse_to_tokens(original);
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "project1");
        assert_eq!(tokens.tokens[1].text, "db");

        // Simulate renaming: project1-db => workspace1-db => project1-db
        let workspace_tokens = vec![Token::new("workspace1"), Token::new("db")];
        let workspace_kebab = to_style(&TokenModel::new(workspace_tokens), Style::Kebab);
        assert_eq!(workspace_kebab, "workspace1-db");

        // And back to project
        let project_tokens = vec![Token::new("project1"), Token::new("db")];
        let project_kebab = to_style(&TokenModel::new(project_tokens), Style::Kebab);
        assert_eq!(project_kebab, "project1-db");

        // Test PascalCase round trip
        let original = "Project1Db";
        let tokens = parse_to_tokens(original);
        assert_eq!(tokens.tokens.len(), 2);
        assert_eq!(tokens.tokens[0].text, "Project1");
        assert_eq!(tokens.tokens[1].text, "Db");

        let kebab = to_style(&tokens, Style::Kebab);
        assert_eq!(kebab, "project1-db");
        let back_to_pascal = to_style(&parse_to_tokens(&kebab), Style::Pascal);
        assert_eq!(back_to_pascal, "Project1Db");
    }

    #[test]
    fn test_is_title_case_edge() {
        assert!(!is_title_case(""));
        assert!(!is_title_case(" "));
        assert!(!is_title_case("hello"));
    }
}
