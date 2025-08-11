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