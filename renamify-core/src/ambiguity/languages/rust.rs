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
        if possible_styles.contains(&Style::LowerFlat) {
            return Some(Style::LowerFlat);
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
    } else if context.ends_with("//") || context.ends_with("///") || context.ends_with("/*") {
        // Comments are natural language - prefer sentence styles
        // Check case by seeing what uppercase styles are available
        if possible_styles.contains(&Style::UpperSentence) {
            return Some(Style::UpperSentence);
        } else if possible_styles.contains(&Style::Sentence) {
            return Some(Style::Sentence);
        } else if possible_styles.contains(&Style::LowerSentence) {
            return Some(Style::LowerSentence);
        } else if possible_styles.contains(&Style::Title) {
            return Some(Style::Title);
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

    #[test]
    fn test_rust_enum_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("enum", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_rust_trait_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("trait", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with "trait " (with space)
        let result = suggest_style("trait MyTrait", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_rust_impl_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("impl", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with generic impl
        let result = suggest_style("impl<T>", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_rust_type_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("type", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_rust_let_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("let", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_mut_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("mut", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_use_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("use", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_pub_fn_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("pub fn", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_async_fn_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("async fn", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_unsafe_fn_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("unsafe fn", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_static_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style("static", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_rust_macro_rules_with_space() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("macro_rules! my_macro", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_module_paths() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("crate::", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("self::", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("super::", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_type_path() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        // After :: that's not a module path
        let result = suggest_style("std::collections::", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_rust_lifetime() {
        let possible_styles = vec![Style::LowerFlat, Style::Camel];
        let result = suggest_style("'", &possible_styles);
        assert_eq!(result, Some(Style::LowerFlat));
    }

    #[test]
    fn test_rust_attributes() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("#[", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("#![", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_cfg() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("cfg(", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_feature() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("feature", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_rust_no_matching_fallback() {
        // When preferred style not available
        let possible_styles = vec![Style::Camel];
        let result = suggest_style("struct", &possible_styles);
        assert_eq!(result, None); // No Pascal available

        let possible_styles = vec![Style::Pascal];
        let result = suggest_style("fn", &possible_styles);
        assert_eq!(result, None); // No Snake available
    }
}
