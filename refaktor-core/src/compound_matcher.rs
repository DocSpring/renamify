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

    // Parse all three into tokens
    let identifier_tokens = parse_to_tokens(identifier);
    let old_tokens = parse_to_tokens(old_pattern);
    let new_tokens = parse_to_tokens(new_pattern);

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
    let mut replacement_tokens = identifier_tokens.tokens.clone();
    let mut replacements_made = 0;
    let mut pos = 0;

    while pos <= replacement_tokens.len().saturating_sub(pattern_len) {
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
                        .map_or(false, |c| c.is_uppercase())
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
        // Detect the style of the original identifier
        if let Some(style) = crate::case_model::detect_style(identifier) {
            // Identifier detected with style and replacements made
            // Check if this style is in our target styles
            if styles.contains(&style) {
                // Convert the replacement tokens to the same style
                let replacement_model = TokenModel::new(replacement_tokens.clone());
                let replacement = to_style(&replacement_model, style);
                // Successfully created replacement in same style

                matches.push(CompoundMatch {
                    full_identifier: identifier.to_string(),
                    replacement,
                    style,
                    pattern_start: 0, // Not meaningful when we have multiple replacements
                    pattern_end: 0,   // Not meaningful when we have multiple replacements
                });
            }
        }
    }

    matches
}

/// Generate all compound variants for a given pattern
pub fn generate_compound_variants(
    old: &str,
    new: &str,
    styles: &[Style],
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    // For each style, generate compound examples
    // This is used during pattern building to create regex patterns
    // that can match compound words

    // Note: This is a simplified version. In practice, we'd need to
    // scan the actual codebase to find real compound words, or use
    // a more sophisticated pattern matching approach.

    map
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
