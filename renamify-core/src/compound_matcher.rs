use crate::case_model::{parse_to_tokens, to_style, Style, Token, TokenModel};
use std::collections::BTreeMap;

/// Represents a compound match where the pattern was found within a larger identifier
#[derive(Debug, Clone)]
pub struct CompoundMatch {
    /// The full identifier that contains the pattern
    pub full_identifier: String,
    /// The replacement for the full identifier
    pub replacement: String,
    /// The style of the identifier
    pub style: Style,
    /// Start position of the pattern within the identifier tokens
    pub pattern_start: usize,
    /// End position of the pattern within the identifier tokens
    pub pattern_end: usize,
}

/// Check if a sequence of tokens matches a pattern
fn tokens_match(tokens: &[Token], pattern_tokens: &[Token]) -> bool {
    if tokens.len() != pattern_tokens.len() {
        return false;
    }

    for (token, pattern) in tokens.iter().zip(pattern_tokens.iter()) {
        if token.text.to_lowercase() != pattern.text.to_lowercase() {
            return false;
        }
    }

    true
}

/// Find compound words that contain the pattern and generate replacements
#[allow(clippy::too_many_lines)]
pub fn find_compound_variants(
    identifier: &str,
    old_pattern: &str,
    new_pattern: &str,
    styles: &[Style],
) -> Vec<CompoundMatch> {
    let mut matches = Vec::new();

    // Debug: print styles array being passed
    if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
        println!("find_compound_variants called with styles: {:?}", styles);
    }

    // Parse all three into tokens
    let identifier_tokens = parse_to_tokens(identifier);
    let old_tokens = parse_to_tokens(old_pattern);
    let new_tokens = parse_to_tokens(new_pattern);

    // Debug: Print token info
    if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
        println!(
            "identifier_tokens for '{}': {:?}",
            identifier, identifier_tokens
        );
        println!("old_tokens for '{}': {:?}", old_pattern, old_tokens);
        println!("new_tokens for '{}': {:?}", new_pattern, new_tokens);
    }

    // If the identifier IS the pattern (exact match), skip compound logic
    if identifier_tokens.tokens.len() == old_tokens.tokens.len()
        && tokens_match(&identifier_tokens.tokens, &old_tokens.tokens)
    {
        return matches; // Let the exact match logic handle this
    }

    // Special case for mixed-style identifiers: if the identifier starts with
    // the exact search pattern followed by a separator, AND has mixed separators,
    // do a simple prefix replacement
    // This handles cases like "renamify_someCAMEL-case" -> "renamed_renaming_tool_someCAMEL-case"
    let has_mixed_separators = (identifier.contains('_') && identifier.contains('-'))
        || (identifier.contains('_') && identifier.contains('.'))
        || (identifier.contains('-') && identifier.contains('.'));

    if has_mixed_separators
        && identifier.starts_with(old_pattern)
        && old_pattern.len() < identifier.len()
    {
        let char_after = identifier.chars().nth(old_pattern.len());
        if let Some(ch) = char_after {
            if ch == '_' || ch == '-' || ch == '.' {
                // The identifier starts with our pattern followed by a separator
                // Do a simple prefix replacement for mixed-style identifiers
                let suffix = &identifier[old_pattern.len()..];
                let replacement = format!("{}{}", new_pattern, suffix);

                matches.push(CompoundMatch {
                    full_identifier: identifier.to_string(),
                    replacement,
                    style: Style::Snake, // Default style for mixed
                    pattern_start: 0,
                    pattern_end: old_pattern.len(),
                });
                return matches;
            }
        }
    }

    // Look for the pattern tokens within the identifier tokens
    let pattern_len = old_tokens.tokens.len();
    let identifier_len = identifier_tokens.tokens.len();

    if pattern_len > identifier_len {
        return matches; // Pattern is longer than identifier, can't be a compound
    }

    // Find ALL occurrences of the pattern and replace them all
    // Track replacement positions for smarter joining later
    #[allow(clippy::redundant_clone)]
    let mut replacement_tokens = identifier_tokens.tokens.clone();
    #[allow(unused_mut, clippy::collection_is_never_read)]
    let mut replacement_ranges = Vec::new(); // Track (start, end) of replacements
    let mut replacements_made = 0;
    let mut pos = 0;

    // Guard against empty pattern_len or empty replacement_tokens
    if pattern_len == 0 || replacement_tokens.is_empty() {
        if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
            println!(
                "Returning early: pattern_len={}, replacement_tokens.len()={}",
                pattern_len,
                replacement_tokens.len()
            );
        }
        return matches;
    }

    // When searching (replacement is empty), only do compound matching for mixed-case identifiers
    if new_tokens.tokens.is_empty() {
        // Check if this is a single-word search (no separators)
        let is_single_word = !old_pattern.contains('_')
            && !old_pattern.contains('-')
            && !old_pattern.contains('.')
            && !old_pattern.contains(' ');

        // For single word searches, skip compound matching
        // (This used to handle Original style fallback for mixed-case identifiers)
        if is_single_word {
            // TODO: Handle ignore_ambiguous flag here
            // For now, skip compound matching for single word searches
            return matches;
        }
        return matches;
    }

    // Additional safety check
    if pattern_len > replacement_tokens.len() {
        return matches;
    }

    while pos <= replacement_tokens.len().saturating_sub(pattern_len) {
        // Extra bounds check for safety
        if pos + pattern_len > replacement_tokens.len() {
            break;
        }
        let window = &replacement_tokens[pos..pos + pattern_len];

        if tokens_match(window, &old_tokens.tokens) {
            // Detect the style of the matched portion specifically
            // For mixed-style identifiers like "FooBarBazQux-config", we want to preserve
            // the style of just the part we're replacing
            let matched_portion_style = {
                // Use the window we already have, which is the actual matched tokens
                let matched_tokens = window;
                if matched_tokens.len() == 1 {
                    crate::case_model::detect_style(&matched_tokens[0].text)
                } else {
                    // Multiple tokens - check if they form a known style pattern
                    // Check for Title Case pattern (each token starts with uppercase, identifier has spaces)
                    let all_title_case = matched_tokens.iter().all(|t| {
                        t.text.chars().next().is_some_and(char::is_uppercase)
                            && t.text.chars().skip(1).all(char::is_lowercase)
                    });
                    if all_title_case && identifier.contains(' ') {
                        // Title Case: "Server Gateway" (with spaces)
                        Some(Style::Title)
                    } else if all_title_case {
                        // Pascal Case: "ServerGateway" (no spaces)
                        Some(Style::Pascal)
                    } else if matched_tokens
                        .iter()
                        .all(|t| t.text.chars().all(char::is_lowercase))
                    {
                        // All lowercase - could be snake, kebab, or other
                        // Check the identifier style for context
                        crate::case_model::detect_style(identifier)
                    } else {
                        // Mixed case tokens - try to detect from the joined text
                        let joined = matched_tokens
                            .iter()
                            .map(|t| t.text.as_str())
                            .collect::<Vec<_>>()
                            .join("");
                        crate::case_model::detect_style(&joined)
                    }
                }
            };

            // For Train case identifiers, always use Train case for replacements
            // Otherwise use the detected style of the matched portion
            let identifier_style = crate::case_model::detect_style(identifier);
            let final_style = if matches!(identifier_style, Some(Style::Train)) {
                // For Train case identifiers, keep Train case
                Some(Style::Train)
            } else {
                // Use the matched portion style or fall back to identifier style
                matched_portion_style.or(identifier_style)
            };

            // Generate replacement in the appropriate style
            // For PascalCase/CamelCase in mixed identifiers, we need to join them as one token
            let new_tokens_styled = if let Some(style) = final_style {
                // Convert new tokens to match the style
                match style {
                    Style::Pascal | Style::Camel => {
                        // For PascalCase and camelCase, join into a single token
                        let mut result = String::new();
                        for (i, token) in new_tokens.tokens.iter().enumerate() {
                            let cased =
                                if style == Style::Pascal || (style == Style::Camel && i > 0) {
                                    // Capitalize first letter
                                    let mut text = token.text.clone();
                                    if let Some(first_char) = text.chars().next() {
                                        text = first_char.to_uppercase().to_string() + &text[1..];
                                    }
                                    text
                                } else {
                                    // camelCase first token is lowercase
                                    token.text.to_lowercase()
                                };
                            result.push_str(&cased);
                        }
                        // Return as a single token
                        vec![Token::new(result)]
                    },
                    Style::Train => {
                        // For Train case, generate the Train-case replacement and split it
                        let new_model = TokenModel::new(new_tokens.tokens.clone());
                        let train_replacement = to_style(&new_model, Style::Train);
                        if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                            println!("    Train case replacement: '{}'", train_replacement);
                        }
                        // Split on hyphens to get individual tokens
                        let tokens: Vec<Token> = train_replacement
                            .split('-')
                            .map(|s| Token::new(s.to_string()))
                            .collect();
                        if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                            println!("    Train case tokens: {:?}", tokens);
                        }
                        tokens
                    },
                    Style::Snake | Style::ScreamingSnake => {
                        // For snake_case styles, keep tokens separate with proper casing
                        new_tokens
                            .tokens
                            .iter()
                            .map(|token| {
                                let text = if style == Style::ScreamingSnake {
                                    token.text.to_uppercase()
                                } else {
                                    token.text.to_lowercase()
                                };
                                Token::new(text)
                            })
                            .collect()
                    },
                    Style::Kebab => {
                        // For kebab-case, lowercase all tokens
                        new_tokens
                            .tokens
                            .iter()
                            .map(|token| Token::new(token.text.to_lowercase()))
                            .collect()
                    },
                    _ => {
                        // For other styles, convert to a single token with the style applied
                        let new_model = TokenModel::new(new_tokens.tokens.clone());
                        let styled_replacement = to_style(&new_model, style);
                        vec![Token::new(styled_replacement)]
                    },
                }
            } else {
                // Fallback: preserve case from the original tokens
                let mut new_tokens_adjusted = Vec::new();
                for (i, new_token) in new_tokens.tokens.iter().enumerate() {
                    if i < window.len()
                        && window[i]
                            .text
                            .chars()
                            .next()
                            .is_some_and(char::is_uppercase)
                    {
                        // Original token started with uppercase, preserve it
                        let mut adjusted_token = Token::new(new_token.text.clone());
                        if let Some(first_char) = adjusted_token.text.chars().next() {
                            adjusted_token.text =
                                first_char.to_uppercase().to_string() + &adjusted_token.text[1..];
                        }
                        new_tokens_adjusted.push(adjusted_token);
                    } else {
                        new_tokens_adjusted.push(new_token.clone());
                    }
                }
                new_tokens_adjusted
            };

            // Track where we made the replacement
            replacement_ranges.push((pos, pos + new_tokens_styled.len()));

            // Replace the tokens at this position
            replacement_tokens.splice(pos..pos + pattern_len, new_tokens_styled.clone());
            replacements_made += 1;

            // Move position forward by the length of the replacement
            pos += new_tokens_styled.len();
        } else {
            pos += 1;
        }
    }

    // If we made any replacements, create the compound match
    if replacements_made > 0 {
        // Debug: print compound matching attempt
        if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
            println!(
                "Found {} replacements in '{}' -> trying to create compound match",
                replacements_made, identifier
            );
        }

        // Detect the style of the original identifier
        let detected_style = crate::case_model::detect_style(identifier);

        if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
            println!(
                "  detect_style('{}') returned: {:?}",
                identifier, detected_style
            );
        }

        // For compound matching, be more permissive than detect_style
        // If we can't detect a style but the identifier has consistent separators,
        // try to infer a style from the separators
        let inferred_style = if detected_style.is_none() {
            if identifier.contains('-') && !identifier.contains('_') && !identifier.contains('.') {
                // Has hyphens only - treat as kebab for compound matching
                Some(Style::Kebab)
            } else if identifier.contains('_')
                && !identifier.contains('-')
                && !identifier.contains('.')
            {
                // Has underscores only - treat as snake
                Some(Style::Snake)
            } else {
                None
            }
        } else {
            detected_style
        };

        if let Some(style) = inferred_style {
            // Debug: print style detection
            if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                println!("  Detected style: {:?} for '{}'", style, identifier);
            }

            // Identifier detected with style and replacements made
            // Check if this style is in our target styles
            if styles.contains(&style) {
                // Join tokens appropriately based on the original identifier's style
                // For mixed separator identifiers like "foo_bar_baz_qux-specific",
                // we need to preserve the original separator pattern
                let mut replacement = if identifier.contains('_') && identifier.contains('-') {
                    // Mixed separators - for simplicity, use the dominant separator
                    // Determine which separator appears first (and thus is dominant)
                    let first_underscore = identifier.find('_').unwrap_or(usize::MAX);
                    let first_hyphen = identifier.find('-').unwrap_or(usize::MAX);

                    let separator = if first_underscore < first_hyphen {
                        '_' // Underscore-dominant
                    } else {
                        '-' // Hyphen-dominant
                    };

                    // Join all tokens with the dominant separator
                    replacement_tokens
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join(&separator.to_string())
                } else if identifier.contains('-') {
                    // Hyphenated identifier - join with hyphens
                    replacement_tokens
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join("-")
                } else if identifier.contains('_') {
                    // Underscore identifier - join with underscores
                    replacement_tokens
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join("_")
                } else if identifier.contains('.') {
                    // Dot identifier - join with dots
                    replacement_tokens
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join(".")
                } else if identifier.contains(' ') {
                    // Space-separated identifier - join with spaces
                    // This handles Title Case, Sentence case, etc.
                    replacement_tokens
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    // No separator - for PascalCase/CamelCase, just concatenate
                    // For other styles, use to_style
                    match style {
                        Style::Pascal | Style::Camel => {
                            // Just concatenate the tokens directly
                            replacement_tokens
                                .iter()
                                .map(|t| t.text.as_str())
                                .collect::<Vec<_>>()
                                .join("")
                        },
                        _ => {
                            // Use to_style for other styles
                            let replacement_model = TokenModel::new(replacement_tokens.clone());
                            to_style(&replacement_model, style)
                        },
                    }
                };

                // Preserve trailing delimiters from the original identifier
                // This handles cases like "oldtool_backup_" in format strings
                if identifier.ends_with('_') && !replacement.ends_with('_') {
                    replacement.push('_');
                } else if identifier.ends_with('-') && !replacement.ends_with('-') {
                    replacement.push('-');
                } else if identifier.ends_with('.') && !replacement.ends_with('.') {
                    replacement.push('.');
                }

                // Successfully created replacement in same style

                matches.push(CompoundMatch {
                    full_identifier: identifier.to_string(),
                    replacement,
                    style,
                    pattern_start: 0, // Not meaningful when we have multiple replacements
                    pattern_end: 0,   // Not meaningful when we have multiple replacements
                });
            } else {
                // Debug: style not in target styles
                if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                    println!(
                        "  Style {:?} not in target styles for '{}', skipping",
                        style, identifier
                    );
                }

                // Style detected but not in target styles - skip this match
                // This ensures that compound matching respects the target styles
            }
        } else {
            // Debug: no style detected
            if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                println!("  No style detected for '{}', skipping", identifier);
            }

            // No style detected - skip this match
            // TODO: Handle ignore_ambiguous flag here
        }
    }

    matches
}

