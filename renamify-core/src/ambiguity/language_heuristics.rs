use crate::case_model::Style;
use std::path::Path;

use super::languages;

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
            // Programming languages
            "rb" | "rake" | "gemspec" => languages::ruby::suggest_style(context, possible_styles),
            "py" | "pyw" | "pyi" => languages::python::suggest_style(context, possible_styles),
            "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => {
                languages::javascript::suggest_style(context, possible_styles)
            },
            "go" => languages::go::suggest_style(context, possible_styles),
            "rs" => languages::rust::suggest_style(context, possible_styles),
            "java" | "kt" | "kts" => languages::java::suggest_style(context, possible_styles),
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" => {
                languages::c_cpp::suggest_style(context, possible_styles)
            },

            // Web technologies
            "css" | "scss" | "sass" | "less" | "styl" => {
                languages::css::suggest_style(context, possible_styles)
            },
            "html" | "htm" | "xml" | "svg" | "vue" => {
                languages::html::suggest_style(context, possible_styles)
            },

            // Shell and scripting
            "sh" | "bash" | "zsh" | "fish" | "ksh" => {
                languages::shell::suggest_style(context, possible_styles)
            },

            // Configuration files
            "yml" | "yaml" => languages::yaml::suggest_style(context, possible_styles),
            "json" | "jsonc" | "json5" | "toml" | "ini" | "cfg" | "conf" | "env" => {
                languages::config::suggest_style(context, possible_styles)
            },

            _ => None,
        }
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
        let possible_styles = vec![Style::Camel, Style::Snake];

        let result = LanguageHeuristics::suggest_style(&path, "function ", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_no_matching_style() {
        let path = PathBuf::from("test.py");
        let possible_styles = vec![Style::Kebab]; // Python doesn't use kebab

        let result = LanguageHeuristics::suggest_style(&path, "def ", &possible_styles);
        assert_eq!(result, None);
    }

    #[test]
    fn test_unsupported_extension() {
        let path = PathBuf::from("test.xyz");
        let possible_styles = vec![Style::Snake, Style::Camel];

        let result = LanguageHeuristics::suggest_style(&path, "function ", &possible_styles);
        assert_eq!(result, None);
    }

    #[test]
    fn test_shell_export_heuristic() {
        let path = PathBuf::from("script.sh");
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];

        // Note: "export " gets trimmed to "export" in suggest_style
        let result = LanguageHeuristics::suggest_style(&path, "export ", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }

    #[test]
    fn test_yaml_key_heuristic() {
        let path = PathBuf::from("config.yml");
        let possible_styles = vec![Style::Snake, Style::Camel];

        let result = LanguageHeuristics::suggest_style(&path, "key:", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }
}
