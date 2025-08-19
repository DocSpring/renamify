use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use renamify_core::{plan_operation, Style};
use std::path::PathBuf;

/// Helper function to create a cross-platform path string for testing
fn path_str(components: &[&str]) -> String {
    let mut path = std::path::PathBuf::new();
    for component in components {
        path.push(component);
    }
    path.to_str().unwrap().to_string()
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Smart search & replace for code and files",
        ));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("renamify"));
}

#[test]
fn test_version_subcommand() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("renamify 0.1.0"));
}

#[test]
fn test_version_subcommand_json() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.args(["version", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{"name":"renamify","version":"0\.1\.0"\}"#).unwrap());
}

#[test]
fn test_plan_command_missing_args() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("plan")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"));
}

#[test]
fn test_plan_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file
        .write_str("fn old_name() { let old_name = 42; }")
        .unwrap();

    let (_result, preview) = plan_operation(
        "old_name",
        "new_name",
        vec![PathBuf::from(".")],   // paths
        vec![],                     // include
        vec![],                     // exclude
        true,                       // respect_gitignore
        0,                          // unrestricted_level
        true,                       // rename_files
        true,                       // rename_dirs
        &[],                        // exclude_styles
        &[],                        // include_styles
        &[],                        // only_styles
        vec![],                     // exclude_match
        None,                       // exclude_matching_lines
        None,                       // plan_out
        Some(&"table".to_string()), // preview_format
        true,                       // dry_run
        true,                       // fixed_table_width - for consistent test output
        false,                      // use_color
        false,                      // no_acronyms
        vec![],                     // include_acronyms
        vec![],                     // exclude_acronyms
        vec![],                     // only_acronyms
        Some(temp_dir.path()),      // working_dir
    )
    .unwrap();

    // Verify the preview contains the table content
    let preview_content = preview.unwrap();
    assert!(
        preview_content.contains("test.rs"),
        "Preview doesn't contain 'test.rs'. Preview content:\n{}",
        preview_content
    );
}

#[test]
fn test_plan_command_with_styles() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file
        .write_str("fn oldName() { let old_name = 42; }")
        .unwrap();

    // Test excluding styles (exclude kebab and pascal, keeping snake and camel)
    let (_result, preview) = plan_operation(
        "old-name",
        "new-name",
        vec![PathBuf::from(".")],                              // paths
        vec![],                                                // include
        vec![],                                                // exclude
        true,                                                  // respect_gitignore
        0,                                                     // unrestricted_level
        true,                                                  // rename_files
        true,                                                  // rename_dirs
        &[Style::Kebab, Style::Pascal, Style::ScreamingSnake], // exclude_styles
        &[],                                                   // include_styles
        &[],                                                   // only_styles
        vec![],                                                // exclude_match
        None,                                                  // exclude_matching_lines
        None,                                                  // plan_out
        Some(&"table".to_string()),                            // preview_format
        true,                                                  // dry_run
        true,                  // fixed_table_width - for consistent test output
        false,                 // use_color
        false,                 // no_acronyms
        vec![],                // include_acronyms
        vec![],                // exclude_acronyms
        vec![],                // only_acronyms
        Some(temp_dir.path()), // working_dir
    )
    .unwrap();

    // Verify the preview contains the table content
    let preview_content = preview.unwrap();
    assert!(
        preview_content.contains("test.rs"),
        "Preview doesn't contain 'test.rs'. Preview content:\n{}",
        preview_content
    );
    assert!(
        preview_content.contains("old_name") || preview_content.contains("oldName"),
        "Preview doesn't contain 'old_name' or 'oldName'. Preview content:\n{}",
        preview_content
    );

    // Test including additional styles
    let (_result2, preview2) = plan_operation(
        "old-name",
        "new-name",
        vec![PathBuf::from(".")],      // paths
        vec![],                        // include
        vec![],                        // exclude
        true,                          // respect_gitignore
        0,                             // unrestricted_level
        true,                          // rename_files
        true,                          // rename_dirs
        &[],                           // exclude_styles
        &[Style::Title, Style::Train], // include_styles
        &[],                           // only_styles
        vec![],                        // exclude_match
        None,                          // exclude_matching_lines
        None,                          // plan_out
        Some(&"table".to_string()),    // preview_format
        true,                          // dry_run
        true,                          // fixed_table_width - for consistent test output
        false,                         // use_color
        false,                         // no_acronyms
        vec![],                        // include_acronyms
        vec![],                        // exclude_acronyms
        vec![],                        // only_acronyms
        Some(temp_dir.path()),         // working_dir
    )
    .unwrap();

    let preview2_content = preview2.unwrap();
    assert!(
        preview2_content.contains("test.rs"),
        "Preview2 doesn't contain 'test.rs'. Preview content:\n{}",
        preview2_content
    );
}

