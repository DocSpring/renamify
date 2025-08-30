use crate::case_model::Style;

/// JavaScript/TypeScript language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("class")
        || context.ends_with("interface")
        || context.ends_with("type")
        || context.ends_with("enum")
        || context.ends_with("namespace")
    {
        // Classes, interfaces, enums, types, and namespaces should be PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("function")
        || context.ends_with("const")
        || context.ends_with("let")
        || context.ends_with("var")
        || context.ends_with("async")
        || context.ends_with("await")
        || context.ends_with("=>")  // Arrow functions
        || context.ends_with("get")   // Getters
        || context.ends_with("set")
    {
        // Setters
        // Functions and variables typically camelCase
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.contains("const") && possible_styles.contains(&Style::ScreamingSnake) {
        // Check if this looks like an all-caps constant
        let after_const = context.split("const").last().unwrap_or("");
        if after_const
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(char::is_uppercase)
            && after_const.chars().any(char::is_alphabetic)
        {
            // Constants in all caps context are SCREAMING_SNAKE
            return Some(Style::ScreamingSnake);
        }
    } else if context.ends_with("import")
        || context.ends_with("from")
        || context.ends_with("require(")
        || context.contains("require('")
        || context.contains("require(\"")
    {
        // Module names often camelCase or kebab-case
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with("extends") || context.ends_with("implements") {
        // After extends/implements, expect PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with('#') {
        // Private fields in JS are typically camelCase
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.ends_with('$') {
        // jQuery or Observable patterns often camelCase
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.contains("process.env.") {
        // Environment variables are SCREAMING_SNAKE
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_class_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_function_heuristic() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("function", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_const_caps_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Camel];
        let result = suggest_style("const MAX_VALUE", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_javascript_env_var_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Camel];
        let result = suggest_style("process.env.", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_javascript_interface_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("interface", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_type_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("type", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_enum_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("enum", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_namespace_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("namespace", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_variables() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("let", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("var", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("const", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_async_await() {
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("async", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("await", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_arrow_function() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("=>", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_getters_setters() {
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("get", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("set", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_import_from() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("import", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("from", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        // Kebab fallback
        let possible_styles = vec![Style::Kebab, Style::Pascal];
        let result = suggest_style("import", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_javascript_require() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("require(", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("require('", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("require(\"", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_extends_implements() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("extends", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("implements", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_private_field() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("#", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_jquery_observable() {
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("$", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_const_not_caps() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Camel];
        // "const myVar" has lowercase letters so won't match the SCREAMING_SNAKE check
        let result = suggest_style("const myVar", &possible_styles);
        assert_eq!(result, None); // Doesn't match any specific pattern

        // But plain "const" matches camelCase
        let result = suggest_style("const", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_javascript_no_matching_style() {
        let possible_styles = vec![Style::Snake, Style::Kebab];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, None);

        let result = suggest_style("function", &possible_styles);
        assert_eq!(result, None);
    }
}
