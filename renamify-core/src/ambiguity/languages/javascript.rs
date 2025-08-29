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
}
