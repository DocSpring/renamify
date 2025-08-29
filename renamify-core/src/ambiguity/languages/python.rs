use crate::case_model::Style;

/// Python language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("class") {
        // Classes should be PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("def")
        || context.ends_with("lambda")
        || context.ends_with("async def")
    {
        // Functions should be snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context
        .chars()
        .all(|c| c.is_uppercase() || c == '_' || c == '=' || c.is_whitespace())
    {
        // Constants (all caps context)
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    } else if context.ends_with("import")
        || context.ends_with("from")
        || context.contains("import ")
    {
        // Module names are typically snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with('@') {
        // Decorators are typically snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("__") && context.starts_with("__") {
        // Dunder methods are snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("Exception") || context.ends_with("Error") {
        // Exception classes are PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with('=') && !context.contains("class") && !context.contains("def") {
        // Variable assignments default to snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_class_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_python_function_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("def", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_constant_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        let result = suggest_style("MAX_SIZE =", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_python_decorator_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("@", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }
}
