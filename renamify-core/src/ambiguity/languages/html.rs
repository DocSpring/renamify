use crate::case_model::Style;

/// HTML/XML language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    // HTML attributes and data attributes use kebab-case
    if (context.contains("data-") || context.contains("class=") || context.contains("id="))
        && possible_styles.contains(&Style::Kebab)
    {
        return Some(Style::Kebab);
    } else if context.ends_with("aria-") {
        // ARIA attributes always kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with('<') || context.ends_with("</") {
        // HTML tags are lowercase
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
        } else if possible_styles.contains(&Style::Kebab) {
            // Custom elements use kebab-case
            return Some(Style::Kebab);
        }
    } else if context.contains("xmlns:") || context.contains("xml:") {
        // XML namespaces often lowercase or camelCase
        if possible_styles.contains(&Style::Lower) {
            return Some(Style::Lower);
        } else if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.ends_with("v-") || context.ends_with("x-") {
        // Vue directives (v-) and Alpine directives (x-) use kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with("ng-") || context.ends_with("*ng") {
        // Angular directives use kebab-case or camelCase
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        } else if possible_styles.contains(&Style::Camel) {
            return Some(Style::Camel);
        }
    } else if context.contains("=\"") || context.contains("='") {
        // Attribute values can be various styles, default to kebab-case for CSS classes
        if (context.contains("class=\"") || context.contains("class='"))
            && possible_styles.contains(&Style::Kebab)
        {
            return Some(Style::Kebab);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_data_attribute_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("data-", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_html_tag_heuristic() {
        let possible_styles = vec![Style::Lower, Style::Kebab];
        let result = suggest_style("<", &possible_styles);
        assert_eq!(result, Some(Style::Lower));
    }

    #[test]
    fn test_html_class_attribute_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        let result = suggest_style("class=\"", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_vue_directive_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("v-", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }
}
