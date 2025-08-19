use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_exclude_matching_lines_cli_flag() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with comments
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        r#"// Comment with old_tool
fn old_tool() {
    println!("old_tool"); // inline comment with old_tool
}
let x = old_tool();
"#,
    )
    .unwrap();

    // Run plan with --exclude-matching-lines to filter out comment lines
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path()).args(&[
        "plan",
        "old_tool",
        "new_tool",
        "--exclude-matching-lines",
        r"^\s*//", // Exclude lines starting with //
        "--preview",
        "summary",
    ]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should report matches but not from comment lines
    assert!(stdout.contains("Matches:"));
    assert!(stdout.contains("test.rs"));

    // The count should be 4:
    // 1. function name (line 15)
    // 2. println string (line 16)
    // 3. inline comment on line 16 (not excluded because line doesn't START with //)
    // 4. function call (line 18)
    // Line 14 is excluded (starts with //)
    assert!(
        stdout.contains("Matches: 4") || stdout.contains("4 matches"),
        "Expected 4 matches (inline comment is not excluded), got output: {}",
        stdout
    );
}

#[test]
fn test_exclude_matching_lines_with_dry_run() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file
    let test_file = temp_dir.path().join("config.yml");
    fs::write(
        &test_file,
        r#"# Configuration
name: old_service
# TODO: rename old_service
debug_old_service: true
production:
  old_service: enabled
"#,
    )
    .unwrap();

    // Run dry-run with exclusion pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path()).args(&[
        "dry-run",
        "old_service",
        "new_service",
        "--exclude-matching-lines",
        r"(^\s*#|TODO)", // Exclude comments and TODO lines
        "--preview",
        "diff",
    ]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show diff but not include comment/TODO lines
    assert!(stdout.contains("old_service") || stdout.contains("new_service"));
    assert!(
        !stdout.contains("# TODO:"),
        "Should not match TODO comment line"
    );
}

#[test]
fn test_exclude_matching_lines_in_rename_command() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize .renamify directory
    fs::create_dir_all(temp_dir.path().join(".renamify")).unwrap();

    // Create a test file
    let test_file = temp_dir.path().join("script.sh");
    fs::write(
        &test_file,
        r#"#!/bin/bash
# Script with old_command
old_command --help
# DEBUG: old_command info
echo "Running old_command"
old_command --version
"#,
    )
    .unwrap();

    // Create a plan first with exclusions
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path()).args(&[
        "plan",
        "old_command",
        "new_command",
        "--exclude-matching-lines",
        r"^\s*#", // Exclude shell comments
    ]);

    cmd.assert().success();

    // Verify the plan was created
    assert!(temp_dir.path().join(".renamify/plan.json").exists());

    // Apply the plan (defaults to .renamify/plan.json)
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path()).args(&["apply"]);

    cmd.assert().success();

    // Check that comments were not modified
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(
        content.contains("# Script with old_command"),
        "Comments should not be modified"
    );
    assert!(
        content.contains("# DEBUG: old_command info"),
        "Debug comments should not be modified"
    );
    assert!(
        content.contains("new_command --help"),
        "Non-comment lines should be modified"
    );
    assert!(
        content.contains("Running new_command"),
        "String content should be modified"
    );
    assert!(
        content.contains("new_command --version"),
        "Non-comment lines should be modified"
    );
}

#[test]
fn test_exclude_matching_lines_invalid_regex_error() {
    let temp_dir = TempDir::new().unwrap();

    // Create a dummy file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Run with invalid regex pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path()).args(&[
        "plan",
        "old",
        "new",
        "--exclude-matching-lines",
        r"[invalid(regex", // Invalid regex
    ]);

    // Should fail with error message about regex
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("regex").or(predicate::str::contains("pattern")));
}

#[test]
fn test_exclude_matching_lines_complex_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with various patterns to exclude
    let test_file = temp_dir.path().join("code.py");
    fs::write(
        &test_file,
        r#"import old_module
# pylint: disable=old_module
# type: ignore[old_module]
"""old_module docstring"""
from old_module import something
# noqa: old_module
old_module.function()
"#,
    )
    .unwrap();

    // Run with complex exclusion pattern for Python linter comments
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path()).args(&[
        "plan",
        "old_module",
        "new_module",
        "--exclude-matching-lines",
        r"#.*(pylint|type:|noqa)", // Exclude linter directive comments
        "--preview",
        "summary",
    ]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find matches but not in linter comments
    assert!(stdout.contains("Matches:"));
    // Should match: import, docstring, from import, and function call (4 matches)
    // Should NOT match: the three linter comment lines
}

#[test]
fn test_exclude_matching_lines_help_text() {
    // Verify the help text includes the new option
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.args(&["plan", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--exclude-matching-lines"))
        .stdout(predicate::str::contains("regex"));
}
