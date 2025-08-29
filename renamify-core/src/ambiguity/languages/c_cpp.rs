use crate::case_model::Style;

/// C/C++ language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    if context.ends_with("class") || context.ends_with("struct") || context.ends_with("union") {
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
    } else if context.contains("typedef") {
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
    } else if context.ends_with("namespace") {
        // Namespaces (C++) typically lowercase or PascalCase
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
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
        } else if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
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
        let possible_styles = vec![Style::Lower, Style::Pascal];
        let result = suggest_style("namespace", &possible_styles);
        assert_eq!(result, Some(Style::Lower));
    }
}
