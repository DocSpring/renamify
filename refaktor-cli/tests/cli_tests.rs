use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI for Refaktor"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("refaktor"));
}

#[test]
fn test_plan_command_missing_args() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.arg("plan")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"));
}

#[test]
fn test_plan_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() { let old_name = 42; }").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

#[test]
fn test_plan_command_with_styles() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn oldName() { let old_name = 42; }").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old-name", "new-name", "--styles", "snake,camel", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("old_name"))
        .stdout(predicate::str::contains("oldName"));
}

#[test]
fn test_plan_command_with_includes() {
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("src").create_dir_all().unwrap();
    temp_dir.child("tests").create_dir_all().unwrap();
    temp_dir.child("src/main.rs").write_str("fn old_name() {}").unwrap();
    temp_dir.child("tests/test.rs").write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--include", "src/**/*.rs", "--dry-run"])
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
    temp_dir.child("src/main.rs").write_str("fn old_name() {}").unwrap();
    temp_dir.child("tests/test.rs").write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--exclude", "tests/**", "--dry-run"])
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
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--preview-format", "json", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("\"old\""))
        .stdout(predicate::str::contains("\"new\""));
}

#[test]
fn test_plan_command_diff_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--preview-format", "diff", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("+++"))
        .stdout(predicate::str::contains("-fn old_name()"))
        .stdout(predicate::str::contains("+fn new_name()"));
}

#[test]
fn test_plan_command_table_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.child("test.rs");
    test_file.write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--preview-format", "table", "--dry-run"])
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
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
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
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
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
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
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
    temp_dir.child("old_name/test.rs").write_str("fn test() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
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
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--no-rename-files", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("old_name.txt").not());
}

#[test]
fn test_apply_command_missing_plan() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply", "--plan", "nonexistent.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read plan"));
}

#[test]
fn test_apply_command_with_plan() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test file with content to replace
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    // First create a plan
    let refaktor_dir = temp_dir.child(".refaktor");
    refaktor_dir.create_dir_all().unwrap();
    
    // Create a minimal valid plan
    let plan_json = r#"{
        "id": "test123",
        "created_at": "2024-01-01T00:00:00Z",
        "old": "old_name",
        "new": "new_name",
        "styles": [],
        "includes": [],
        "excludes": [],
        "matches": [],
        "renames": [],
        "stats": {
            "files_scanned": 1,
            "total_matches": 0,
            "matches_by_variant": {},
            "files_with_matches": 0
        },
        "version": "1.0.0"
    }"#;
    
    refaktor_dir.child("plan.json").write_str(plan_json).unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Applying plan test123"))
        .stderr(predicate::str::contains("Plan applied successfully!"));
}

#[test]
fn test_undo_command_missing_entry() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create .refaktor directory with empty history
    temp_dir.child(".refaktor").create_dir_all().unwrap();
    temp_dir.child(".refaktor/history.json").write_str("[]").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("History entry 'nonexistent' not found"));
}

#[test]
fn test_redo_command_missing_entry() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create .refaktor directory with empty history
    temp_dir.child(".refaktor").create_dir_all().unwrap();
    temp_dir.child(".refaktor/history.json").write_str("[]").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["redo", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("History entry 'nonexistent' not found"));
}

#[test]
fn test_status_command() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create .refaktor directory
    temp_dir.child(".refaktor").create_dir_all().unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        // Status shows "No plans applied yet" when empty
        .stdout(predicate::str::contains("No plans applied yet"))
        .stdout(predicate::str::contains("Working tree"));
}

#[test]
fn test_history_command_empty() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create .refaktor directory with empty history
    temp_dir.child(".refaktor").create_dir_all().unwrap();
    temp_dir.child(".refaktor/history.json").write_str("[]").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("history")
        .assert()
        .success()
        // Empty history shows an empty table
        .stdout(predicate::str::contains("ID"))
        .stdout(predicate::str::contains("Date"))
        .stdout(predicate::str::contains("Rename"));
}

#[test]
fn test_history_command_with_entries() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create .refaktor directory with some history
    temp_dir.child(".refaktor").create_dir_all().unwrap();
    
    let history_json = r#"[
        {
            "id": "test1",
            "created_at": "2024-01-01T00:00:00Z",
            "old": "foo",
            "new": "bar",
            "styles": [],
            "includes": [],
            "excludes": [],
            "affected_files": {},
            "renames": [],
            "backups_path": ".refaktor/backups/test1",
            "revert_of": null,
            "redo_of": null
        },
        {
            "id": "test2",
            "created_at": "2024-01-02T00:00:00Z",
            "old": "baz",
            "new": "qux",
            "styles": [],
            "includes": [],
            "excludes": [],
            "affected_files": {},
            "renames": [],
            "backups_path": ".refaktor/backups/test2",
            "revert_of": null,
            "redo_of": null
        }
    ]"#;
    
    temp_dir.child(".refaktor/history.json").write_str(history_json).unwrap();
    
    // Test without limit - should show both entries
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("history")
        .assert()
        .success()
        .stdout(predicate::str::contains("test1"))
        .stdout(predicate::str::contains("test2"))
        .stdout(predicate::str::contains("foo → bar"))
        .stdout(predicate::str::contains("baz → qux"));
    
    // Test with limit - should show only one entry
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["history", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test2"))
        .stdout(predicate::str::contains("test1").not());
}

#[test]
fn test_invalid_style_arg() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.args(["plan", "old", "new", "--styles", "invalid"])
        .assert()
        .failure();
}

#[test]
fn test_invalid_preview_format() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.args(["plan", "old", "new", "--preview-format", "invalid"])
        .assert()
        .failure();
}

#[test]
fn test_exit_codes() {
    // Test normal success
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success()
        .code(0);
    
    // Test invalid arguments (should exit with 2)
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.arg("plan")
        .assert()
        .failure();
}

#[test]
fn test_init_command_default() {
    // Test default behavior: adds to .gitignore
    let temp_dir = TempDir::new().unwrap();
    
    // Run init command
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicates::str::contains("Added .refaktor/ to .gitignore"));
    
    // Check .gitignore was created with correct content
    temp_dir.child(".gitignore").assert(predicates::str::contains(".refaktor/"));
    temp_dir.child(".gitignore").assert(predicates::str::contains("# Refaktor workspace"));
}

#[test]
fn test_init_command_idempotent() {
    // Test that running init twice doesn't duplicate the entry
    let temp_dir = TempDir::new().unwrap();
    
    // Run init command first time
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();
    
    // Run init command second time
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicates::str::contains("already ignored"));
    
    // Check .gitignore only has one entry
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    let count = content.matches(".refaktor/").count();
    assert_eq!(count, 1, "Should only have one .refaktor/ entry");
}

#[test]
fn test_init_command_existing_gitignore() {
    // Test adding to existing .gitignore
    let temp_dir = TempDir::new().unwrap();
    
    // Create existing .gitignore
    temp_dir.child(".gitignore").write_str("target/\n*.tmp\n").unwrap();
    
    // Run init command
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();
    
    // Check .gitignore preserved existing content and added new
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("*.tmp"));
    assert!(content.contains(".refaktor/"));
    assert!(content.contains("# Refaktor workspace"));
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
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--local"])
        .assert()
        .success()
        .stderr(predicates::str::contains(".git/info/exclude"));
    
    // Check .git/info/exclude was created with correct content
    let exclude_path = temp_dir.path().join(".git/info/exclude");
    assert!(exclude_path.exists());
    let content = std::fs::read_to_string(exclude_path).unwrap();
    assert!(content.contains(".refaktor/"));
}

#[test]
fn test_init_command_not_in_git_repo() {
    // Test --local flag when not in a git repo (should fail)
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
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
    temp_dir.child(".gitignore").write_str("/.refaktor\n").unwrap();
    
    // Run init command - should detect existing pattern
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
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
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();
    
    // Check proper formatting
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    assert!(content.contains("target/\n"));  // Should have newline after existing content
    assert!(content.contains("\n# Refaktor workspace\n"));  // Should have blank line before comment
    assert!(content.ends_with(".refaktor/\n"));  // Should end with newline
}

#[test]
fn test_init_check_mode() {
    // Test --check flag functionality
    let temp_dir = TempDir::new().unwrap();
    
    // When .refaktor is not ignored, should exit with code 1
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--check"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(".refaktor is NOT ignored"));
    
    // Add .refaktor to .gitignore
    temp_dir.child(".gitignore").write_str(".refaktor/\n").unwrap();
    
    // Now --check should succeed
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["init", "--check"])
        .assert()
        .success()
        .stderr(predicates::str::contains(".refaktor is properly ignored"));
}

#[test]
fn test_auto_init_flag() {
    // Test --auto-init=repo flag
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    // Run plan with --auto-init=repo
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["--auto-init=repo", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    
    // Check .gitignore was created
    temp_dir.child(".gitignore").assert(predicates::str::contains(".refaktor/"));
}

#[test]
fn test_no_auto_init_flag() {
    // Test --no-auto-init flag prevents initialization
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    // Run plan with --no-auto-init
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["--no-auto-init", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    
    // Check .gitignore was NOT created
    assert!(!temp_dir.path().join(".gitignore").exists());
}

#[test]
fn test_yes_flag_auto_init() {
    // Test -y flag chooses repo mode automatically
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    // Run plan with -y flag
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["-y", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    
    // Check .gitignore was created (default repo mode)
    temp_dir.child(".gitignore").assert(predicates::str::contains(".refaktor/"));
}

#[test]
fn test_auto_init_local_mode() {
    // Test --auto-init=local flag
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");
    
    // Run plan with --auto-init=local
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["--auto-init=local", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    
    // Check .git/info/exclude was created
    let exclude_path = temp_dir.path().join(".git/info/exclude");
    assert!(exclude_path.exists());
    let content = std::fs::read_to_string(exclude_path).unwrap();
    assert!(content.contains(".refaktor/"));
}

#[test]
fn test_auto_init_idempotent() {
    // Test that auto-init doesn't duplicate entries
    let temp_dir = TempDir::new().unwrap();
    temp_dir.child("test.rs").write_str("fn old_name() {}").unwrap();
    
    // First run with auto-init
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["--auto-init=repo", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    
    // Second run - should not duplicate
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["--auto-init=repo", "plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success();
    
    // Check only one entry exists
    let content = std::fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    let count = content.matches(".refaktor/").count();
    assert_eq!(count, 1, "Should only have one .refaktor/ entry");
}