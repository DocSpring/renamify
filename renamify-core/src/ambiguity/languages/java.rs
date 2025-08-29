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
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
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
        let possible_styles = vec![Style::Lower, Style::Snake];
        let result = suggest_style("package", &possible_styles);
        assert_eq!(result, Some(Style::Lower));
    }
}
