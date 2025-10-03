use renamify_core::{scan_repository, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_gitignore_with_leading_dot_gets_renamed() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create a .gitignore with a pattern that has a leading dot
    fs::write(
        repo_path.join(".gitignore"),
        ".renamify/\nrenamify-core/bindings/\n",
    )
    .unwrap();

    // Scan to rename renamify -> testword
    let options = PlanOptions {
        respect_gitignore: false, // We want to scan the .gitignore file itself
        unrestricted_level: 2,    // Ignore all ignore files
        ..Default::default()
    };

    let plan = scan_repository(repo_path, "renamify", "testword", &options).unwrap();

    // Check that .gitignore was scanned
    let gitignore_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with(".gitignore"))
        .collect();

    assert!(
        !gitignore_matches.is_empty(),
        "Should find matches in .gitignore"
    );

    // Check that we found "renamify" in ".renamify/" by checking line_after
    let leading_dot_match = plan.matches.iter().find(|m| {
        m.file.ends_with(".gitignore")
            && m.line_before.as_ref().map_or(false, |l| l.contains(".renamify"))
            && m.line_after.as_ref().map_or(false, |l| l.contains(".testword"))
    });

    assert!(
        leading_dot_match.is_some(),
        "Should match '.renamify/' pattern in .gitignore and replace with '.testword/'. Matches: {:#?}",
        gitignore_matches
    );
}

#[test]
fn test_pattern_with_leading_dot_and_trailing_slash() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create a file with the pattern
    fs::write(
        repo_path.join("test.txt"),
        "Ignoring .renamify/ directory\n",
    )
    .unwrap();

    let options = PlanOptions {
        unrestricted_level: 2,
        ..Default::default()
    };

    let plan = scan_repository(repo_path, "renamify", "testword", &options).unwrap();

    // Should find "renamify" in ".renamify/"
    let matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("test.txt"))
        .collect();

    assert_eq!(matches.len(), 1, "Should find one match");
    assert!(
        matches[0].content.contains("renamify"),
        "Content should contain 'renamify'"
    );
    assert!(
        matches[0].replace.contains("testword"),
        "Replacement should contain 'testword'"
    );
}
