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
        if possible_styles.contains(&Style::LowerFlat) {
            return Some(Style::LowerFlat);
        } else if possible_styles.contains(&Style::Kebab) {
            // Custom elements use kebab-case
            return Some(Style::Kebab);
        }
    } else if context.contains("xmlns:") || context.contains("xml:") {
        // XML namespaces often lowercase or camelCase
        if possible_styles.contains(&Style::LowerFlat) {
            return Some(Style::LowerFlat);
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
        let possible_styles = vec![Style::LowerFlat, Style::Kebab];
        let result = suggest_style("<", &possible_styles);
        assert_eq!(result, Some(Style::LowerFlat));
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

    #[test]
    fn test_html_aria_attribute() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        let result = suggest_style("aria-", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));

        // No kebab available
        let possible_styles = vec![Style::Snake, Style::Camel];
        let result = suggest_style("aria-", &possible_styles);
        assert_eq!(result, None);
    }

    #[test]
    fn test_html_closing_tag() {
        let possible_styles = vec![Style::LowerFlat, Style::Kebab];
        let result = suggest_style("</", &possible_styles);
        assert_eq!(result, Some(Style::LowerFlat));

        // Kebab fallback for custom elements
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("<", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_xml_namespace() {
        let possible_styles = vec![Style::LowerFlat, Style::Camel];
        let result = suggest_style("xmlns:", &possible_styles);
        assert_eq!(result, Some(Style::LowerFlat));

        let result = suggest_style("xml:", &possible_styles);
        assert_eq!(result, Some(Style::LowerFlat));

        // Camel fallback
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("xmlns:", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_alpine_directive() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        let result = suggest_style("x-", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_angular_directive() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("ng-", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));

        let result = suggest_style("*ng", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));

        // Camel fallback
        let possible_styles = vec![Style::Camel, Style::Pascal];
        let result = suggest_style("ng-", &possible_styles);
        assert_eq!(result, Some(Style::Camel));
    }

    #[test]
    fn test_html_id_attribute() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        let result = suggest_style("id=", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_html_class_single_quote() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("class='", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_html_generic_attribute() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        // Generic attribute value
        let result = suggest_style("=\"", &possible_styles);
        assert_eq!(result, None); // No specific style for generic attributes

        let result = suggest_style("='", &possible_styles);
        assert_eq!(result, None);
    }

    #[test]
    fn test_html_no_matching_style() {
        let possible_styles = vec![Style::Pascal, Style::ScreamingSnake];
        let result = suggest_style("data-", &possible_styles);
        assert_eq!(result, None);
    }
}
