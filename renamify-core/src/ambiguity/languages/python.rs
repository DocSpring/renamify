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

    #[test]
    fn test_python_lambda_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("lambda", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_async_def_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("async def", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_import_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("import", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("from", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("import numpy", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_dunder_methods() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("__init__", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("__name__", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_exception_classes() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        // "ValueError" ends with "Error" so it matches
        let result = suggest_style("ValueError", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("CustomException", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("MyError", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_python_variable_assignment() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("x =", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("my_var=", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_constants_context() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        // "MAX_VALUE" has uppercase M, A, X, V, L, U, E - all alphabetic chars are uppercase
        let result = suggest_style("MAX_VALUE", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));

        let result = suggest_style("PI", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_python_class_in_context() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        // Contains "class" but doesn't end with it
        let result = suggest_style("class MyClass:", &possible_styles);
        assert_eq!(result, None); // Doesn't end with "class"

        // Assignment with class/def should default to snake_case
        let result = suggest_style("my_var =", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_no_matching_style() {
        let possible_styles = vec![Style::Kebab, Style::Train];
        let result = suggest_style("def", &possible_styles);
        assert_eq!(result, None);

        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, None);
    }
}
