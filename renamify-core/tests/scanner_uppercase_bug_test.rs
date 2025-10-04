//! Integration test for the uppercase replacement bug
//!
//! This test reproduces the actual bug where "TESTWORD" in "TESTWORD CORE ENGINE"
//! gets replaced with "Module" instead of "MODULE".

use renamify_core::scanner::{scan_repository_multi, CoercionMode, PlanOptions};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_uppercase_comment_replacement() {
    // Create a temp directory with a test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    let content = "        Style::UpperSentence,  // TESTWORD CORE ENGINE\n";
    fs::write(&test_file, content).unwrap();

    // Scan with testword -> module
    let options = PlanOptions {
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 2, // Ignore all ignore files
        styles: None,          // Use default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from(".renamify/plan.json"),
        coerce_separators: CoercionMode::Auto,
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        enable_plural_variants: true,
    };

    let result = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "testword",
        "module",
        &options,
    )
    .unwrap();

    // Check the replacement
    assert_eq!(result.matches.len(), 1, "Should find one match");

    let m = &result.matches[0];
    assert_eq!(m.file, test_file);

    // The bug: replacement is "Module CORE ENGINE" but should be "MODULE CORE ENGINE"
    println!("Variant: {}", m.variant);
    println!("Content: {}", m.content);
    println!("Replace: {}", m.replace);
    if let Some(ref before) = m.line_before {
        println!("Line before: {}", before);
    }
    if let Some(ref after) = m.line_after {
        println!("Line after: {}", after);
    }

    assert!(
        m.replace
            .chars()
            .all(|c| !c.is_lowercase() || !c.is_alphabetic()),
        "Replacement should be all uppercase (got: {}), since original was all uppercase ({})",
        m.replace,
        m.content
    );
}

#[test]
fn test_uppercase_sentence_variants_in_map() {
    // Test that the variant map contains the right uppercase variants
    let map = renamify_core::case_model::generate_variant_map("testword", "module", None);

    // Should have TESTWORD -> MODULE (UpperFlat)
    assert_eq!(
        map.get("TESTWORD"),
        Some(&"MODULE".to_string()),
        "Variant map should have TESTWORD -> MODULE"
    );

    // Print all uppercase variants for debugging
    println!("\nUppercase variants in map:");
    for (k, v) in map.iter() {
        if k.chars().all(|c| !c.is_lowercase() || !c.is_alphabetic()) {
            println!("  {} -> {}", k, v);
        }
    }
}

#[test]
fn test_uppercase_comment_with_multiword_replacement() {
    // This tests the ACTUAL e2e bug: single word uppercase -> multi word replacement
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    let content = "        Style::UpperSentence,  // TESTWORD CORE ENGINE\n";
    fs::write(&test_file, content).unwrap();

    // Scan with testword -> config_helper_utility_module (4 words, matches e2e pattern)
    let options = PlanOptions {
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 2,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from(".renamify/plan.json"),
        coerce_separators: CoercionMode::Auto,
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        enable_plural_variants: true,
    };

    let result = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "testword",
        "config_helper_utility_module",
        &options,
    )
    .unwrap();

    assert_eq!(result.matches.len(), 1, "Should find one match");

    let m = &result.matches[0];
    println!("Variant: {}", m.variant);
    println!("Content: {}", m.content);
    println!("Replace: {}", m.replace);

    // The bug: "TESTWORD" should become "CONFIG HELPER UTILITY MODULE" (all uppercase)
    // NOT "ConfigHelperUtilityModule" (PascalCase)
    assert!(
        m.replace
            .chars()
            .all(|c| !c.is_lowercase() || !c.is_alphabetic()),
        "Replacement should be all uppercase (got: {}), since original was all uppercase ({})",
        m.replace,
        m.content
    );

    // Should specifically be "CONFIG HELPER UTILITY MODULE" for UpperSentence style
    assert_eq!(
        m.replace, "CONFIG HELPER UTILITY MODULE",
        "Multi-word replacement should be in UpperSentence style"
    );
}

#[test]
fn test_multiword_variant_map_uppercase() {
    // Test that multi-word variant maps contain proper uppercase variants
    let map = renamify_core::case_model::generate_variant_map(
        "testword",
        "config_helper_utility_module",
        None,
    );

    // TESTWORD should map to CONFIG HELPER UTILITY MODULE (UpperSentence)
    // NOT ConfigHelperUtilityModule (Pascal)
    let uppercase_variant = map.get("TESTWORD");
    println!("TESTWORD maps to: {:?}", uppercase_variant);

    assert!(
        uppercase_variant.is_some(),
        "Variant map should contain TESTWORD"
    );

    let replacement = uppercase_variant.unwrap();
    assert!(
        replacement
            .chars()
            .all(|c| !c.is_lowercase() || !c.is_alphabetic()),
        "TESTWORD should map to an all-uppercase replacement, got: {}",
        replacement
    );
}
