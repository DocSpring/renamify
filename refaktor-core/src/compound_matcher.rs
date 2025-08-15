use crate::acronym::{classify_hyphen_container, AcronymSet, HyphenContainerStyle};
use crate::case_model::{detect_style, parse_to_tokens, to_style, Style, Token, TokenModel};
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
    if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
        println!("find_compound_variants called with styles: {:?}", styles);
    }

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
    let mut replacement_tokens = identifier_tokens.tokens;
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
        if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
            println!(
                "Found {} replacements in '{}' -> trying to create compound match",
                replacements_made, identifier
            );
        }

        // Detect the style of the original identifier
        if let Some(style) = crate::case_model::detect_style(identifier) {
            // Debug: print style detection
            if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
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
            } else if styles.contains(&Style::Original) {
                // Debug: style not in target styles but Original is enabled
                if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
                    println!("  Style {:?} not in target styles for '{}', but Original style is enabled - using simple replacement", style, identifier);
                }

                // Style detected but not in target styles - however, Original style is enabled
                // so we do a simple string replacement
                let replacement = identifier.replace(old_pattern, new_pattern);

                matches.push(CompoundMatch {
                    full_identifier: identifier.to_string(),
                    replacement,
                    style: Style::Original,
                    pattern_start: 0,
                    pattern_end: 0,
                });
            } else {
                // Debug: style not in target styles and Original not enabled
                if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
                    println!("  Style {:?} not in target styles and Original not enabled for '{}', skipping", style, identifier);
                }

                // Style detected but not in target styles and Original style not enabled - skip this match
                // This ensures that compound matching respects the target styles
            }
        } else if styles.contains(&Style::Original) {
            // Debug: no style detected but Original is enabled
            if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
                println!("  No style detected for '{}', but Original style is enabled - using simple replacement", identifier);
            }

            // No style detected (mixed or unknown style), but Original style is enabled
            // For hyphenated identifiers where we've already replaced tokens, rebuild with original separators
            let replacement = if identifier.contains('-') {
                // Rebuild the identifier from the replacement tokens, preserving hyphen positions
                let original_parts: Vec<&str> = identifier.split('-').collect();
                let original_first_part = original_parts[0];

                // Check if the first part (before hyphen) was what we replaced
                let first_part_tokens = parse_to_tokens(original_first_part);
                if first_part_tokens.tokens.len() == old_tokens.tokens.len()
                    && tokens_match(&first_part_tokens.tokens, &old_tokens.tokens)
                {
                    // The first part matches our pattern, rebuild it from replacement tokens
                    let mut result_parts = Vec::new();

                    // Rebuild the first part in its original style
                    if let Some(style) = crate::case_model::detect_style(original_first_part) {
                        let replacement_model =
                            TokenModel::new(replacement_tokens[..new_tokens.tokens.len()].to_vec());
                        result_parts.push(to_style(&replacement_model, style));
                    } else {
                        // If no style detected, check the case of the original
                        let replacement_model =
                            TokenModel::new(replacement_tokens[..new_tokens.tokens.len()].to_vec());
                        let style = if original_first_part
                            .chars()
                            .next()
                            .map_or(false, |c| c.is_uppercase())
                        {
                            Style::Pascal
                        } else {
                            Style::Snake // Default to snake_case for all-lowercase
                        };
                        result_parts.push(to_style(&replacement_model, style));
                    }

                    // Add the remaining parts unchanged
                    for part in &original_parts[1..] {
                        result_parts.push(part.to_string());
                    }

                    result_parts.join("-")
                } else {
                    // Fallback to the old hyphenated replacement logic
                    replace_in_hyphenated(identifier, old_pattern, new_pattern)
                }
            } else {
                // Simple case-insensitive replacement for other patterns
                case_insensitive_replace(identifier, old_pattern, new_pattern)
            };

            matches.push(CompoundMatch {
                full_identifier: identifier.to_string(),
                replacement,
                style: Style::Original,
                pattern_start: 0,
                pattern_end: 0,
            });
        } else {
            // Debug: no style detected and Original not enabled
            if std::env::var("REFAKTOR_DEBUG_COMPOUND").is_ok() {
                println!(
                    "  No style detected and Original not enabled for '{}', skipping",
                    identifier
                );
            }

            // No style detected and Original style not enabled - skip this match
        }
    }

    matches
}

