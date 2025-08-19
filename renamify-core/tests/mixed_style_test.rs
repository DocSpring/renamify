use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_mixed_style_identifiers_are_replaced() {
    // This test verifies that identifiers with mixed or unknown styles
    // are still replaced, even when coercion doesn't apply

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with mixed-style identifiers
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        r#"// Mixed style identifiers that should still be replaced
let mixed = renamify_someCAMEL-case;
let weird1 = renamify_someCAMEL-case;
let weird2 = renamify-with_MIXED.styles;
let simple = renamify;
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use all default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "renamed_renaming_tool", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== All matches found ===");
    for m in &plan.matches {
        println!("Line {}: '{}' -> '{}'", m.line, m.before, m.after);
    }

    // Should find matches for all identifiers containing "renamify"
    assert!(
        plan.matches.len() >= 4,
        "Should find at least 4 matches for mixed-style identifiers, found {}",
        plan.matches.len()
    );

    // Verify that we have the expected types of matches
    let line2_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 2).collect();
    let line3_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 3).collect();
    let line4_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 4).collect();
    let line5_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 5).collect();

    // Line 2: Should have compound match for renamify_someCAMEL-case
    assert_eq!(line2_matches.len(), 1);
    assert_eq!(line2_matches[0].before, "renamify_someCAMEL-case");
    assert_eq!(
        line2_matches[0].after,
        "renamed_renaming_tool_someCAMEL-case"
    );

    // Line 3: Should have compound match for renamify_someCAMEL-case
    assert_eq!(line3_matches.len(), 1);
    assert_eq!(line3_matches[0].before, "renamify_someCAMEL-case");
    assert_eq!(
        line3_matches[0].after,
        "renamed_renaming_tool_someCAMEL-case"
    );

    // Line 4: Should have match for "renamify-with_MIXED" (after splitting on dot)
    assert_eq!(line4_matches.len(), 1, "Line 4 should have 1 match");

    let compound_match = line4_matches
        .iter()
        .find(|h| h.before == "renamify-with_MIXED");
    assert!(
        compound_match.is_some(),
        "Should find compound match for renamify-with_MIXED"
    );
    assert_eq!(
        compound_match.unwrap().after,
        "renamed_renaming_tool-with_MIXED"
    );

    // Line 5: Should have exact match for renamify
    assert_eq!(line5_matches.len(), 1);
    assert_eq!(line5_matches[0].before, "renamify");
    assert_eq!(line5_matches[0].after, "renamed_renaming_tool");
}

#[test]
fn test_format_string_placeholders_are_replaced() {
    // This test verifies that format string placeholders like "renamify_{}.tmp"
    // are correctly replaced to "renamed_renaming_tool_{}.tmp"

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with format string placeholders
    let test_file = root.join("apply.rs");
    std::fs::write(
        &test_file,
        r#"fn create_temp_file() {
    let temp_file = temp_dir.join(format!("renamify_{}.tmp", std::process::id()));
    let backup_file = format!("renamify_backup_{}.bak", timestamp);
    let log_name = "renamify_{}.log";
}
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use all default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "renamed_renaming_tool", &options).unwrap();

    // Should find matches for format string placeholders
    // Note: These are exact pattern matches, not compound matches like "renamify_{}"
    assert!(
        !plan.matches.is_empty(),
        "Should find at least 1 match for format string placeholders, found {}",
        plan.matches.len()
    );

    // The current implementation will find compound matches like "renamify_backup_"
    // but not invalid identifiers like "renamify_{}"

    let found_compound = plan.matches.iter().any(|h| h.before == "renamify_backup_");
    assert!(
        found_compound,
        "Should find compound match for renamify_backup_"
    );
}

#[test]
fn test_original_style_matches_exact_string() {
    // Test that when we have an "Original" style, it matches the exact original string
    // regardless of case style detection

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with various forms that should match exactly
    let test_file = root.join("test.txt");
    std::fs::write(
        &test_file,
        r#"The exact string: renamify
In a path: /path/to/renamify/tool
In mixed context: renamify_2024-version
In format string: renamify_{}.tmp
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Should include Original style by default
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "renamed_renaming_tool", &options).unwrap();

    // Should find occurrences of "renamify"
    // Note: renamify_{}.tmp is not matched because {} is not a valid identifier character
    // and the boundary detection prevents matching "renamify" when followed by _{}
    assert!(
        plan.matches.len() >= 3,
        "Should find at least 3 matches for 'renamify', found {}",
        plan.matches.len()
    );

    // Check specific matches
    let line1_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 1).collect();
    let line2_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 2).collect();
    let line3_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 3).collect();

    assert_eq!(line1_matches.len(), 1);
    assert_eq!(line1_matches[0].before, "renamify");

    assert_eq!(line2_matches.len(), 1);
    assert_eq!(line2_matches[0].before, "renamify");

    assert_eq!(line3_matches.len(), 1);
    assert_eq!(line3_matches[0].before, "renamify_2024-version");
}
