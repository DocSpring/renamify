use crate::case_model::Style;

/// C/C++ language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    // Check typedef FIRST (more specific)
    if context.contains("typedef") {
        // Typedefs often use snake_case or PascalCase
        if context.contains("typedef struct") || context.contains("typedef enum") {
            // C-style typedefs often snake_case
            if possible_styles.contains(&Style::Snake) {
                return Some(Style::Snake);
            } else if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("class")
        || context.ends_with("struct")
        || context.ends_with("union")
    {
        // Classes and structs often PascalCase (C++ style)
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Snake) {
            // C style often uses snake_case for structs
            return Some(Style::Snake);
        }
    } else if context.ends_with("#define") || context.contains("#define ") {
        // Macros are SCREAMING_SNAKE
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    } else if context.ends_with("namespace") {
        // Namespaces (C++) typically lowercase or PascalCase
        if possible_styles.contains(&Style::LowerJoined) {
            return Some(Style::LowerJoined);
        } else if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("enum") {
        // Enum types often PascalCase or snake_case
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.contains("const ") || context.ends_with("constexpr") {
        // Constants might be SCREAMING_SNAKE or regular case
        if context
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(char::is_uppercase)
            && possible_styles.contains(&Style::ScreamingSnake)
        {
            return Some(Style::ScreamingSnake);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("template<") || context.contains("typename ") {
        // Template parameters typically PascalCase
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        }
    } else if context.ends_with("::") {
        // After scope resolution, could be class (Pascal) or namespace member
        if possible_styles.contains(&Style::Pascal) {
            return Some(Style::Pascal);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("#include")
        || context.contains("#include <")
        || context.contains("#include \"")
    {
        // Include files often snake_case or lowercase
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::LowerJoined) {
            return Some(Style::LowerJoined);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_class_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_c_define_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Pascal];
        let result = suggest_style("#define", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_c_typedef_struct_heuristic() {
        // Only provide Snake to ensure it's selected
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("typedef struct", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_cpp_namespace_heuristic() {
        let possible_styles = vec![Style::LowerJoined, Style::Pascal];
        let result = suggest_style("namespace", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));
    }

    #[test]
    fn test_c_struct_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("struct", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with only Snake available
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("struct", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_c_union_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("union", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_c_enum_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("enum", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with only Snake available
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("enum", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_cpp_template_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style("template<", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        let result = suggest_style("typename ", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_cpp_scope_resolution_heuristic() {
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("std::", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // Test with only Snake available
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("std::", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_c_include_heuristic() {
        let possible_styles = vec![Style::Snake, Style::LowerJoined];
        let result = suggest_style("#include", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("#include <", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("#include \"", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Test with only Lower available
        let possible_styles = vec![Style::LowerJoined];
        let result = suggest_style("#include", &possible_styles);
        assert_eq!(result, Some(Style::LowerJoined));
    }

    #[test]
    fn test_c_const_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        // "const " contains a space, so not all chars are uppercase
        let result = suggest_style("const ", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Test with all caps context - but "CONST" doesn't contain "const " or end with "constexpr"
        // so it won't match the const heuristic at all
        let result = suggest_style("CONST", &possible_styles);
        assert_eq!(result, None); // Doesn't match any const pattern

        let result = suggest_style("constexpr", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_c_typedef_variations() {
        // typedef struct prefers snake_case when both are available
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("typedef struct", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // typedef enum prefers snake_case when both are available
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("typedef enum", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // When only Pascal is available for typedef struct
        let possible_styles = vec![Style::Pascal];
        let result = suggest_style("typedef struct", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // plain typedef prefers Pascal when both are available
        let possible_styles = vec![Style::Pascal, Style::Snake];
        let result = suggest_style("typedef", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));

        // With only Snake available for plain typedef
        let possible_styles = vec![Style::Snake];
        let result = suggest_style("typedef", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_cpp_namespace_pascal_only() {
        let possible_styles = vec![Style::Pascal];
        let result = suggest_style("namespace", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_no_matching_style() {
        let possible_styles = vec![Style::Kebab, Style::Title];
        let result = suggest_style("class", &possible_styles);
        assert_eq!(result, None);
    }
}
