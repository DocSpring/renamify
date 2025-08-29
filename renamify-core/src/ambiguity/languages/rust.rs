use crate::case_model::Style;

/// Rust language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("struct")
        || context.ends_with("enum")
        || context.ends_with("trait")
        || context.ends_with("impl")
        || context.ends_with("type")
        || context.contains("impl<")
        || context.contains("trait ")
    {
        // Types should be PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("fn")
        || context.ends_with("let")
        || context.ends_with("mut")
        || context.ends_with("mod")
        || context.ends_with("use")
        || context.ends_with("pub fn")
        || context.ends_with("async fn")
        || context.ends_with("unsafe fn")
    {
        // Functions, variables, modules, and use statements should be snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("const") || context.ends_with("static") {
        // Constants are SCREAMING_SNAKE
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    } else if context.ends_with("macro_rules!") || context.contains("macro_rules! ") {
        // Macros are snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("crate::")
        || context.ends_with("self::")
        || context.ends_with("super::")
    {
        // Module paths are snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("::")
        && !context.ends_with("crate::")
        && !context.ends_with("self::")
        && !context.ends_with("super::")
    {
        // After :: (not module paths), likely a type
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with('\'') {
        // Lifetime parameters are lowercase
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
        }
    } else if context.ends_with("#[") || context.ends_with("#![") {
        // Attributes are typically snake_case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("cfg(") || context.ends_with("feature") {
        // Config flags and features are snake_case
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
    fn test_rust_struct_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("struct", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_rust_function_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("fn", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_const_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        let result = suggest_style("const", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_rust_module_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("mod", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_macro_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("macro_rules!", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }
}
