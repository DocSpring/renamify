use crate::case_model::Style;

/// Ruby language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("class") || context.ends_with("module") {
        // Classes and modules should be PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("def")
        || context.ends_with("attr_reader")
        || context.ends_with("attr_writer")
        || context.ends_with("attr_accessor")
        || context.ends_with("alias")
        || context.ends_with("alias_method")
    {
        // Methods and attributes should be snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.contains("CONSTANT")
        || context.contains("VERSION")
        || (context.ends_with('=')
            && context
                .chars()
                .all(|c| c.is_uppercase() || c == '_' || c == '=' || c.is_whitespace()))
    {
        // Constants are SCREAMING_SNAKE
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    } else if context.ends_with("require")
        || context.ends_with("require_relative")
        || context.ends_with("load")
        || context.ends_with("autoload")
    {
        // File paths in require statements typically snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("::") {
        // After namespace separator, expect PascalCase for classes
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with('@') || context.ends_with("@@") {
        // Instance and class variables are snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with(':') && !context.ends_with("::") {
        // Symbols are typically snake_case
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
    fn test_ruby_class_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel, Style::Snake];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_ruby_method_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel, Style::Kebab];
        let result = suggest_style("def", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_ruby_constant_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        let result = suggest_style("VERSION =", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_ruby_symbol_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style(":", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_ruby_module_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("module", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_ruby_attr_methods() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("attr_reader", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("attr_writer", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("attr_accessor", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_ruby_alias_methods() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("alias", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("alias_method", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_ruby_constant_with_version() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style("VERSION", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));

        let result = suggest_style("CONSTANT", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_ruby_require_statements() {
        let possible_styles = vec![Style::Snake, Style::Kebab];
        let result = suggest_style("require", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("require_relative", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("load", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("autoload", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_ruby_namespace_separator() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("::", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_ruby_instance_variables() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("@", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("@@", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_ruby_equals_with_uppercase() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        // All uppercase with equals sign
        let result = suggest_style("CONST =", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_ruby_no_matching_style() {
        let possible_styles = vec![Style::Kebab, Style::Title];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, None);

        let result = suggest_style("def", &possible_styles);
        assert_eq!(result, None);
    }
}
