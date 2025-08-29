use renamify_core::{scan_repository_multi, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_exclude_matching_lines_with_comments() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with comments and code
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        r#"// This is a comment with old_name
fn old_name() {
    // Another comment with old_name
    println!("old_name in string");
    let old_name = 42;
}
// old_name in trailing comment
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_matching_lines: Some(r"^\s*//".to_string()), // Exclude comment lines
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "old_name",
        "new_name",
        &options,
    )
    .unwrap();

    // Should find matches on non-comment lines only
    assert_eq!(plan.stats.total_matches, 3); // Function name, string content, variable

    // Verify no matches from comment lines
    for match_hunk in &plan.matches {
        let line_content = match_hunk
            .line_before
            .as_ref()
            .unwrap_or(&match_hunk.content);
        assert!(
            !line_content.trim_start().starts_with("//"),
            "Found match in comment line: {}",
            line_content
        );
    }
}

#[test]
fn test_exclude_matching_lines_with_regex_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with various patterns
    let test_file = temp_dir.path().join("config.txt");
    fs::write(
        &test_file,
        r#"# Config file
old_name = "value"
# DEBUG: old_name settings
DEBUG_old_name = true
PRODUCTION_old_name = false
# old_name configuration end
regular_old_name = "test"
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_matching_lines: Some(r"(^\s*#|DEBUG)".to_string()), // Exclude comments and DEBUG lines
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "old_name",
        "new_name",
        &options,
    )
    .unwrap();

    // Should find matches only on non-comment, non-DEBUG lines
    eprintln!("Found {} matches:", plan.stats.total_matches);
    for m in &plan.matches {
        eprintln!("  - {} at line {}", m.content, m.line);
    }
    assert_eq!(plan.stats.total_matches, 3); // old_name = "value", PRODUCTION_old_name, regular_old_name

    // Verify excluded patterns
    for match_hunk in &plan.matches {
        let line_content = match_hunk
            .line_before
            .as_ref()
            .unwrap_or(&match_hunk.content);
        assert!(
            !line_content.trim_start().starts_with("#"),
            "Found match in comment line: {}",
            line_content
        );
        assert!(
            !line_content.contains("DEBUG"),
            "Found match in DEBUG line: {}",
            line_content
        );
    }
}

#[test]
fn test_exclude_matching_lines_with_todo_comments() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with TODO/FIXME comments
    let test_file = temp_dir.path().join("code.js");
    fs::write(
        &test_file,
        r#"function oldName() {
    // TODO: rename oldName to something better
    console.log("oldName");
    // FIXME: oldName is deprecated
    return oldName + 1;
}
// NOTE: oldName should be refactored
const result = oldName();
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_matching_lines: Some(r"(TODO|FIXME|NOTE)".to_string()), // Exclude TODO/FIXME/NOTE lines
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "oldName",
        "newName",
        &options,
    )
    .unwrap();

    // Should find matches only on actual code lines
    assert_eq!(plan.stats.total_matches, 4); // function name, console.log string, return statement, const call

    // Verify TODO/FIXME/NOTE lines are excluded
    for match_hunk in &plan.matches {
        let line_content = match_hunk
            .line_before
            .as_ref()
            .unwrap_or(&match_hunk.content);
        assert!(!line_content.contains("TODO"), "Found match in TODO line");
        assert!(!line_content.contains("FIXME"), "Found match in FIXME line");
        assert!(!line_content.contains("NOTE"), "Found match in NOTE line");
    }
}

#[test]
fn test_exclude_matching_lines_empty_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(
        &test_file,
        r#"old_name line 1
old_name line 2
old_name line 3
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_matching_lines: None, // No exclusion pattern
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "old_name",
        "new_name",
        &options,
    )
    .unwrap();

    // Should find all matches
    assert_eq!(plan.stats.total_matches, 3);
}

#[test]
fn test_exclude_matching_lines_invalid_regex() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "old_name test").unwrap();

    let options = PlanOptions {
        exclude_matching_lines: Some(r"[invalid(regex".to_string()), // Invalid regex
        ..Default::default()
    };

    // Should handle invalid regex gracefully
    let result = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "old_name",
        "new_name",
        &options,
    );

    // The scan should fail with an error about invalid regex
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("regex") || error_msg.contains("pattern"),
        "Expected regex error, got: {}",
        error_msg
    );
}

#[test]
fn test_exclude_matching_lines_case_sensitive() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with mixed case patterns
    let test_file = temp_dir.path().join("test.txt");
    fs::write(
        &test_file,
        r#"SKIP: old_name uppercase
skip: old_name lowercase
Skip: old_name mixed
normal old_name line
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_matching_lines: Some(r"^SKIP:".to_string()), // Case-sensitive pattern
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "old_name",
        "new_name",
        &options,
    )
    .unwrap();

    // Should exclude only exact case matches
    assert_eq!(plan.stats.total_matches, 3); // lowercase, mixed case, and normal line
}

#[test]
fn test_exclude_matching_lines_multiline_context() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file where excluded lines are near matches
    let test_file = temp_dir.path().join("test.py");
    fs::write(
        &test_file,
        r#"def old_name():
    """
    This is a docstring with old_name
    """
    # Comment with old_name
    print("old_name")
    return old_name
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_matching_lines: Some(r"^\s*#".to_string()), // Exclude Python comments
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "old_name",
        "new_name",
        &options,
    )
    .unwrap();

    // Should find matches except in comment lines
    // Note: docstrings are not comments, so they should be included
    let match_count = plan.stats.total_matches;
    assert!(
        match_count >= 4,
        "Expected at least 4 matches, got {}",
        match_count
    ); // function def, docstring, print, return

    // Verify comment line is excluded
    for match_hunk in &plan.matches {
        if let Some(line) = &match_hunk.line_before {
            assert!(
                !line.trim_start().starts_with("#"),
                "Found match in Python comment: {}",
                line
            );
        }
    }
}
