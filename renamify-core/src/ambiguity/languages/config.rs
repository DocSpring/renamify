use crate::case_model::Style;

/// Configuration file heuristics (JSON, TOML, INI, etc.)
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    // JSON keys often use camelCase or snake_case
    if context.contains("\":") || (context.ends_with('\"') && context.contains(':')) {
        // JSON key
        if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.starts_with('[') && context.ends_with(']') {
        // TOML/INI section headers often snake_case or kebab-case
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with('=') && !context.contains('\"') {
        // TOML/INI key-value pairs
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.contains(".env") || context.contains("dotenv") {
        // .env files use SCREAMING_SNAKE
        if possible_styles.contains(&Style::ScreamingSnake) {
            return Some(Style::ScreamingSnake);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_key_heuristic() {
        let possible_styles = vec![Style::Camel, Style::Snake];
        let result = suggest_style("\"key\":", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_toml_section_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("[section]", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_env_file_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style(".env", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }
}
