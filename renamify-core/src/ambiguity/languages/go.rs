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
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
        }
    } else if context.ends_with("import") || context.contains("import (") {
        // Import paths often use lowercase or snake_case
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
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
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
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
        let possible_styles = vec![Style::Lower, Style::Snake];
        let result = suggest_style("package", &possible_styles);
        assert_eq!(result, Some(Style::Lower));
    }

    #[test]
    fn test_go_var_heuristic() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("var", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }
}
