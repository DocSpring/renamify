use crate::case_model::{parse_to_tokens, to_style, Style, TokenModel, Token};
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
    if identifier_tokens.tokens.len() == old_tokens.tokens.len() &&
       tokens_match(&identifier_tokens.tokens, &old_tokens.tokens) {
        return matches; // Let the exact match logic handle this
    }
    
    // Look for the pattern tokens within the identifier tokens
    let pattern_len = old_tokens.tokens.len();
    let identifier_len = identifier_tokens.tokens.len();
    
    if pattern_len > identifier_len {
        return matches; // Pattern is longer than identifier, can't be a compound
    }
    
    // Sliding window to find the pattern within the identifier
    for start_pos in 0..=(identifier_len - pattern_len) {
        let end_pos = start_pos + pattern_len;
        let window = &identifier_tokens.tokens[start_pos..end_pos];
        
        if tokens_match(window, &old_tokens.tokens) {
            // Found the pattern! Now construct the replacement
            let mut replacement_tokens = Vec::new();
            
            // Add tokens before the pattern
            replacement_tokens.extend_from_slice(&identifier_tokens.tokens[..start_pos]);
            
            // Add the new pattern tokens
            replacement_tokens.extend_from_slice(&new_tokens.tokens);
            
            // Add tokens after the pattern
            replacement_tokens.extend_from_slice(&identifier_tokens.tokens[end_pos..]);
            
            // Detect the style of the original identifier
            if let Some(style) = crate::case_model::detect_style(identifier) {
                // Check if this style is in our target styles
                if styles.contains(&style) {
                    // Convert the replacement tokens to the same style
                    let replacement_model = TokenModel::new(replacement_tokens);
                    let replacement = to_style(&replacement_model, style);
                    
                    matches.push(CompoundMatch {
                        full_identifier: identifier.to_string(),
                        replacement,
                        style,
                        pattern_start: start_pos,
                        pattern_end: end_pos,
                    });
                }
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
        let matches = find_compound_variants(
            "PreviewFormatArg",
            "preview_format",
            "preview",
            &styles,
        );
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "PreviewFormatArg");
        assert_eq!(matches[0].replacement, "PreviewArg");
    }
    
    #[test]
    fn test_find_compound_in_middle() {
        let styles = vec![Style::Camel];
        let matches = find_compound_variants(
            "shouldPreviewFormatPlease",
            "preview_format",
            "preview",
            &styles,
        );
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "shouldPreviewFormatPlease");
        assert_eq!(matches[0].replacement, "shouldPreviewPlease");
    }
    
    #[test]
    fn test_find_compound_at_end() {
        let styles = vec![Style::Snake];
        let matches = find_compound_variants(
            "get_preview_format",
            "preview_format",
            "preview",
            &styles,
        );
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].full_identifier, "get_preview_format");
        assert_eq!(matches[0].replacement, "get_preview");
    }
    
    #[test]
    fn test_exact_match_returns_empty() {
        let styles = vec![Style::Pascal];
        let matches = find_compound_variants(
            "PreviewFormat",
            "preview_format",
            "preview",
            &styles,
        );
        
        // Exact matches should be handled by the regular logic, not compound logic
        assert_eq!(matches.len(), 0);
    }
    
    #[test]
    fn test_no_match_returns_empty() {
        let styles = vec![Style::Pascal];
        let matches = find_compound_variants(
            "SomethingElse",
            "preview_format",
            "preview",
            &styles,
        );
        
        assert_eq!(matches.len(), 0);
    }
}