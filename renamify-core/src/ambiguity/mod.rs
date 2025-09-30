// Ambiguity resolution system for determining case styles when multiple interpretations are possible

pub mod cross_file_context;
pub mod file_context;
pub mod language_heuristics;
pub mod languages;
pub mod resolver;

pub use resolver::{AmbiguityContext, AmbiguityResolver, ResolvedStyle};

use crate::case_model::Style;

/// Check if a given text could potentially be a certain style
/// based on its character properties
pub fn could_be_style(text: &str, style: Style) -> bool {
    if text.is_empty() {
        return false;
    }

    let first_char = text.chars().next().unwrap();
    let has_uppercase = text.chars().any(char::is_uppercase);
    let has_lowercase = text.chars().any(char::is_lowercase);
    let has_underscore = text.contains('_');
    let has_hyphen = text.contains('-');
    let has_dot = text.contains('.');
    let has_space = text.contains(' ');

    match style {
        Style::Snake => {
            // snake_case: all lowercase, may have underscores
            !has_uppercase
                && !has_space
                && (has_underscore
                    || text
                        .chars()
                        .all(|c| c.is_lowercase() || c.is_numeric() || c == '_'))
        },
        Style::Kebab => {
            // kebab-case: all lowercase, may have hyphens
            !has_uppercase
                && !has_space
                && (has_hyphen
                    || text
                        .chars()
                        .all(|c| c.is_lowercase() || c.is_numeric() || c == '-'))
        },
        Style::Camel => {
            // camelCase: starts lowercase, no separators
            first_char.is_lowercase() && !has_underscore && !has_hyphen && !has_dot && !has_space
        },
        Style::Pascal => {
            // PascalCase: starts uppercase, has mixed case, no separators
            first_char.is_uppercase() && !has_underscore && !has_hyphen && !has_dot && !has_space
        },
        Style::ScreamingSnake => {
            // SCREAMING_SNAKE: all uppercase, may have underscores
            !has_lowercase
                && !has_space
                && (has_underscore
                    || text
                        .chars()
                        .all(|c| c.is_uppercase() || c.is_numeric() || c == '_'))
        },
        Style::Train => {
            // Train-Case: Each segment is Title case (First-Second) or all caps acronym (API-Service)
            // Must have hyphens for multi-segment, single segment must be Title or all caps
            if has_space {
                false
            } else if has_hyphen {
                // Check if all segments follow Train-Case pattern
                text.split('-').all(|segment| {
                    !segment.is_empty()
                        && (
                            // Title case: First letter upper, rest lower
                            (segment.chars().next().unwrap().is_uppercase()
                         && segment.chars().skip(1).all(|c| c.is_lowercase() || c.is_numeric()))
                        // Or all uppercase (acronym)
                        || segment.chars().all(|c| c.is_uppercase() || c.is_numeric())
                        )
                })
            } else {
                // Single segment: must be Title case or all uppercase
                first_char.is_uppercase()
                    && (
                        // All uppercase (like API, SAML)
                        text.chars().all(|c| c.is_uppercase() || c.is_numeric())
                    // Or Title case (like Hello, World)
                    || text.chars().skip(1).all(|c| c.is_lowercase() || c.is_numeric())
                    )
            }
        },
        Style::ScreamingTrain => {
            // SCREAMING-TRAIN: all uppercase, MUST have hyphens
            !has_lowercase && has_hyphen && !has_space
        },
        Style::Title => {
            // Title Case: MUST have spaces and capital letters
            text.contains(' ') && first_char.is_uppercase()
        },
        Style::Dot => {
            // dot.case: all lowercase with dots
            !has_uppercase
                && !has_space
                && (has_dot
                    || text
                        .chars()
                        .all(|c| c.is_lowercase() || c.is_numeric() || c == '.'))
        },
        Style::LowerJoined => {
            // lower: all lowercase, no separators
            !has_uppercase && !has_underscore && !has_hyphen && !has_dot && !has_space
        },
        Style::UpperJoined => {
            // UPPER: all uppercase, no separators
            !has_lowercase && !has_underscore && !has_hyphen && !has_dot && !has_space
        },
        Style::Sentence => {
            // Sentence case: First word capitalized, rest lowercase, spaces
            has_space && !has_underscore && !has_hyphen && !has_dot
        },
        Style::LowerSentence => {
            // lower sentence: all lowercase with spaces
            has_space && !has_uppercase && !has_underscore && !has_hyphen && !has_dot
        },
        Style::UpperSentence => {
            // UPPER SENTENCE: all uppercase with spaces
            has_space && !has_lowercase && !has_underscore && !has_hyphen && !has_dot
        },
    }
}

/// Get all possible styles that a text could be interpreted as
pub fn get_possible_styles(text: &str) -> Vec<Style> {
    let mut styles = Vec::new();

    for style in &[
        Style::Snake,
        Style::Kebab,
        Style::Camel,
        Style::Pascal,
        Style::ScreamingSnake,
        Style::Train,
        Style::ScreamingTrain,
        Style::Title,
        Style::Dot,
        // NOTE: Lower and Upper are excluded as they destroy word boundaries
        // and should never be used for identifier transformations
    ] {
        if could_be_style(text, *style) {
            styles.push(*style);
        }
    }

    styles
}

/// Check if text is ambiguous (could be multiple styles)
pub fn is_ambiguous(text: &str) -> bool {
    get_possible_styles(text).len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_could_be_style() {
        // Lowercase text
        assert!(could_be_style("api", Style::Snake));
        assert!(could_be_style("api", Style::Kebab));
        assert!(could_be_style("api", Style::Camel));
        assert!(!could_be_style("api", Style::Pascal)); // Starts lowercase
        assert!(could_be_style("api", Style::LowerJoined));
        assert!(!could_be_style("api", Style::UpperJoined));

        // Uppercase text
        assert!(!could_be_style("API", Style::Snake)); // Has uppercase
        assert!(!could_be_style("API", Style::Camel)); // Starts uppercase
        assert!(could_be_style("API", Style::Pascal));
        assert!(could_be_style("API", Style::UpperJoined));

        // Mixed case
        assert!(!could_be_style("userId", Style::Snake));
        assert!(could_be_style("userId", Style::Camel));
        assert!(!could_be_style("userId", Style::Pascal));

        // With separators
        assert!(could_be_style("user_id", Style::Snake));
        assert!(!could_be_style("user_id", Style::Camel));
        assert!(could_be_style("user-id", Style::Kebab));
        assert!(!could_be_style("user-id", Style::Snake)); // Has hyphen, not underscore
    }

    #[test]
    fn test_is_ambiguous() {
        assert!(is_ambiguous("api")); // Could be snake, kebab, camel, lower
        assert!(is_ambiguous("API")); // Could be Pascal, SCREAMING_SNAKE, upper
        assert!(is_ambiguous("config")); // Could be multiple styles
        assert!(is_ambiguous("id")); // Could be multiple styles
                                     // Note: user_id, userId, UserID would never reach ambiguity resolution
                                     // as they have clear styles and would be handled by detect_style()
    }
}
