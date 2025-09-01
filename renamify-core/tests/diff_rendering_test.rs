use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_diff_merging_same_line() {
    // This test verifies that the diff view properly merges multiple changes on the same line
    // Instead of showing two separate diffs for the same line

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with multiple replacements on the same line
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r"fn getPreviewFormatOption() -> PreviewFormatOption { }",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
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

    let plan = scan_repository(&root, "preview_format", "foo_bar", &options).unwrap();

    // Render as diff
    let diff_output = renamify_core::preview::render_plan(
        &plan,
        renamify_core::preview::Preview::Diff,
        Some(false), // No color for easier testing
    );

    println!("\n=== Diff Merging Test ===");
    println!("{diff_output}");

    // Count how many times "@@ line 1 @@" appears in the diff
    let line_1_headers = diff_output.matches("@@ line 1 @@").count();

    // The diff should show only ONE entry for line 1, not multiple
    assert_eq!(
        line_1_headers, 1,
        "Diff should merge multiple changes on the same line into a single diff entry, found {line_1_headers} entries"
    );

    // The diff should show the cumulative effect of both replacements
    assert!(
        diff_output.contains("-fn getPreviewFormatOption() -> PreviewFormatOption { }"),
        "Diff should show the original line"
    );

    // This assertion will fail until the bug is fixed
    // It should show the line with BOTH replacements applied
    assert!(
        diff_output.contains("+fn getFooBarOption() -> FooBarOption { }"),
        "Diff should show the line with ALL replacements applied"
    );
}

#[test]
fn test_diff_highlighting() {
    // This test verifies that diff highlighting positions are correct
    // Bug: highlighting is off by one character to the right
    // Example: "core_ext" -> "ruby_extras" was highlighting "ore_ext/" -> "uby_extras/"

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with the exact string that shows the bug
    let test_file = root.join("test.rb");
    std::fs::write(&test_file, r#"require "active_support/core_ext/hash""#).unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
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

    let plan = scan_repository(&root, "core_ext", "ruby_extras", &options).unwrap();

    // Verify we found the match
    assert_eq!(plan.matches.len(), 1, "Should find exactly one match");

    let hunk = &plan.matches[0];
    assert_eq!(hunk.content, "core_ext", "Should match 'core_ext'");
    assert_eq!(
        hunk.replace, "ruby_extras",
        "Should replace with 'ruby_extras'"
    );

    // The critical test: verify the column position is correct
    // The string is: require "active_support/core_ext/hash"
    // The position of 'core_ext' should be at index 24 (0-based)
    let expected_line = r#"require "active_support/core_ext/hash""#;
    let expected_col = expected_line
        .find("core_ext")
        .expect("Should find core_ext in line");

    assert_eq!(
        hunk.col as usize, expected_col,
        "Column position should be {} but got {}. This would cause highlighting to start at '{}' instead of 'core_ext'",
        expected_col,
        hunk.col,
        &expected_line[(hunk.col as usize)..(hunk.col as usize + 8)]
    );

    // Also verify that line_before contains the full line
    assert_eq!(
        hunk.line_before.as_ref().unwrap(),
        expected_line,
        "line_before should contain the full line"
    );

    // Verify that when we slice the line at hunk.col, we get "core_ext"
    let line = hunk.line_before.as_ref().unwrap();
    let sliced = &line[hunk.col as usize..];
    assert!(
        sliced.starts_with("core_ext"),
        "Slicing line at column {} should yield 'core_ext...' but got '{}'",
        hunk.col,
        &sliced[..20.min(sliced.len())]
    );
}
