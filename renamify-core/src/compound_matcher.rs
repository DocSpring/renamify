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

    // Look for the pattern tokens within the identifier tokens
    let pattern_len = old_tokens.tokens.len();
    let identifier_len = identifier_tokens.tokens.len();

    if pattern_len > identifier_len {
        return matches; // Pattern is longer than identifier, can't be a compound
    }

    // Find ALL occurrences of the pattern and replace them all
    let mut replacement_tokens = identifier_tokens.tokens;
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
            // Found a match! Replace these tokens with the new pattern
            let mut new_tokens_adjusted = Vec::new();

            // Preserve case from the original tokens
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

            // Replace the tokens at this position
            replacement_tokens.splice(pos..pos + pattern_len, new_tokens_adjusted.clone());
            replacements_made += 1;

            // Move position forward by the length of the replacement
            pos += new_tokens.tokens.len();
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

        if let Some(style) = detected_style {
            // Debug: print style detection
            if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                println!("  Detected style: {:?} for '{}'", style, identifier);
            }

            // Identifier detected with style and replacements made
            // Check if this style is in our target styles
            if styles.contains(&style) {
                // Convert the replacement tokens to the same style
                let replacement_model = TokenModel::new(replacement_tokens.clone());
                let mut replacement = to_style(&replacement_model, style);

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
