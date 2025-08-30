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

    #[test]
    fn test_yaml_key_kebab_fallback() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("key:", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));

        // Test colon but not double colon
        let result = suggest_style("::", &possible_styles);
        assert_eq!(result, None);
    }

    #[test]
    fn test_yaml_template_var() {
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        // Dollar brace syntax - no alphabetic chars means all uppercase check passes
        let result = suggest_style("${", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));

        // With lowercase context
        let result = suggest_style("${env", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Snake fallback when screaming not available
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("${", &possible_styles);
        assert_eq!(result, Some(Style::Snake));
    }

    #[test]
    fn test_yaml_name_field() {
        let possible_styles = vec![Style::Snake, Style::Pascal];
        let result = suggest_style("- name:", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        let result = suggest_style("  name:", &possible_styles);
        assert_eq!(result, Some(Style::Snake));

        // Kebab fallback
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("- name:", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_yaml_env_section() {
        // Note: "env:" only contains lowercase letters, so it won't match the ScreamingSnake pattern
        // The env: check happens after the template check, and returns ScreamingSnake unconditionally when it contains "env:"
        let possible_styles = vec![Style::ScreamingSnake, Style::Snake];
        let result = suggest_style("containing env: somewhere", &possible_styles);
        assert_eq!(result, Some(Style::ScreamingSnake));

        // No screaming snake available - but "env:" doesn't match because it ends with ":"
        // which is caught by the first check for keys
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("env:", &possible_styles);
        assert_eq!(result, Some(Style::Snake)); // Matches key pattern first
    }

    #[test]
    fn test_yaml_no_matching_style() {
        let possible_styles = vec![Style::Pascal, Style::Camel];
        let result = suggest_style(":", &possible_styles);
        assert_eq!(result, None);
    }
}
