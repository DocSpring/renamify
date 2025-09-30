use crate::case_model::Style;

/// Go language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    // Check if this is likely an exported identifier (starts with uppercase)
    // Not currently used but kept for potential future use
    let _is_exported_context = context
        .trim()
        .chars()
        .last()
        .is_some_and(char::is_uppercase);

    if context.ends_with("type") || context.ends_with("struct") || context.ends_with("interface") {
        // Types: PascalCase for exported, camelCase for private
        // Prefer PascalCase when both are available (more common for exported types)
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.ends_with("func") || context.contains("func (") {
        // Functions: PascalCase for exported, camelCase for private
        // Prefer PascalCase when both are available (more common for exported funcs)
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.ends_with("const") || context.ends_with("var") {
        // Variables and constants: camelCase preferred
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.ends_with("package") {
        // Package names are lowercase
        if possible_styles.contains(&Style::LowerJoined) {
            return Some(Style::LowerJoined);
        }
    } else if context.ends_with("import") || context.contains("import (") {
        // Import paths often use lowercase or snake_case
        if possible_styles.contains(&Style::LowerJoined) {
            return Some(Style::LowerJoined);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("error") || context.ends_with("Error") {
        // Error types are typically PascalCase if exported
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.contains("//go:") || context.contains("// +build") {
        // Go directives and build tags often use lowercase
        if possible_styles.contains(&Style::LowerJoined) {
            return Some(Style::LowerJoined);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_type_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("type", &possible_styles);
        // Should prefer Pascal for types
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_go_func_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("func", &possible_styles);
        // Should prefer Pascal for potentially exported funcs
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_go_package_heuristic() {
        let possible_styles = vec![Style::LowerJoined, Style::Snake];
        let result = suggest_style("package", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));
    }

    #[test]
    fn test_go_var_heuristic() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("var", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_go_struct_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("struct", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with only Camel available
        let possible_styles = vec![Style::Camel];
        let result = suggest_style("struct", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_go_interface_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("interface", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_go_const_heuristic() {
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("const", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_go_import_heuristic() {
        let possible_styles = vec![Style::LowerJoined, Style::Snake];
        let result = suggest_style("import", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));

        let result = suggest_style("import (", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));

        // Test with only Snake available
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("import", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_go_error_type_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("error", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("Error", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with only Camel available
        let possible_styles = vec![Style::Camel];
        let result = suggest_style("error", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_go_build_directive_heuristic() {
        let possible_styles = vec![Style::LowerJoined, Style::Camel];
        let result = suggest_style("//go:", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));

        let result = suggest_style("// +build", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));
    }

    #[test]
    fn test_go_method_receiver_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("func (", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with only Camel available
        let possible_styles = vec![Style::Camel];
        let result = suggest_style("func (", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_no_matching_style() {
        let possible_styles = vec![Style::Kebab, Style::Title];
        let result = suggest_style("type", &possible_styles);
        assert_eq!(result, None);
    }
}