#[test]
fn test_plan_command_with_includes() {
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("src").create_dir_all().unwrap();
    temp_dir.child("tests").create_dir_all().unwrap();
    temp_dir
        .child("src/main.rs")
        .write_str("fn old_name() {}")
        .unwrap();
    temp_dir
        .child("tests/test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--include",
            "src/**/*.rs",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("tests/test.rs").not());
}

#[test]
fn test_plan_command_with_excludes() {
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("src").create_dir_all().unwrap();
    temp_dir.child("tests").create_dir_all().unwrap();
    temp_dir
        .child("src/main.rs")
        .write_str("fn old_name() {}")
        .unwrap();
    temp_dir
        .child("tests/test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--exclude",
            "tests/**",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("tests/test.rs").not());
}

#[test]
fn test_plan_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--output",
            "json",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("\"search\""))
        .stdout(predicate::str::contains("\"replace\""));
}

#[test]
fn test_plan_command_diff_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--preview",
            "diff",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("+++"))
        .stdout(predicate::str::contains("-fn old_name() {}"))
        .stdout(predicate::str::contains("+fn new_name() {}"));
}

#[test]
fn test_plan_command_table_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--preview",
            "table",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("File"))
        .stdout(predicate::str::contains("Kind"))
        .stdout(predicate::str::contains("Matches"))
        .stdout(predicate::str::contains("TOTALS"));
}

#[test]
fn test_dry_run_command() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["dry-run", "old_name", "new_name"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

#[test]
fn test_no_color_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["--no-color", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    // Just verify it doesn't crash with no-color flag
}

#[test]
fn test_rename_files_flag() {
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("old_name.txt").write_str("test").unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("old_name.txt"))
        .stdout(predicate::str::contains("new_name.txt"));
}

#[test]
fn test_rename_dirs_flag() {
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("old_name").create_dir_all().unwrap();
    temp_dir
        .child("old_name/test.rs")
        .write_str("fn test() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("old_name"))
        .stdout(predicate::str::contains("new_name"));
}

#[test]
fn test_no_rename_files_flag() {
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("old_name.txt").write_str("test").unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--no-rename-files",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("old_name.txt").not());
}

#[test]
fn test_apply_command_missing_plan() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply", "nonexistent.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read plan file"));
}

#[test]
fn test_apply_command_with_plan() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with content to replace
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // First create a plan
    let renamify_dir = temp_dir.child(".renamify");
    renamify_dir.create_dir_all().unwrap();

    // Create a minimal valid plan
    let plan_json = r#"{
        "id": "test123",
        "created_at": "2024-01-01T00:00:00Z",
        "search": "old_name",
        "replace": "new_name",
        "styles": [],
        "includes": [],
        "excludes": [],
        "matches": [],
        "paths": [],
        "stats": {
            "files_scanned": 1,
            "total_matches": 0,
            "matches_by_variant": {},
            "files_with_matches": 0
        },
        "version": "1.0.0"
    }"#;

    let plan_path = renamify_dir.child("plan.json");
    plan_path.write_str(plan_json).unwrap();

    // Apply the plan (uses default plan.json)
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));
}

