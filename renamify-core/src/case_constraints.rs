//! Case constraints implementation
//!
//! This module provides centralized logic for determining if a piece of text can validly
//! represent a given case style. This is critical for:
//!
//! 1. **Ambiguity resolution** - filtering possible styles based on matched text constraints
//! 2. **Coercion logic** - ensuring we never violate case constraints when coercing
//! 3. **Correctness** - "TESTWORD" must NEVER become "Module" in any context
//!
//! ## Core Principle
//!
//! **Case constraints are HARD CONSTRAINTS that can never be violated.**
//!
//! If text is all uppercase (like "TESTWORD"), it can ONLY match uppercase styles:
//! - ScreamingSnake
//! - ScreamingTrain
//! - UpperFlat
//! - UpperSentence
//!
//! It can NEVER match:
//! - Camel (requires first lowercase)
//! - Pascal (no consecutive uppercase allowed)
//! - Title (requires title pattern per word)
//! - Snake/Kebab (require all lowercase)

use crate::case_model::Style;
use std::collections::HashMap;

/// All known separators in naming conventions
const ALL_SEPARATORS: &[char] = &['_', '-', ' ', '.'];

/// Constraints that define what text can validly represent a style
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleConstraints {
    /// What case pattern the text must follow
    pub case: CaseConstraint,
    /// The separator this style uses (None for no-separator styles)
    pub separator: Option<char>,
}

/// Case patterns that text must follow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseConstraint {
    /// All letters uppercase (HELLO, TESTWORD)
    AllUppercase,

    /// All letters lowercase (hello, testword)
    AllLowercase,

    /// First letter uppercase, rest lowercase (Hello, Testword)
    /// Used for: Title Case words, Train-Case words, Sentence case first word
    TitlePattern,

    /// First letter lowercase, NO consecutive uppercase (hello, helloWorld, testWord)
    /// Used for: camelCase
    CamelPattern,

    /// First letter uppercase, NO consecutive uppercase (Hello, HelloWorld, TestWord)
    /// Used for: PascalCase
    PascalPattern,
}

impl Style {
    /// Get the constraints for this style
    pub const fn constraints(self) -> StyleConstraints {
        match self {
            // Lowercase with separators
            Style::Snake => StyleConstraints {
                case: CaseConstraint::AllLowercase,
                separator: Some('_'),
            },
            Style::Kebab => StyleConstraints {
                case: CaseConstraint::AllLowercase,
                separator: Some('-'),
            },
            Style::Dot => StyleConstraints {
                case: CaseConstraint::AllLowercase,
                separator: Some('.'),
            },
            Style::LowerSentence => StyleConstraints {
                case: CaseConstraint::AllLowercase,
                separator: Some(' '),
            },

            // Uppercase with separators
            Style::ScreamingSnake => StyleConstraints {
                case: CaseConstraint::AllUppercase,
                separator: Some('_'),
            },
            Style::ScreamingTrain => StyleConstraints {
                case: CaseConstraint::AllUppercase,
                separator: Some('-'),
            },
            Style::UpperSentence => StyleConstraints {
                case: CaseConstraint::AllUppercase,
                separator: Some(' '),
            },

            // Title pattern with separators
            Style::Train => StyleConstraints {
                case: CaseConstraint::TitlePattern,
                separator: Some('-'),
            },
            Style::Title => StyleConstraints {
                case: CaseConstraint::TitlePattern,
                separator: Some(' '),
            },

            // Sentence case (first word title, rest lower)
            Style::Sentence => StyleConstraints {
                case: CaseConstraint::TitlePattern, // At least first word follows this
                separator: Some(' '),
            },

            // No separators - case transitions
            Style::Camel => StyleConstraints {
                case: CaseConstraint::CamelPattern,
                separator: None,
            },
            Style::Pascal => StyleConstraints {
                case: CaseConstraint::PascalPattern,
                separator: None,
            },

            // No separators - flat case
            Style::LowerFlat => StyleConstraints {
                case: CaseConstraint::AllLowercase,
                separator: None,
            },
            Style::UpperFlat => StyleConstraints {
                case: CaseConstraint::AllUppercase,
                separator: None,
            },
        }
    }
}

