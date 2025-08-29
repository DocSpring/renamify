use crate::case_model::Style;
use std::path::Path;

/// Language-specific heuristics for resolving case style ambiguity
pub struct LanguageHeuristics;

impl LanguageHeuristics {
    /// Apply language-specific heuristics based on file extension and context
    pub fn suggest_style(
        file_path: &Path,
        preceding_context: &str,
        possible_styles: &[Style],
    ) -> Option<Style> {
        if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
            eprintln!("DEBUG LanguageHeuristics: called suggest_style");
        }

        let extension = file_path.extension()?.to_str()?;
        let context = preceding_context.trim();

        match extension {
            "rb" => Self::ruby_heuristics(context, possible_styles),
            "py" => Self::python_heuristics(context, possible_styles),
            "js" | "jsx" | "ts" | "tsx" => Self::javascript_heuristics(context, possible_styles),
            "go" => Self::go_heuristics(context, possible_styles),
            "rs" => Self::rust_heuristics(context, possible_styles),
            "java" => Self::java_heuristics(context, possible_styles),
            "c" | "cpp" | "cc" | "h" | "hpp" => Self::c_cpp_heuristics(context, possible_styles),
            "css" | "scss" | "sass" | "less" => Self::css_heuristics(context, possible_styles),
            "html" | "htm" | "xml" => Self::html_heuristics(context, possible_styles),
            _ => None,
        }
    }

    fn ruby_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("class") || context.ends_with("module") {
            // Classes and modules should be PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if context.ends_with("def") {
            // Methods should be snake_case
            if possible_styles.contains(&Style::Snake) {
                return Some(Style::Snake);
            }
        } else if context.contains("CONSTANT") || context.contains("VERSION") {
            // Constants are SCREAMING_SNAKE
            if possible_styles.contains(&Style::ScreamingSnake) {
                return Some(Style::ScreamingSnake);
            }
        }
        None
    }

    fn python_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("class") {
            // Classes should be PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if context.ends_with("def") {
            // Functions should be snake_case
            if possible_styles.contains(&Style::Snake) {
                return Some(Style::Snake);
            }
        } else if context
            .chars()
            .all(|c| c.is_uppercase() || c == '_' || c == '=')
        {
            // Constants (all caps context)
            if possible_styles.contains(&Style::ScreamingSnake) {
                return Some(Style::ScreamingSnake);
            }
        }
        None
    }

    fn javascript_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("class") || context.ends_with("interface") || context.ends_with("type")
        {
            // Classes, interfaces, and types should be PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if context.ends_with("function")
            || context.ends_with("const")
            || context.ends_with("let")
            || context.ends_with("var")
        {
            // Functions and variables typically camelCase
            if possible_styles.contains(&Style::Camel) {
                return Some(Style::Camel);
            }
        } else if context.contains("export const")
            && possible_styles.contains(&Style::ScreamingSnake)
        {
            // Exported constants might be SCREAMING_SNAKE
            return Some(Style::ScreamingSnake);
        }
        None
    }

    fn go_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("type")
            || context.ends_with("struct")
            || context.ends_with("interface")
        {
            // Types: PascalCase for exported, camelCase for private
            // Since we don't know if it's exported, prefer PascalCase if available
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            } else if possible_styles.contains(&Style::Camel) {
                return Some(Style::Camel);
            }
        } else if context.ends_with("func") {
            // Functions: same as types
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            } else if possible_styles.contains(&Style::Camel) {
                return Some(Style::Camel);
            }
        }
        None
    }

    fn rust_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("struct")
            || context.ends_with("enum")
            || context.ends_with("trait")
            || context.ends_with("impl")
        {
            // Types should be PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if context.ends_with("fn") || context.ends_with("let") || context.ends_with("mut") {
            // Functions and variables should be snake_case
            if possible_styles.contains(&Style::Snake) {
                return Some(Style::Snake);
            }
        } else if context.ends_with("const") || context.ends_with("static") {
            // Constants are SCREAMING_SNAKE
            if possible_styles.contains(&Style::ScreamingSnake) {
                return Some(Style::ScreamingSnake);
            }
        }
        None
    }

    fn java_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("class") || context.ends_with("interface") || context.ends_with("enum")
        {
            // Classes should be PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if context.contains("private")
            || context.contains("public")
            || context.contains("protected")
        {
            // Methods and fields typically camelCase
            if possible_styles.contains(&Style::Camel) {
                return Some(Style::Camel);
            }
        } else if context.ends_with("final") && context.contains("static") {
            // Constants are SCREAMING_SNAKE
            if possible_styles.contains(&Style::ScreamingSnake) {
                return Some(Style::ScreamingSnake);
            }
        }
        None
    }

    fn c_cpp_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        if context.ends_with("class") || context.ends_with("struct") {
            // Classes and structs often PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            }
        } else if context.ends_with("#define") {
            // Macros are SCREAMING_SNAKE
            if possible_styles.contains(&Style::ScreamingSnake) {
                return Some(Style::ScreamingSnake);
            }
        } else if context.contains("typedef") {
            // Typedefs often use snake_case or PascalCase
            if possible_styles.contains(&Style::Pascal) {
                return Some(Style::Pascal);
            } else if possible_styles.contains(&Style::Snake) {
                return Some(Style::Snake);
            }
        }
        None
    }

    fn css_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        // CSS classes and IDs typically use kebab-case
        if (context.ends_with('.') || context.ends_with('#') || context.contains("class="))
            && possible_styles.contains(&Style::Kebab)
        {
            return Some(Style::Kebab);
        }
        None
    }

    fn html_heuristics(context: &str, possible_styles: &[Style]) -> Option<Style> {
        // HTML attributes and data attributes use kebab-case
        if (context.contains("data-") || context.contains("class=") || context.contains("id="))
            && possible_styles.contains(&Style::Kebab)
        {
            return Some(Style::Kebab);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ruby_class_heuristic() {
        let path = PathBuf::from("test.rb");
        let possible_styles = vec![Style::Pascal, Style::Camel, Style::Snake];

        let result = LanguageHeuristics::suggest_style(&path, "class ", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_ruby_method_heuristic() {
        let path = PathBuf::from("test.rb");
        let possible_styles = vec![Style::Snake, Style::Camel, Style::Kebab];

        let result = LanguageHeuristics::suggest_style(&path, "def ", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_python_class_heuristic() {
        let path = PathBuf::from("test.py");
        let possible_styles = vec![Style::Pascal, Style::Camel];

        let result = LanguageHeuristics::suggest_style(&path, "class ", &possible_styles);
        assert_eq!(result, Some(Style::Pascal));
    }

    #[test]
    fn test_javascript_function_heuristic() {
        let path = PathBuf::from("test.js");
        let possible_styles = vec![Style::Camel, Style::Snake, Style::Kebab];

        let result = LanguageHeuristics::suggest_style(&path, "function ", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_no_matching_style() {
        let path = PathBuf::from("test.rb");
        let possible_styles = vec![Style::Camel, Style::Snake]; // No PascalCase

        let result = LanguageHeuristics::suggest_style(&path, "class ", &possible_styles);
        assert_eq!(result, None); // Can't suggest PascalCase if it's not possible
    }
}
