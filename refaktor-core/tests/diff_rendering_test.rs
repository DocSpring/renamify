use refaktor_core::{scan_repository, PlanOptions};
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
        r#"fn getPreviewFormatOption() -> PreviewFormatOption { }"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use all default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "preview_format", "foo_bar", &options).unwrap();

    // Render as diff
    let diff_output = refaktor_core::preview::render_plan(
        &plan,
        refaktor_core::preview::PreviewFormat::Diff,
        Some(false), // No color for easier testing
    )
    .unwrap();

    println!("\n=== Diff Merging Test ===");
    println!("{}", diff_output);

    // Count how many times "@@ line 1 @@" appears in the diff
    let line_1_headers = diff_output.matches("@@ line 1 @@").count();

    // The diff should show only ONE entry for line 1, not multiple
    assert_eq!(
        line_1_headers, 1,
        "Diff should merge multiple changes on the same line into a single diff entry, found {} entries",
        line_1_headers
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