#[test]
fn test_apply_command_deletes_plan_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with content to replace
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Create .renamify directory with plan
    let renamify_dir = temp_dir.child(".renamify");
    renamify_dir.create_dir_all().unwrap();

    // Create a minimal valid plan
    let plan_json = r#"{
        "id": "test456",
        "created_at": "2024-01-01T00:00:00Z",
        "search": "old_name",
        "replace": "new_name",
        "styles": [],
        "includes": [],
        "excludes": [],
        "matches": [],
        "paths": [],
        "stats": {
            "files_scanned": 1,
            "total_matches": 0,
            "matches_by_variant": {},
            "files_with_matches": 0
        },
        "version": "1.0.0"
    }"#;

    let plan_file = renamify_dir.child("plan.json");
    plan_file.write_str(plan_json).unwrap();

    // Verify plan file exists before apply
    assert!(plan_file.exists());

    // Apply the plan
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // Verify plan file was deleted after successful apply
    assert!(!plan_file.exists());
}

#[test]
fn test_apply_command_with_custom_plan_path_keeps_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with content to replace
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Create custom plan file outside .renamify directory
    let custom_plan_file = temp_dir.child("custom_plan.json");
    let plan_json = r#"{
        "id": "test789",
        "created_at": "2024-01-01T00:00:00Z",
        "search": "old_name",
        "replace": "new_name",
        "styles": [],
        "includes": [],
        "excludes": [],
        "matches": [],
        "paths": [],
        "stats": {
            "files_scanned": 1,
            "total_matches": 0,
            "matches_by_variant": {},
            "files_with_matches": 0
        },
        "version": "1.0.0"
    }"#;

    custom_plan_file.write_str(plan_json).unwrap();

    // Create .renamify directory (needed for apply)
    temp_dir.child(".renamify").create_dir_all().unwrap();

    // Verify custom plan file exists before apply
    assert!(custom_plan_file.exists());

    // Apply the plan using custom path as positional argument
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply", "custom_plan.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes applied successfully"));

    // Verify custom plan file still exists (should NOT be deleted)
    assert!(custom_plan_file.exists());
}

#[test]
fn test_undo_command_missing_entry() {
    let temp_dir = TempDir::new().unwrap();

    // Create .renamify directory with empty history
    temp_dir.child(".renamify").create_dir_all().unwrap();
    temp_dir
        .child(".renamify/history.json")
        .write_str("[]")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "History entry 'nonexistent' not found",
        ));
}

#[test]
fn test_redo_command_missing_entry() {
    let temp_dir = TempDir::new().unwrap();

    // Create .renamify directory with empty history
    temp_dir.child(".renamify").create_dir_all().unwrap();
    temp_dir
        .child(".renamify/history.json")
        .write_str("[]")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["redo", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "History entry 'nonexistent' not found",
        ));
}

#[test]
fn test_status_command() {
    let temp_dir = TempDir::new().unwrap();

    // Create .renamify directory
    temp_dir.child(".renamify").create_dir_all().unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        // Status shows "No pending plan" when empty
        .stdout(predicate::str::contains("No pending plan"))
        .stdout(predicate::str::contains("History entries"));
}

#[test]
fn test_history_command_empty() {
    let temp_dir = TempDir::new().unwrap();

    // Create .renamify directory with empty history
    temp_dir.child(".renamify").create_dir_all().unwrap();
    temp_dir
        .child(".renamify/history.json")
        .write_str("[]")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("history")
        .assert()
        .success()
        // Empty history shows message
        .stdout(predicate::str::contains("No history entries found"));
}

#[test]
fn test_history_command_with_entries() {
    let temp_dir = TempDir::new().unwrap();

    // Create .renamify directory with some history
    temp_dir.child(".renamify").create_dir_all().unwrap();

    let history_json = r#"[
        {
            "id": "test1",
            "created_at": "2024-01-01T00:00:00Z",
            "search": "foo",
            "replace": "bar",
            "styles": [],
            "includes": [],
            "excludes": [],
            "affected_files": {},
            "renames": [],
            "backups_path": ".renamify/backups/test1",
            "revert_of": null,
            "redo_of": null
        },
        {
            "id": "test2",
            "created_at": "2024-01-02T00:00:00Z",
            "search": "baz",
            "replace": "qux",
            "styles": [],
            "includes": [],
            "excludes": [],
            "affected_files": {},
            "renames": [],
            "backups_path": ".renamify/backups/test2",
            "revert_of": null,
            "redo_of": null
        }
    ]"#;

    temp_dir
        .child(".renamify/history.json")
        .write_str(history_json)
        .unwrap();

    // Test without limit - should show both entries
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("history")
        .assert()
        .success()
        .stdout(predicate::str::contains("test1"))
        .stdout(predicate::str::contains("test2"))
        .stdout(predicate::str::contains("foo -> bar"))
        .stdout(predicate::str::contains("baz -> qux"));

    // Test with limit - should show only one entry
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["history", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test2"))
        .stdout(predicate::str::contains("test1").not());
}

