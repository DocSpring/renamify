//! Tests for case coercion constraints
//!
//! This tests that coercion NEVER chooses a case style that doesn't match
//! the original case constraints. For example:
//! - All uppercase can only be coerced to ScreamingSnake, ScreamingTrain, UpperFlat, UpperSentence
//! - All lowercase can only be coerced to snake, kebab, lower_flat, lower sentence
//! - Mixed case can be coerced to camelCase, PascalCase, Title Case, etc.

use renamify_core::coercion::{apply_coercion, detect_style, Style};

const COMMENT_WITH_UPPERCASE: &str = "        Style::UpperSentence,  // TESTWORD DEFAULT STYLES";

#[test]
fn test_uppercase_only_gets_uppercase_coercion() {
    // When we have "TESTWORD" in an all-uppercase context like "TESTWORD DEFAULT STYLES"
    // it should ONLY be coerced to uppercase styles

    let result = apply_coercion(COMMENT_WITH_UPPERCASE, "testword", "module");

    if let Some((coerced, reason)) = result {
        // The replacement should be "MODULE" not "Module" or "module"
        assert!(
            coerced.contains("MODULE"),
            "Expected TESTWORD to be replaced with MODULE (all uppercase), got: {}",
            coerced
        );

        // Should NOT contain mixed case "Module"
        assert!(
            !coerced.contains("Module"),
            "Should not contain Title Case 'Module' in all-uppercase context, got: {}",
            coerced
        );

        // The style should be an uppercase style
        assert!(
            reason.contains("UpperSentence") ||
            reason.contains("UpperFlat") ||
            reason.contains("ScreamingSnake") ||
            reason.contains("ScreamingTrain"),
            "Expected uppercase style, got: {}",
            reason
        );
    }
}

#[test]
fn test_detect_style_uppercase_sentence() {
    // "TESTWORD DEFAULT STYLES" should be detected as UpperSentence
    assert_eq!(
        detect_style("TESTWORD DEFAULT STYLES"),
        Style::UpperSentence,
        "All uppercase with spaces should be UpperSentence"
    );
}

#[test]
fn test_detect_style_screaming_snake() {
    assert_eq!(
        detect_style("TESTWORD_CORE_ENGINE"),
        Style::ScreamingSnake,
        "All uppercase with underscores should be ScreamingSnake"
    );
}

#[test]
fn test_detect_style_screaming_train() {
    assert_eq!(
        detect_style("TESTWORD-CORE-ENGINE"),
        Style::ScreamingTrain,
        "All uppercase with hyphens should be ScreamingTrain"
    );
}

#[test]
fn test_detect_style_upper_flat() {
    assert_eq!(
        detect_style("TESTWORDCOREENGINE"),
        Style::UpperFlat,
        "All uppercase with no separators should be UpperFlat"
    );
}

#[test]
fn test_lowercase_only_gets_lowercase_coercion() {
    // All lowercase should only coerce to lowercase styles
    let container = "testword default styles";
    let result = apply_coercion(container, "testword", "module");

    if let Some((coerced, _)) = result {
        // Should still be all lowercase
        assert_eq!(
            coerced.to_lowercase(),
            coerced,
            "All lowercase input should produce all lowercase output: {}",
            coerced
        );
    }
}

#[test]
fn test_detect_style_lower_sentence() {
    assert_eq!(
        detect_style("testword default styles"),
        Style::LowerSentence,
        "All lowercase with spaces should be LowerSentence"
    );
}

#[test]
fn test_detect_style_snake() {
    assert_eq!(
        detect_style("testword_core_engine"),
        Style::Snake,
        "All lowercase with underscores should be Snake"
    );
}

#[test]
fn test_detect_style_kebab() {
    assert_eq!(
        detect_style("testword-core-engine"),
        Style::Kebab,
        "All lowercase with hyphens should be Kebab"
    );
}

#[test]
fn test_detect_style_lower_flat() {
    assert_eq!(
        detect_style("testwordcoreengine"),
        Style::LowerFlat,
        "All lowercase with no separators should be LowerFlat"
    );
}

#[test]
fn test_title_case_preserved() {
    let container = "Testword Default Styles";
    let result = apply_coercion(container, "testword", "module");

    if let Some((coerced, reason)) = result {
        // Should be Title Case
        assert!(
            reason.contains("Title") || reason.contains("Sentence"),
            "Expected Title or Sentence case, got: {}",
            reason
        );

        // First letter of each word should be uppercase
        assert!(
            coerced.starts_with("Module") || coerced.starts_with("M"),
            "Expected Title Case output, got: {}",
            coerced
        );
    }
}

#[test]
fn test_camel_case_preserved() {
    let container = "testwordCoreEngine";
    let result = apply_coercion(container, "testword", "module");

    if let Some((coerced, reason)) = result {
        assert!(
            reason.contains("Camel"),
            "Expected Camel case, got: {}",
            reason
        );

        // Should be camelCase
        assert!(
            coerced.chars().next().unwrap().is_lowercase(),
            "Expected camelCase (lowercase start), got: {}",
            coerced
        );
    }
}

#[test]
fn test_pascal_case_preserved() {
    let container = "TestwordCoreEngine";
    let result = apply_coercion(container, "testword", "module");

    if let Some((coerced, reason)) = result {
        assert!(
            reason.contains("Pascal"),
            "Expected Pascal case, got: {}",
            reason
        );

        // Should be PascalCase
        assert!(
            coerced.chars().next().unwrap().is_uppercase(),
            "Expected PascalCase (uppercase start), got: {}",
            coerced
        );
    }
}
