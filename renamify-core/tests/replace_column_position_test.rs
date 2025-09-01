use renamify_core::{create_simple_plan, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_replace_command_column_positions() {
    // This test verifies that column positions in replace command are correct (0-based)
    // Bug: replace command was using 1-based columns causing off-by-one highlighting

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with exact content where column position matters
    let test_file = root.join("test.rb");
    std::fs::write(&test_file, r#"require "active_support/core_ext/hash""#).unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: true,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
    };

    // Test literal replacement (no regex)
    let plan = create_simple_plan(
        "core_ext",
        "ruby_extras",
        vec![root.clone()],
        &options,
        false, // no regex
    )
    .unwrap();

    assert_eq!(plan.matches.len(), 1, "Should find exactly one match");

    let hunk = &plan.matches[0];

    // The critical test: column should be 0-based
    // The string is: require "active_support/core_ext/hash"
    // The position of 'core_ext' is at index 24 (0-based)
    let expected_col = 24;

    assert_eq!(
        hunk.col, expected_col,
        "Column position should be {} (0-based) but got {}",
        expected_col, hunk.col
    );

    // Verify that the column points to the correct position in the line
    let line = hunk.line_before.as_ref().unwrap();
    assert_eq!(
        &line[hunk.col as usize..(hunk.col as usize + 8)],
        "core_ext",
        "Column {} should point to 'core_ext' in the line",
        hunk.col
    );
}

#[test]
fn test_replace_command_regex_column_positions() {
    // Test regex replacement column positions

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file
    let test_file = root.join("test.txt");
    std::fs::write(&test_file, "The number is 42 and another is 99").unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: true,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
    };

    // Test regex replacement
    let plan = create_simple_plan(
        r"\d+",
        "NUM",
        vec![root],
        &options,
        true, // regex mode
    )
    .unwrap();

    assert_eq!(plan.matches.len(), 2, "Should find two number matches");

    // Check first match (42)
    let hunk1 = &plan.matches[0];
    assert_eq!(hunk1.content, "42");
    assert_eq!(
        hunk1.col, 14,
        "First number '42' should be at column 14 (0-based)"
    );

    // Check second match (99)
    let hunk2 = &plan.matches[1];
    assert_eq!(hunk2.content, "99");
    assert_eq!(
        hunk2.col, 32,
        "Second number '99' should be at column 32 (0-based)"
    );

    // Verify columns point to correct positions
    let line = hunk1.line_before.as_ref().unwrap();
    assert_eq!(&line[14..16], "42", "Column 14 should point to '42'");
    assert_eq!(&line[32..34], "99", "Column 32 should point to '99'");
}

#[test]
fn test_replace_highlighting_compatibility() {
    // Test that column positions work correctly with the highlight_line_with_hunks function
    // This ensures diff highlighting will work properly

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("code.rs");
    std::fs::write(&test_file, r#"let old_name = "value";"#).unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: true,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
    };

    let plan = create_simple_plan("old_name", "new_name", vec![root], &options, false).unwrap();

    let hunk = &plan.matches[0];

    // The highlighting function expects to slice the line at hunk.col
    // to get the exact match text
    let line = hunk.line_before.as_ref().unwrap();
    let expected_col = 4; // Position of "old_name" in the line (0-based)

    assert_eq!(
        hunk.col, expected_col,
        "Column should be {} for proper highlighting",
        expected_col
    );

    // This is what the highlighting function does - it should extract "old_name"
    let extracted = &line[hunk.col as usize..(hunk.col as usize + hunk.content.len())];
    assert_eq!(
        extracted, "old_name",
        "Highlighting function should be able to extract the correct text at column position"
    );
}