#[test]
fn test_invalid_style_arg() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.args(["plan", "old", "new", "--styles", "invalid"])
        .assert()
        .failure();
}

#[test]
fn test_invalid_preview() {
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.args(["plan", "old", "new", "--preview", "invalid"])
        .assert()
        .failure();
}

#[test]
fn test_exit_codes() {
    // Test normal success
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success()
        .code(0);

    // Test invalid arguments (should exit with 2)
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("plan").assert().failure();
}

#[test]
fn test_init_command_default() {
    // Test default behavior: adds to .gitignore
    let temp_dir = TempDir::new().unwrap();

    // Run init command
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicates::str::contains("Added .renamify/ to .gitignore"));

    // Check .gitignore was created with correct content
    temp_dir
        .child(".gitignore")
        .assert(predicates::str::contains(".renamify/"));
    temp_dir
        .child(".gitignore")
        .assert(predicates::str::contains("# Renamify workspace"));
}

#[test]
fn test_init_command_idempotent() {
    // Test that running init twice doesn't duplicate the entry
    let temp_dir = TempDir::new().unwrap();

    // Run init command first time
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Run init command second time
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicates::str::contains("already ignored"));

    // Check .gitignore only has one entry
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    let count = content.matches(".renamify/").count();
    assert_eq!(count, 1, "Should only have one .renamify/ entry");
}

#[test]
fn test_init_command_existing_gitignore() {
    // Test adding to existing .gitignore
    let temp_dir = TempDir::new().unwrap();

    // Create existing .gitignore
    temp_dir
        .child(".gitignore")
        .write_str("target/\n*.tmp\n")
        .unwrap();

    // Run init command
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Check .gitignore preserved existing content and added new
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("*.tmp"));
    assert!(content.contains(".renamify/"));
    assert!(content.contains("# Renamify workspace"));
}

#[test]
fn test_init_command_local_flag() {
    // Test --local flag: adds to .git/info/exclude
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Run init command with --local
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--local"])
        .assert()
        .success()
        .stderr(predicates::str::contains(path_str(&[
            ".git", "info", "exclude",
        ])));

    // Check .git/info/exclude was created with correct content
    let exclude_path = temp_dir.path().join(".git/info/exclude");
    assert!(exclude_path.exists());
    let content = std::fs::read_to_string(exclude_path).unwrap();
    assert!(content.contains(".renamify/"));
}

#[test]
fn test_init_command_not_in_git_repo() {
    // Test --local flag when not in a git repo (should fail)
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--local"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Not in a git repository"));
}

#[test]
fn test_init_command_with_variations() {
    // Test that it detects existing patterns with variations
    let temp_dir = TempDir::new().unwrap();

    // Create .gitignore with variation
    temp_dir
        .child(".gitignore")
        .write_str("/.renamify\n")
        .unwrap();

    // Run init command - should detect existing pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicates::str::contains("already ignored"));
}

#[test]
fn test_init_command_appends_with_newline() {
    // Test that it properly handles files without trailing newlines
    let temp_dir = TempDir::new().unwrap();

    // Create .gitignore without trailing newline
    temp_dir.child(".gitignore").write_str("target/").unwrap();

    // Run init command
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Check proper formatting
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    assert!(content.contains("target/\n")); // Should have newline after existing content
    assert!(content.contains("\n# Renamify workspace\n")); // Should have blank line before comment
    assert!(content.ends_with(".renamify/\n")); // Should end with newline
}

#[test]
fn test_init_check_mode() {
    // Test --check flag functionality
    let temp_dir = TempDir::new().unwrap();

    // When .renamify is not ignored, should exit with code 1
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--check"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(".renamify is NOT ignored"));

    // Add .renamify to .gitignore
    temp_dir
        .child(".gitignore")
        .write_str(".renamify/\n")
        .unwrap();

    // Now --check should succeed
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--check"])
        .assert()
        .success()
        .stderr(predicates::str::contains(".renamify is properly ignored"));
}

