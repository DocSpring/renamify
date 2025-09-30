use crate::case_model::Style;

/// Java language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("class")
        || context.ends_with("interface")
        || context.ends_with("enum")
        || context.ends_with("@interface")
        || context.ends_with("record")
        || context.ends_with("extends")
        || context.ends_with("implements")
    {
        // Classes, interfaces, records should be PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.contains("static final")
        || (context.ends_with("final") && context.contains("static"))
    {
        // Constants are SCREAMING_SNAKE - check this before general public/private/protected
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    } else if context.contains("private")
        || context.contains("public")
        || context.contains("protected")
        || context.ends_with("void")
        || context.ends_with("return")
        || context.contains("new ")
        || context.ends_with("this.")
        || context.ends_with("super.")
    {
        // Methods and fields typically camelCase
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.ends_with("package") || context.ends_with("import") {
        // Package names are lowercase with dots
        if possible_styles.contains(&Style::LowerJoined) {
            return Some(Style::LowerJoined);
        }
    } else if context.ends_with('@') {
        // Annotations typically PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with('<') || context.contains("<T") || context.contains("extends ") {
        // Generic type parameters typically single uppercase letter or PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("Exception") || context.ends_with("Error") {
        // Exception classes are PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with('.') && !context.ends_with("this.") && !context.ends_with("super.")
    {
        // After dot notation (method calls/field access), typically camelCase
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_class_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_method_heuristic() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("public void", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_constant_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        let result = suggest_style("public static final", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_java_annotation_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("@", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_package_heuristic() {
        let possible_styles = vec![Style::LowerJoined, Style::Snake];
        let result = suggest_style("package", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));
    }

    #[test]
    fn test_java_interface_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("interface", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_enum_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("enum", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_record_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("record", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_annotation_interface() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("@interface", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_extends_implements() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("extends", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("implements", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_import_heuristic() {
        let possible_styles = vec![Style::LowerJoined, Style::Camel];
        let result = suggest_style("import", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));
    }

    #[test]
    fn test_java_modifiers() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("public", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("private", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("protected", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_static_final_reversed() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        // "final static" doesn't contain "static final" so it won't match
        let result = suggest_style("final static", &possible_styles);
        assert_eq!(result, None); // Doesn't match the specific pattern

        // But this should work
        let result = suggest_style("static final", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_java_void_method() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("void", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_return_statement() {
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("return", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_new_keyword() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("new ArrayList", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_this_super() {
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("this.", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        let result = suggest_style("super.", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_generic_types() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("<", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("<T", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("extends List", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_exception_types() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("RuntimeException", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("CustomError", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_java_dot_notation() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("obj.", &possible_styles);
        assert_eq!(result, Some(Style::Camel));

        // But not for this. or super.
        let result = suggest_style("this.", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_java_no_matching_style() {
        let possible_styles = vec![Style::Snake, Style::Kebab];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, None);

        let result = suggest_style("@", &possible_styles);
        assert_eq!(result, None);
    }
}
