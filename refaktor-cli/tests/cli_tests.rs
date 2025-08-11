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
        .stdout(predicate::str::contains("main.rs").not().or(predicate::str::contains("tests/test.rs").not()));
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
fn test_apply_command_not_implemented() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.args(["apply", "--plan", "plan.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_undo_command_not_implemented() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.args(["undo", "abc123"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_redo_command_not_implemented() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.args(["redo", "abc123"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_status_command_not_implemented() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_history_command_not_implemented() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.arg("history")
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_history_command_with_limit() {
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.args(["history", "--limit", "10"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
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