#[test]
fn test_auto_init_flag() {
    // Test --auto-init=repo flag
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with --auto-init=repo
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "--auto-init=repo",
            "plan",
            "old_name",
            "new_name",
            "--dry-run",
        ])
        .assert()
        .success();

    // Check .gitignore was created
    temp_dir
        .child(".gitignore")
        .assert(predicates::str::contains(".renamify/"));
}

#[test]
fn test_no_auto_init_flag() {
    // Test --no-auto-init flag prevents initialization
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with --no-auto-init
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "--no-auto-init",
            "plan",
            "old_name",
            "new_name",
            "--dry-run",
        ])
        .assert()
        .success();

    // Check .gitignore was NOT created
    assert!(!temp_dir.path().join(".gitignore").exists());
}

#[test]
fn test_yes_flag_auto_init() {
    // Test -y flag chooses repo mode automatically
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with -y flag
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["-y", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();

    // Check .gitignore was created (default repo mode)
    temp_dir
        .child(".gitignore")
        .assert(predicates::str::contains(".renamify/"));
}

#[test]
fn test_auto_init_local_mode() {
    // Test --auto-init=local flag
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Run plan with --auto-init=local
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "--auto-init=local",
            "plan",
            "old_name",
            "new_name",
            "--dry-run",
        ])
        .assert()
        .success();

    // Check .git/info/exclude was created
    let exclude_path = temp_dir.path().join(".git/info/exclude");
    assert!(exclude_path.exists());
    let content = std::fs::read_to_string(exclude_path).unwrap();
    assert!(content.contains(".renamify/"));
}

#[test]
fn test_auto_init_idempotent() {
    // Test that auto-init doesn't duplicate entries
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // First run with auto-init
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "--auto-init=repo",
            "plan",
            "old_name",
            "new_name",
            "--dry-run",
        ])
        .assert()
        .success();

    // Second run - should not duplicate
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "--auto-init=repo",
            "plan",
            "old_name",
            "new_name",
            "--dry-run",
        ])
        .assert()
        .success();

    // Check only one entry exists
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    let count = content.matches(".renamify/").count();
    assert_eq!(count, 1, "Should only have one .renamify/ entry");
}

#[test]
fn test_rename_command_basic() {
    // E2E test for the rename command that creates a temp file and verifies the rename works
    let temp_dir = TempDir::new().unwrap();

    // Create a test file with content containing old_name in various forms
    let test_content = "This is a test file with old_name in it.\nHere's another old_name reference.\nAnd a test_old_name variable too.";
    temp_dir.child("test.txt").write_str(test_content).unwrap();

    // Run rename command with -y to auto-approve, including txt files explicitly
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y", "--include=*.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"))
        .stdout(predicate::str::contains("replacements"));

    // Verify the file content was changed
    let updated_content = std::fs::read_to_string(temp_dir.path().join("test.txt")).unwrap();
    assert!(updated_content.contains("new_name"));

    // Check that all old_name occurrences are replaced, including compounds
    let old_name_count = updated_content.matches("old_name").count();
    assert_eq!(
        old_name_count, 0,
        "All old_name occurrences should be replaced"
    );

    // Should have replaced all occurrences: 2 standalone + 1 in test_old_name = 3 total
    assert_eq!(updated_content.matches("new_name").count(), 3);
    assert!(updated_content.contains("test_new_name")); // Compound should be updated
}

#[test]
fn test_rename_command_with_preview() {
    // Test rename command with preview option
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() { let old_name = 42; }")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "--preview", "table", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("File"))
        .stdout(predicate::str::contains("Kind"))
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("Applied"));

    // Verify the file was actually modified
    let content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(content.contains("fn new_name()"));
    assert!(content.contains("let new_name = 42;"));
}

