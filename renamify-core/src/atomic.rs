use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ts_rs::TS;

/// Configuration for atomic identifier handling
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[allow(clippy::struct_field_names)]
pub struct AtomicConfig {
    /// Set of identifiers that should be treated as atomic (indivisible)
    pub atomic_identifiers: HashSet<String>,
    /// Whether the search term should be treated as atomic
    pub atomic_search: bool,
    /// Whether the replace term should be treated as atomic
    pub atomic_replace: bool,
}

impl AtomicConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an identifier should be treated as atomic
    /// Performs case-insensitive matching against configured atomic identifiers
    pub fn is_atomic(&self, identifier: &str) -> bool {
        // Check if explicitly marked as atomic via flags
        // This will be checked by the caller based on whether we're checking search or replace

        // Check against configured atomic identifiers (case-insensitive)
        self.atomic_identifiers
            .iter()
            .any(|atomic| atomic.eq_ignore_ascii_case(identifier))
    }

    /// Create config from CLI flags and config file
    pub fn from_flags_and_config(
        atomic_both: bool,
        atomic_search: bool,
        atomic_replace: bool,
        config_atomics: Vec<String>,
    ) -> Self {
        let mut config = Self::new();

        // Add configured atomic identifiers
        for atomic in config_atomics {
            config.atomic_identifiers.insert(atomic);
        }

        // Apply CLI flags
        if atomic_both {
            config.atomic_search = true;
            config.atomic_replace = true;
        } else {
            config.atomic_search = atomic_search;
            config.atomic_replace = atomic_replace;
        }

        config
    }

    /// Check if search term should be treated as atomic
    pub fn should_treat_search_atomic(&self, search: &str) -> bool {
        self.atomic_search || self.is_atomic(search)
    }

    /// Check if replace term should be treated as atomic
    pub fn should_treat_replace_atomic(&self, replace: &str) -> bool {
        self.atomic_replace || self.is_atomic(replace)
    }
}

/// Parse an identifier as atomic (no word boundary detection)
pub fn parse_atomic(s: &str) -> crate::case_model::TokenModel {
    // Treat the entire string as a single token
    crate::case_model::TokenModel::new(vec![crate::case_model::Token::new(s)])
}

/// Convert an atomic token to a specific style
pub fn to_atomic_style(identifier: &str, style: crate::case_model::Style) -> String {
    match style {
        crate::case_model::Style::Snake
        | crate::case_model::Style::Kebab
        | crate::case_model::Style::Dot
        | crate::case_model::Style::LowerFlat
        | crate::case_model::Style::LowerSentence => identifier.to_lowercase(),

        crate::case_model::Style::ScreamingSnake
        | crate::case_model::Style::ScreamingTrain
        | crate::case_model::Style::UpperFlat
        | crate::case_model::Style::UpperSentence => identifier.to_uppercase(),

        crate::case_model::Style::Pascal
        | crate::case_model::Style::Train
        | crate::case_model::Style::Title
        | crate::case_model::Style::Sentence => {
            // Preserve original casing for these styles when atomic
            identifier.to_string()
        },

        crate::case_model::Style::Camel => {
            // For camelCase, lowercase the first letter only
            if identifier.is_empty() {
                String::new()
            } else {
                let mut chars = identifier.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
                }
            }
        },
    }
}

/// Generate variants for an atomic identifier
/// Only generates basic case variants without word boundaries
pub fn generate_atomic_variants(
    identifier: &str,
    styles: &[crate::case_model::Style],
) -> Vec<(String, crate::case_model::Style)> {
    let mut variants = Vec::new();

    for style in styles {
        let variant = to_atomic_style(identifier, *style);
        variants.push((variant, *style));
    }

    variants
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_config_basic() {
        let config = AtomicConfig::from_flags_and_config(
            true, // atomic_both
            false,
            false,
            vec![],
        );

        assert!(config.atomic_search);
        assert!(config.atomic_replace);
    }

    #[test]
    fn test_atomic_config_separate_flags() {
        let config = AtomicConfig::from_flags_and_config(
            false,
            true, // atomic_search
            false,
            vec![],
        );

        assert!(config.atomic_search);
        assert!(!config.atomic_replace);
    }

    #[test]
    fn test_atomic_config_with_identifiers() {
        let config = AtomicConfig::from_flags_and_config(
            false,
            false,
            false,
            vec!["DocSpring".to_string(), "GitHub".to_string()],
        );

        assert!(config.is_atomic("DocSpring"));
        assert!(config.is_atomic("docspring")); // Case insensitive
        assert!(config.is_atomic("DOCSPRING"));
        assert!(!config.is_atomic("RandomName"));
    }

    #[test]
    fn test_should_treat_atomic() {
        let config =
            AtomicConfig::from_flags_and_config(false, false, false, vec!["DocSpring".to_string()]);

        // DocSpring is in the atomic list
        assert!(config.should_treat_search_atomic("DocSpring"));
        assert!(config.should_treat_replace_atomic("docspring"));

        // RandomName is not
        assert!(!config.should_treat_search_atomic("RandomName"));
        assert!(!config.should_treat_replace_atomic("RandomName"));
    }

    #[test]
    fn test_parse_atomic() {
        let tokens = parse_atomic("DocSpring");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "DocSpring");

        let tokens = parse_atomic("GitHub");
        assert_eq!(tokens.tokens.len(), 1);
        assert_eq!(tokens.tokens[0].text, "GitHub");
    }

    #[test]
    fn test_generate_atomic_variants() {
        use crate::case_model::Style;

        let styles = vec![
            Style::Snake,
            Style::Kebab,
            Style::Pascal,
            Style::ScreamingSnake,
        ];

        let variants = generate_atomic_variants("DocSpring", &styles);

        // Check we get the right variants
        assert!(variants.contains(&("docspring".to_string(), Style::Snake)));
        assert!(variants.contains(&("docspring".to_string(), Style::Kebab)));
        assert!(variants.contains(&("DocSpring".to_string(), Style::Pascal)));
        assert!(variants.contains(&("DOCSPRING".to_string(), Style::ScreamingSnake)));

        // Should NOT contain word-separated variants
        let variant_strings: Vec<String> = variants.iter().map(|(s, _)| s.clone()).collect();
        assert!(!variant_strings.contains(&"doc_spring".to_string()));
        assert!(!variant_strings.contains(&"doc-spring".to_string()));
    }

    #[test]
    fn test_atomic_camel_case() {
        use crate::case_model::Style;

        let variants = generate_atomic_variants("DocSpring", &[Style::Camel]);
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].0, "docSpring");
    }
}