/// Check if text can validly represent the given style
pub fn can_match_style(text: &str, style: Style) -> bool {
    let constraints = style.constraints();
    check_case_constraint(text, constraints.case) && check_separator_constraints(text, &constraints)
}

/// Pre-compute which styles each variant can match
/// Called ONCE during variant map generation for performance
pub fn compute_variant_constraints(
    variants: &[String],
    all_styles: &[Style],
) -> HashMap<String, Vec<Style>> {
    variants
        .iter()
        .map(|variant| {
            let compatible = all_styles
                .iter()
                .filter(|&&style| can_match_style(variant, style))
                .copied()
                .collect();
            (variant.clone(), compatible)
        })
        .collect()
}

/// Filter styles to only those compatible with text
pub fn filter_compatible_styles(text: &str, styles: &[Style]) -> Vec<Style> {
    styles
        .iter()
        .filter(|&&style| can_match_style(text, style))
        .copied()
        .collect()
}

fn check_case_constraint(text: &str, constraint: CaseConstraint) -> bool {
    if text.is_empty() {
        return false;
    }

    let has_upper = text.bytes().any(|b| b.is_ascii_uppercase());
    let has_lower = text.bytes().any(|b| b.is_ascii_lowercase());

    match constraint {
        CaseConstraint::AllUppercase => {
            // Every letter must be uppercase
            !has_lower
        }

        CaseConstraint::AllLowercase => {
            // Every letter must be lowercase
            !has_upper
        }

        CaseConstraint::TitlePattern => {
            // First letter uppercase, rest lowercase
            let mut chars = text.chars();
            let first = chars.next();
            first.is_some_and(|c| c.is_uppercase())
                && chars.all(|c| c.is_lowercase() || !c.is_alphabetic())
        }

        CaseConstraint::CamelPattern => {
            // First letter lowercase, no consecutive uppercase
            let mut chars = text.chars();
            let first = chars.next();
            first.is_some_and(|c| c.is_lowercase()) && !has_consecutive_uppercase(text)
        }

        CaseConstraint::PascalPattern => {
            // First letter uppercase, no consecutive uppercase
            let mut chars = text.chars();
            let first = chars.next();
            first.is_some_and(|c| c.is_uppercase()) && !has_consecutive_uppercase(text)
        }
    }
}

fn has_consecutive_uppercase(text: &str) -> bool {
    // Check if text has consecutive uppercase that would violate camelCase/PascalCase patterns
    // BUT: Allow known acronyms using the existing acronym detection system

    let acronym_set = crate::acronym::get_default_acronym_set();

    // If the entire text is a known acronym, allow it
    if acronym_set.is_acronym(text) {
        return false;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_uppercase() {
            // Found start of uppercase sequence, collect it
            let start = i;
            while i < chars.len() && (chars[i].is_uppercase() || chars[i].is_ascii_digit()) {
                i += 1;
            }

            let sequence: String = chars[start..i].iter().collect();

            if sequence.len() >= 2 {
                // Try to find the longest known acronym from this position
                let mut found_acronym = false;
                for len in (2..=sequence.len()).rev() {
                    let subseq: String = chars[start..start + len].iter().collect();
                    if acronym_set.is_acronym(&subseq) {
                        found_acronym = true;
                        break;
                    }
                }

                // If no acronym found and we have 2+ consecutive uppercase, reject it
                if !found_acronym {
                    return true;
                }
            }
        } else {
            i += 1;
        }
    }

    false
}