#[test]
fn test_rename_command_with_file_rename() {
    // Test rename command that renames both content and files
    let temp_dir = TempDir::new().unwrap();

    // Create files with matching names
    temp_dir
        .child("old_name.txt")
        .write_str("content with old_name")
        .unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn test_old_name() {}")
        .unwrap();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"))
        .stdout(predicate::str::contains("Renamed"));

    // Verify file was renamed
    assert!(!temp_dir.path().join("old_name.txt").exists());
    assert!(temp_dir.path().join("new_name.txt").exists());

    // Verify content was updated
    let renamed_file_content =
        std::fs::read_to_string(temp_dir.path().join("new_name.txt")).unwrap();
    assert!(renamed_file_content.contains("content with new_name"));

    let test_file_content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(test_file_content.contains("fn test_new_name()")); // Compound should be updated
}

#[test]
fn test_rename_command_requires_confirmation() {
    // Test that rename command requires confirmation when not auto-approved
    let temp_dir = TempDir::new().unwrap();
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Without -y flag and in non-interactive mode, should fail
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot prompt for confirmation in non-interactive mode",
        ));
}

#[test]
fn test_rename_command_large_size_guard() {
    // Test that rename command respects the --large flag for size guards
    let temp_dir = TempDir::new().unwrap();

    // Create many files to trigger size guard (this is a simplified test)
    for i in 0..10 {
        temp_dir
            .child(format!("test_{i}.rs"))
            .write_str("fn old_name() {}")
            .unwrap();
    }

    // Should succeed with normal amount of files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y"])
        .assert()
        .success();
}

#[test]
fn test_affected_files_not_empty() {
    // Test that affected_files is populated in history
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Apply a rename
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y"])
        .assert()
        .success();

    // Check history has affected_files
    let history_file = temp_dir.path().join(".renamify/history.json");
    let history_content = std::fs::read_to_string(&history_file).unwrap();
    let history: Vec<serde_json::Value> = serde_json::from_str(&history_content).unwrap();
    assert!(!history.is_empty(), "History should not be empty");

    let entry = &history[0];
    let affected_files = entry["affected_files"].as_object().unwrap();
    assert!(
        !affected_files.is_empty(),
        "affected_files should NOT be empty! Files that were modified should be tracked!"
    );

    // Verify the file path is in affected_files
    let has_test_file = affected_files.keys().any(|k| k.ends_with("test.rs"));
    assert!(has_test_file, "test.rs should be in affected_files");
}

#[test]
fn test_undo_after_rename() {
    // Integration test for undo after a rename operation
    let temp_dir = TempDir::new().unwrap();

    // Create test files with content to rename
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() { let old_name = 42; }")
        .unwrap();
    temp_dir
        .child("old_name.txt")
        .write_str("This is the old_name file")
        .unwrap();

    // First perform a rename operation
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // Verify files were changed
    let test_rs_content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(test_rs_content.contains("new_name"));
    assert!(!test_rs_content.contains("old_name"));
    assert!(temp_dir.path().join("new_name.txt").exists());
    assert!(!temp_dir.path().join("old_name.txt").exists());

    // Get the history to find the ID
    let history_file = temp_dir.path().join(".renamify/history.json");
    let history_content = std::fs::read_to_string(&history_file).unwrap();
    let history: Vec<serde_json::Value> = serde_json::from_str(&history_content).unwrap();
    let last_entry = &history[history.len() - 1];
    let id = last_entry["id"].as_str().unwrap();

    // Now undo the operation
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["undo", id])
        .assert()
        .success()
        .get_output()
        .clone();

    eprintln!(
        "DEBUG test_undo_after_rename undo stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    eprintln!(
        "DEBUG test_undo_after_rename undo stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files are back to original state
    let test_rs_content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(test_rs_content.contains("old_name"));
    assert!(!test_rs_content.contains("new_name"));
    assert!(temp_dir.path().join("old_name.txt").exists());
    assert!(!temp_dir.path().join("new_name.txt").exists());

    // Verify content is restored
    let txt_content = std::fs::read_to_string(temp_dir.path().join("old_name.txt")).unwrap();
    assert_eq!(txt_content, "This is the old_name file");
}

