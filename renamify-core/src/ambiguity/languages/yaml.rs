use crate::case_model::Style;

/// YAML language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    // YAML keys typically use snake_case or kebab-case
    if context.ends_with(':') && !context.ends_with("::") {
        // Key definitions
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.contains("${{") || context.contains("${") {
        // GitHub Actions or template variables
        if context
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(char::is_uppercase)
            && possible_styles.contains(&Style::ScreamingSnake)
        {
            // Environment variables in templates
            return Some(Style::ScreamingSnake);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("- name:") || context.ends_with("  name:") {
        // Common pattern in CI configs
        if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        } else if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.contains("env:") {
        // Environment variables section
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
    fn test_yaml_key_heuristic() {
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("key:", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_yaml_env_var_heuristic() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style("${{ ENV_VAR", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));
    }
}