fn check_separator_constraints(text: &str, constraints: &StyleConstraints) -> bool {
    // Check that text only contains the allowed separator (if any)
    for &sep in ALL_SEPARATORS {
        let has_sep = text.contains(sep);

        match constraints.separator {
            Some(required) if sep == required => {
                // This is the required separator - OK
                continue;
            }
            _ => {
                // Any other separator is forbidden
                if has_sep {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test string used across all tests
    const TEST_UPPER: &str = "TESTWORD";
    const TEST_LOWER: &str = "testword";
    const TEST_CAMEL: &str = "testWord";
    const TEST_PASCAL: &str = "TestWord";
    const TEST_TITLE: &str = "Test";

    #[test]
    fn test_all_uppercase_constraint() {
        // Should match
        assert!(can_match_style(TEST_UPPER, Style::ScreamingSnake));
        assert!(can_match_style(TEST_UPPER, Style::UpperFlat));
        assert!(can_match_style(TEST_UPPER, Style::ScreamingTrain));
        assert!(can_match_style(TEST_UPPER, Style::UpperSentence));

        // Should NOT match
        assert!(!can_match_style(TEST_UPPER, Style::Snake)); // requires lowercase
        assert!(!can_match_style(TEST_UPPER, Style::Camel)); // requires first lowercase
        assert!(!can_match_style(TEST_UPPER, Style::Pascal)); // no consecutive uppercase
        assert!(!can_match_style(TEST_UPPER, Style::Train)); // requires title pattern
        assert!(!can_match_style(TEST_UPPER, Style::Title)); // requires title pattern
    }

    #[test]
    fn test_all_lowercase_constraint() {
        // Should match
        assert!(can_match_style(TEST_LOWER, Style::Snake));
        assert!(can_match_style(TEST_LOWER, Style::Kebab));
        assert!(can_match_style(TEST_LOWER, Style::LowerFlat));
        assert!(can_match_style(TEST_LOWER, Style::LowerSentence));

        // Should NOT match
        assert!(!can_match_style(TEST_LOWER, Style::ScreamingSnake)); // requires uppercase
        assert!(!can_match_style(TEST_LOWER, Style::Pascal)); // requires first uppercase
        assert!(!can_match_style(TEST_LOWER, Style::Title)); // requires title pattern
    }

    #[test]
    fn test_camel_pattern_constraint() {
        // Should match
        assert!(can_match_style(TEST_CAMEL, Style::Camel));
        assert!(can_match_style("helloWorld", Style::Camel));
        assert!(can_match_style("test", Style::Camel)); // single word, lowercase

        // Should match - with known acronyms
        assert!(can_match_style("getAPIKey", Style::Camel)); // API is known
        assert!(can_match_style("fetchJSONData", Style::Camel)); // JSON is known
        assert!(can_match_style("apiClient", Style::Camel)); // API at start (lowercase a, then API)

        // Should NOT match - consecutive uppercase that's NOT an acronym
        assert!(!can_match_style("testWOrd", Style::Camel)); // WO not an acronym
        assert!(!can_match_style("testWORD", Style::Camel)); // WORD not an acronym
        assert!(!can_match_style(TEST_UPPER, Style::Camel)); // all uppercase

        // Should NOT match - wrong first letter
        assert!(!can_match_style(TEST_PASCAL, Style::Camel)); // starts with uppercase (that's Pascal)
        assert!(!can_match_style("Test", Style::Camel)); // starts with uppercase
    }

    #[test]
    fn test_pascal_pattern_constraint() {
        // Should match
        assert!(can_match_style(TEST_PASCAL, Style::Pascal));
        assert!(can_match_style("HelloWorld", Style::Pascal));
        assert!(can_match_style("Test", Style::Pascal)); // single word, title case

        // Should match - with known acronyms
        assert!(can_match_style("APIClient", Style::Pascal)); // API is a known acronym
        assert!(can_match_style("HTTPSConnection", Style::Pascal)); // HTTPS is known
        assert!(can_match_style("UserAPI", Style::Pascal)); // API at end

        // Should NOT match - consecutive uppercase that's NOT an acronym
        assert!(!can_match_style("TESTWord", Style::Pascal)); // TEST not in acronym list
        assert!(!can_match_style("TestWORD", Style::Pascal)); // WORD not in acronym list
        assert!(!can_match_style(TEST_UPPER, Style::Pascal)); // all uppercase, not an acronym

        // Should NOT match - wrong first letter
        assert!(!can_match_style(TEST_CAMEL, Style::Pascal)); // starts with lowercase (that's Camel)
        assert!(!can_match_style("test", Style::Pascal)); // starts with lowercase
    }

    #[test]
    fn test_title_pattern_constraint() {
        // Should match
        assert!(can_match_style(TEST_TITLE, Style::Train));
        assert!(can_match_style("Hello", Style::Title));
        assert!(can_match_style("Testword", Style::Train));

        // Should NOT match
        assert!(!can_match_style(TEST_UPPER, Style::Train)); // all uppercase
        assert!(!can_match_style(TEST_LOWER, Style::Title)); // all lowercase
        assert!(!can_match_style(TEST_PASCAL, Style::Train)); // W is uppercase (not title pattern for single word)
    }

    #[test]
    fn test_separator_constraints() {
        // Snake requires underscores, forbids others
        assert!(can_match_style("test_word", Style::Snake));
        assert!(!can_match_style("test-word", Style::Snake)); // has hyphen
        assert!(!can_match_style("test word", Style::Snake)); // has space
        assert!(!can_match_style("test.word", Style::Snake)); // has dot

        // Kebab requires hyphens, forbids others
        assert!(can_match_style("test-word", Style::Kebab));
        assert!(!can_match_style("test_word", Style::Kebab)); // has underscore
        assert!(!can_match_style("test word", Style::Kebab)); // has space

        // Camel/Pascal forbid ALL separators
        assert!(can_match_style(TEST_CAMEL, Style::Camel));
        assert!(!can_match_style("test_word", Style::Camel)); // has underscore
        assert!(!can_match_style("test-word", Style::Camel)); // has hyphen
        assert!(!can_match_style("test word", Style::Camel)); // has space
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert!(!can_match_style("", Style::Snake));
        assert!(!can_match_style("", Style::Camel));

        // Single character
        assert!(can_match_style("t", Style::LowerFlat));
        assert!(can_match_style("T", Style::UpperFlat));
        assert!(can_match_style("t", Style::Camel)); // lowercase first = camel
        assert!(can_match_style("T", Style::Pascal)); // uppercase first = pascal

        // Non-alphabetic characters
        assert!(can_match_style("test123", Style::LowerFlat)); // digits don't break lowercase
        assert!(can_match_style("TEST123", Style::UpperFlat)); // digits don't break uppercase
        assert!(can_match_style("test123Word", Style::Camel)); // digits in middle OK
    }

    #[test]
    fn test_filter_compatible_styles() {
        let all_styles = [
            Style::Snake,
            Style::Camel,
            Style::Pascal,
            Style::ScreamingSnake,
            Style::UpperFlat,
        ];

        // All uppercase text should only match uppercase styles
        let compatible = filter_compatible_styles(TEST_UPPER, &all_styles);
        assert!(compatible.contains(&Style::ScreamingSnake));
        assert!(compatible.contains(&Style::UpperFlat));
        assert!(!compatible.contains(&Style::Snake));
        assert!(!compatible.contains(&Style::Camel));
        assert!(!compatible.contains(&Style::Pascal));

        // camelCase should only match Camel
        let compatible = filter_compatible_styles(TEST_CAMEL, &all_styles);
        assert!(compatible.contains(&Style::Camel));
        assert!(!compatible.contains(&Style::Pascal));
        assert!(!compatible.contains(&Style::Snake));
    }
}