#[test]
fn test_undo_latest() {
    // Test undo with "latest" keyword
    let temp_dir = TempDir::new().unwrap();

    temp_dir.child("test.rs").write_str("fn foo() {}").unwrap();

    // Apply a rename
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "foo", "bar", "-y"])
        .assert()
        .success();

    // Verify the rename was applied correctly
    let content_after_rename = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    eprintln!(
        "DEBUG test_undo_latest: content after rename = {:?}",
        content_after_rename
    );
    assert!(
        content_after_rename.contains("bar"),
        "Expected 'bar' in content after rename: {:?}",
        content_after_rename
    );
    assert!(
        !content_after_rename.contains("foo"),
        "Did not expect 'foo' in content after rename: {:?}",
        content_after_rename
    );
    assert!(
        !content_after_rename.ends_with('\n'),
        "Expected no trailing newline after rename: {:?}",
        content_after_rename
    );

    // Undo using "latest"
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["undo", "latest"])
        .assert()
        .success()
        .get_output()
        .clone();

    eprintln!(
        "DEBUG undo stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    eprintln!(
        "DEBUG undo stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify revert
    let content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    eprintln!("DEBUG test_undo_latest: actual content = {:?}", content);
    assert!(
        content.contains("foo"),
        "Expected 'foo' in content: {:?}",
        content
    );
    assert!(
        !content.contains("bar"),
        "Did not expect 'bar' in content: {:?}",
        content
    );
}

#[test]
fn test_undo_already_undone() {
    // Test that trying to undo twice fails
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn original() {}")
        .unwrap();

    // Apply a rename
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "original", "modified", "-y"])
        .assert()
        .success();

    // Get the ID
    let history_file = temp_dir.path().join(".renamify/history.json");
    let history_content = std::fs::read_to_string(&history_file).unwrap();
    let history: Vec<serde_json::Value> = serde_json::from_str(&history_content).unwrap();
    let id = history[0]["id"].as_str().unwrap();

    // First undo succeeds
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", id])
        .assert()
        .success();

    // Second undo should fail
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already been reverted"));
}

#[test]
fn test_redo_after_undo() {
    // Test redo functionality
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn alpha() {}")
        .unwrap();

    // Apply a rename
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "alpha", "beta", "-y"])
        .assert()
        .success();

    // Get the ID
    let history_file = temp_dir.path().join(".renamify/history.json");
    let history_content = std::fs::read_to_string(&history_file).unwrap();
    let history: Vec<serde_json::Value> = serde_json::from_str(&history_content).unwrap();
    let id = history[0]["id"].as_str().unwrap();

    // Undo the operation
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", id])
        .assert()
        .success();

    // Verify undo worked
    let content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(content.contains("alpha"));

    // Now redo the operation
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["redo", id])
        .assert()
        .success()
        .stderr(predicate::str::contains("Successfully redid"));

    // Verify redo worked
    let content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(content.contains("beta"));
    assert!(!content.contains("alpha"));
}

#[test]
fn test_undo_with_multiple_files() {
    // Test undo with multiple files and renames
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    temp_dir.child("src").create_dir_all().unwrap();
    temp_dir
        .child("src/main.rs")
        .write_str("fn old_func() { old_func(); }")
        .unwrap();
    temp_dir
        .child("src/lib.rs")
        .write_str("pub fn old_func() {}")
        .unwrap();
    temp_dir
        .child("old_func.txt")
        .write_str("Documentation for old_func")
        .unwrap();

    // Apply a rename across all files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_func", "new_func", "-y"])
        .assert()
        .success();

    // Verify changes
    assert!(temp_dir.path().join("new_func.txt").exists());
    assert!(!temp_dir.path().join("old_func.txt").exists());
    let main_content = std::fs::read_to_string(temp_dir.path().join("src/main.rs")).unwrap();
    assert!(main_content.contains("new_func"));

    // Undo the operation
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", "latest"])
        .assert()
        .success();

    // Verify all files are restored
    assert!(temp_dir.path().join("old_func.txt").exists());
    assert!(!temp_dir.path().join("new_func.txt").exists());

    let main_content = std::fs::read_to_string(temp_dir.path().join("src/main.rs")).unwrap();
    assert!(main_content.contains("old_func"));
    assert!(!main_content.contains("new_func"));

    let lib_content = std::fs::read_to_string(temp_dir.path().join("src/lib.rs")).unwrap();
    assert!(lib_content.contains("old_func"));

    let txt_content = std::fs::read_to_string(temp_dir.path().join("old_func.txt")).unwrap();
    assert_eq!(txt_content, "Documentation for old_func");
}
