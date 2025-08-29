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
}
