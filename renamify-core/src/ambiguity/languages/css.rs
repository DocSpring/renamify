use crate::case_model::Style;

/// CSS/SCSS/SASS/LESS language-specific heuristics for resolving case style ambiguity
pub fn suggest_style(context: &str, possible_styles: &[Style]) -> Option<Style> {
    // CSS classes and IDs typically use kebab-case
    if (context.ends_with('.') || context.ends_with('#') || context.contains("class="))
        && possible_styles.contains(&Style::Kebab)
    {
        return Some(Style::Kebab);
    } else if context.ends_with('$') || context.ends_with('@') {
        // SASS/SCSS variables ($) and LESS variables (@) often kebab-case or snake_case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with("--") {
        // CSS custom properties (CSS variables) use kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with("@mixin")
        || context.ends_with("@include")
        || context.ends_with("@function")
    {
        // SASS/SCSS mixins and functions often kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        } else if possible_styles.contains(&Style::Snake) {
            return Some(Style::Snake);
        }
    } else if context.ends_with('%') {
        // SASS placeholder selectors often kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.contains('[') && context.contains('=') {
        // Attribute selectors often kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    } else if context.ends_with("data-") || context.ends_with("aria-") {
        // Data and ARIA attributes use kebab-case
        if possible_styles.contains(&Style::Kebab) {
            return Some(Style::Kebab);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_class_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style(".", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_css_id_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        let result = suggest_style("#", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_scss_variable_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Camel];
        let result = suggest_style("$", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }

    #[test]
    fn test_css_custom_property_heuristic() {
        let possible_styles = vec![Style::Kebab, Style::Snake];
        let result = suggest_style("--", &possible_styles);
        assert_eq!(result, Some(Style::Kebab));
    }
}
