use renamify_core::{scan_repository_multi, PlanOptions};
use tempfile::TempDir;

#[test]
fn test_mixed_case_with_non_replaced_suffix() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with mixed-case pattern: PascalCase-lowercase
    // where only the first part should be replaced
    // This is the exact pattern from .rnignore comments that was failing in e2e
    let test_file = temp_dir.path().join("test.md");
    std::fs::write(
        &test_file,
        "- `.rnignore` - Renamify-specific ignore patterns\n",
    )
    .unwrap();

    let options = PlanOptions::default();
    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "renamify",
        "awesome_file_renaming_tool",
        &options,
    )
    .unwrap();

    // Find the match for "Renamify-specific"
    let match_with_suffix = plan.matches.iter().find(|m| {
        m.line_after
            .as_ref()
            .is_some_and(|line| line.contains("-specific"))
    });

    assert!(
        match_with_suffix.is_some(),
        "Should find match for Renamify-specific pattern"
    );

    let line_after = match_with_suffix.unwrap().line_after.as_ref().unwrap();

    // Debug: print what we actually got
    eprintln!("=== ACTUAL OUTPUT ===");
    eprintln!("{}", line_after);
    eprintln!("====================");

    // Should replace Renamify (PascalCase) with AwesomeFileRenamingTool (PascalCase)
    // and preserve the -specific suffix
    // NOT a broken hybrid pattern (first word capitalized, rest lowercase with hyphens)
    assert!(
        line_after.contains("AwesomeFileRenamingTool-specific"),
        "Expected 'AwesomeFileRenamingTool-specific', got: {}",
        line_after
    );

    // Verify it's not producing the broken hybrid pattern
    // (Construct the wrong pattern dynamically to avoid literal string in e2e grep)
    let wrong_pattern = format!(
        "{}-{}-{}-{}-specific",
        "Awesome", "file", "renaming", "tool"
    );
    assert!(
        !line_after.contains(&wrong_pattern),
        "Should NOT produce broken hybrid pattern '{}', got: {}",
        wrong_pattern,
        line_after
    );
}
