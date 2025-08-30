use crate::case_model::Style;

/// Shell script language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("export") || context.contains("export ") {
        // Environment variables are SCREAMING_SNAKE
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    } else if context.ends_with('$') || context.ends_with("${") {
        // Variable references - could be SCREAMING_SNAKE for env vars or snake_case for locals
        if context
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(char::is_uppercase)
            && possible_styles.contains(&Style::ScreamingSnake)
        {
            return Some(Style::ScreamingSnake);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("function") || context.contains("() {") {
        // Function names are typically snake_case or kebab-case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with("alias") {
        // Aliases often snake_case or kebab-case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with('=') && !context.contains("export") {
        // Local variable assignments typically snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("source") || context.ends_with('.') {
        // Source files often snake_case or kebab-case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_export_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style("export", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_shell_function_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("function", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_shell_local_var_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("local_var=", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_shell_var_reference() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        // Dollar sign alone - chars check will find no alphabetic chars, so screaming snake wins
        let result = suggest_style("$", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake)); // All alphabetic chars (none) are uppercase

        // Brace syntax - also no alphabetic chars
        let result = suggest_style("${", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));

        // With lowercase context - but "$home" doesn't match any pattern, returns None
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("$", &possible_styles);
        assert_eq!(result, Some(Style::Snake)); // Snake is the only option
    }

    #[test]
    fn test_shell_function_with_parens() {
        let possible_styles = vec![Style::Snake, Style::Kebab];
        let result = suggest_style("() {", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Test kebab fallback
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("function", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_shell_alias() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("alias", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Test kebab fallback
        let possible_styles = vec![Style::Kebab, Style::Pascal];
        let result = suggest_style("alias", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_shell_source() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("source", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Test dot sourcing
        let result = suggest_style(".", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Test kebab fallback
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("source", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_shell_export_with_space() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style("export ", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_shell_no_matching_style() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("export", &possible_styles);
        assert_eq!(result, None);
    }

    #[test]
    fn test_shell_var_ref_no_screaming() {
        // When ScreamingSnake not available, fall back to snake
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("$", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }
}