/// Generate all compound variants for a given pattern
pub fn generate_compound_variants(
    _search: &str,
    _replace: &str,
    _styles: &[Style],
) -> BTreeMap<String, String> {
    // For each style, generate compound examples
    // This is used during pattern building to create regex patterns
    // that can match compound words

    // Note: This is a simplified version. In practice, we'd need to
    // scan the actual codebase to find real compound words, or use
    // a more sophisticated pattern matching approach.

    BTreeMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_compound_at_start() {
        let styles = vec![Style::Pascal];
        let matches = find_compound_variants("FooBarArg", "foo_bar", "foo", &styles);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "FooBarArg");
        assert_eq!(matches[0].replacement, "FooArg");
    }

    #[test]
    fn test_find_compound_in_middle() {
        let styles = vec![Style::Camel];
        let matches = find_compound_variants("shouldFooBarPlease", "foo_bar", "foo", &styles);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "shouldFooBarPlease");
        assert_eq!(matches[0].replacement, "shouldFooPlease");
    }

    #[test]
    fn test_find_compound_at_end() {
        let styles = vec![Style::Snake];
        let matches = find_compound_variants("get_foo_bar", "foo_bar", "foo", &styles);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "get_foo_bar");
        assert_eq!(matches[0].replacement, "get_foo");
    }

    #[test]
    fn test_exact_match_returns_empty() {
        let styles = vec![Style::Pascal];
        let matches = find_compound_variants("FooBar", "foo_bar", "foo", &styles);

        // Exact matches should be handled by the regular logic, not compound logic
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_no_match_returns_empty() {
        let styles = vec![Style::Pascal];
        let matches = find_compound_variants("SomethingElse", "foo_bar", "foo", &styles);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_multiple_occurrences_in_single_identifier() {
        let styles = vec![Style::Snake];
        let matches = find_compound_variants("old_name_old_name", "old_name", "new_name", &styles);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "old_name_old_name");
        assert_eq!(matches[0].replacement, "new_name_new_name");
    }

    #[test]
    fn test_multiple_occurrences_camel_case() {
        let styles = vec![Style::Camel];
        let matches = find_compound_variants("oldNameAndOldName", "old_name", "new_name", &styles);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "oldNameAndOldName");
        assert_eq!(matches[0].replacement, "newNameAndNewName");
    }
}