/// Replace pattern in a hyphenated identifier, handling each part independently
fn replace_in_hyphenated(identifier: &str, old_pattern: &str, new_pattern: &str) -> String {
    let parts: Vec<&str> = identifier.split('-').collect();
    let mut replaced_parts = Vec::new();

    for part in parts {
        // Check if this part matches the pattern exactly (case-insensitive)
        if part.to_lowercase() == old_pattern.to_lowercase() {
            // Exact match - detect the style of this part and apply appropriate replacement
            if part.chars().all(|c| c.is_ascii_uppercase()) {
                // All caps part - use SCREAMING_SNAKE style regardless of separators
                let new_tokens = crate::case_model::parse_to_tokens(new_pattern);
                let replacement = crate::case_model::to_style(
                    &new_tokens,
                    crate::case_model::Style::ScreamingSnake,
                );
                replaced_parts.push(replacement);
            } else if let Some(style) = crate::case_model::detect_style(part) {
                // Convert new pattern to match the style of this part
                let new_tokens = crate::case_model::parse_to_tokens(new_pattern);
                let replacement = crate::case_model::to_style(&new_tokens, style);
                replaced_parts.push(replacement);
            } else if part.chars().next().map_or(false, |c| c.is_uppercase()) {
                // Part starts with uppercase, likely Pascal case
                // Convert new pattern to Pascal case
                let new_tokens = crate::case_model::parse_to_tokens(new_pattern);
                let replacement = crate::case_model::to_style(&new_tokens, Style::Pascal);
                replaced_parts.push(replacement);
            } else {
                // Default: keep the same case style as the original
                replaced_parts.push(new_pattern.to_string());
            }
        } else if part.to_lowercase().contains(&old_pattern.to_lowercase()) {
            // Part contains the pattern but not an exact match
            // Do a case-preserving replacement within this part
            replaced_parts.push(case_insensitive_replace(part, old_pattern, new_pattern));
        } else {
            // Part doesn't contain the pattern at all, keep it as-is
            replaced_parts.push(part.to_string());
        }
    }

    replaced_parts.join("-")
}

/// Case-insensitive replacement for non-hyphenated patterns
fn case_insensitive_replace(identifier: &str, old_pattern: &str, new_pattern: &str) -> String {
    // Try to detect if the identifier contains the pattern with different casing
    let lower_id = identifier.to_lowercase();
    let lower_pattern = old_pattern.to_lowercase();

    if let Some(pos) = lower_id.find(&lower_pattern) {
        // Found the pattern, extract the actual cased version
        let actual_pattern = &identifier[pos..pos + old_pattern.len()];

        // Detect the style of the actual pattern and apply to new pattern
        if let Some(style) = crate::case_model::detect_style(actual_pattern) {
            let new_tokens = crate::case_model::parse_to_tokens(new_pattern);
            let replacement = crate::case_model::to_style(&new_tokens, style);
            identifier.replace(actual_pattern, &replacement)
        } else {
            // No clear style, just do a simple replacement preserving the case of the first letter
            let replacement = if actual_pattern
                .chars()
                .next()
                .map_or(false, |c| c.is_uppercase())
            {
                // Capitalize the new pattern
                let mut chars = new_pattern.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            } else {
                new_pattern.to_string()
            };
            identifier.replace(actual_pattern, &replacement)
        }
    } else {
        // Pattern not found, return as-is
        identifier.to_string()
    }
}

/// Generate all compound variants for a given pattern
pub fn generate_compound_variants(
    old: &str,
    new: &str,
    styles: &[Style],
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
