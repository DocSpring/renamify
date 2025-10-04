// Ambiguity resolution system for determining case styles when multiple interpretations are possible

pub mod cross_file_context;
pub mod file_context;
pub mod language_heuristics;
pub mod languages;
pub mod resolver;

pub use resolver::{AmbiguityContext, AmbiguityResolver, ResolvedStyle};

use crate::case_model::Style;

/// Check if text could match multiple styles from the given list
pub fn is_ambiguous(text: &str, styles: &[Style]) -> bool {
    use crate::case_constraints::filter_compatible_styles;
    filter_compatible_styles(text, styles).len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_model::Style;

    #[test]
    fn test_is_ambiguous() {
        let all_styles = Style::all_styles();

        assert!(is_ambiguous("api", &all_styles)); // Could be snake, kebab, camel, lower
        assert!(is_ambiguous("API", &all_styles)); // Could be Pascal (acronym), SCREAMING_SNAKE, upper
        assert!(is_ambiguous("config", &all_styles)); // Could be multiple styles
        assert!(is_ambiguous("id", &all_styles)); // Could be multiple styles

        // Not ambiguous - has clear separator or case pattern
        assert!(!is_ambiguous("user_id", &all_styles)); // snake_case
        assert!(!is_ambiguous("userId", &all_styles)); // camelCase
        assert!(!is_ambiguous("UserID", &all_styles)); // PascalCase with acronym
    }
}
