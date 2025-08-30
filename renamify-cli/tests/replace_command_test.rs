use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_replace_command_literal() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt")
        .write_str("hello world\nhello rust\nworld of rust")
        .unwrap();

    // Run replace with literal pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", "--no-regex", "hello", "goodbye", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("goodbye world"))
        .stdout(predicates::str::contains("goodbye rust"));
}

#[test]
fn test_replace_command_regex() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt")
        .write_str("test123\ntest456\ncode789")
        .unwrap();

    // Run replace with regex pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", r"test(\d+)", "result$1", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("result123"))
        .stdout(predicates::str::contains("result456"));
}

#[test]
fn test_replace_command_with_includes() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("foo bar").unwrap();
    temp.child("test.md").write_str("foo bar").unwrap();
    temp.child("test.rs").write_str("foo bar").unwrap();

    // Run replace only on .txt files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "foo",
            "baz",
            "--include",
            "*.txt",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"))
        .stdout(predicates::str::contains("test.md").not())
        .stdout(predicates::str::contains("test.rs").not());
}

#[test]
fn test_replace_command_with_excludes() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("foo bar").unwrap();
    temp.child("test.md").write_str("foo bar").unwrap();
    temp.child("test.rs").write_str("foo bar").unwrap();

    // Run replace excluding .md files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "foo",
            "baz",
            "--exclude",
            "*.md",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"))
        .stdout(predicates::str::contains("test.md").not())
        .stdout(predicates::str::contains("test.rs"));
}

#[test]
fn test_replace_command_rename_files() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("foo_bar.txt").write_str("content").unwrap();

    // Run replace with file renaming
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", "--no-regex", "foo", "baz", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("baz_bar.txt"));
}

#[test]
fn test_replace_command_no_rename_files() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("foo_bar.txt").write_str("foo content").unwrap();

    // Run replace without file renaming
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "foo",
            "baz",
            "--no-rename-files",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("baz content"))
        .stdout(predicates::str::contains("baz_bar.txt").not());
}

#[test]
fn test_replace_command_regex_capture_groups() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt")
        .write_str("fn foo_bar() {}\nfn baz_qux() {}")
        .unwrap();

    // Run replace with capture groups
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", r"fn (\w+)_(\w+)", "function $2_$1", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("function bar_foo"))
        .stdout(predicates::str::contains("function qux_baz"));
}

#[test]
fn test_replace_command_exclude_matching_lines() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt")
        .write_str("foo bar\n// TODO: foo bar\nfoo baz")
        .unwrap();

    // Run replace excluding lines with TODO
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "foo",
            "replaced",
            "--exclude-matching-lines",
            "TODO",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("replaced bar"))
        .stdout(predicates::str::contains("replaced baz"));
}

#[test]
fn test_replace_command_json_output() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt").write_str("hello world").unwrap();

    // Run replace with JSON output
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "hello",
            "hi",
            "--dry-run",
            "--output",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"id\""))
        .stdout(predicates::str::contains("\"total_matches\""));
}

#[test]
fn test_replace_command_force_apply() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt").write_str("hello world").unwrap();

    // Run replace and actually apply changes
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", "--no-regex", "hello", "hi", "--yes"])
        .assert()
        .success();

    // Verify the file was actually changed
    temp.child("test.txt").assert("hi world");
}

#[test]
fn test_replace_command_multiple_files() {
    let temp = TempDir::new().unwrap();

    // Create multiple test files
    temp.child("file1.txt").write_str("hello world").unwrap();
    temp.child("file2.txt").write_str("hello rust").unwrap();
    temp.child("subdir").create_dir_all().unwrap();
    temp.child("subdir/file3.txt")
        .write_str("hello code")
        .unwrap();

    // Run replace across all files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", "--no-regex", "hello", "goodbye", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("file1.txt"))
        .stdout(predicates::str::contains("file2.txt"))
        .stdout(predicates::str::contains("file3.txt"));
}

#[test]
fn test_replace_command_invalid_regex() {
    let temp = TempDir::new().unwrap();

    // Run replace with invalid regex
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["replace", "[invalid(regex", "replacement", "--dry-run"])
        .assert()
        .failure()
        .stderr(
            predicates::str::contains("regex")
                .or(predicates::str::contains("pattern"))
                .or(predicates::str::contains("invalid")),
        );
}

#[test]
fn test_replace_command_quiet_mode() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt").write_str("hello world").unwrap();

    // Run replace with quiet mode
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "hello",
            "hi",
            "--quiet",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}

#[test]
fn test_replace_command_table_preview() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt").write_str("hello world").unwrap();

    // Run replace with table preview
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "hello",
            "hi",
            "--preview",
            "table",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("File"))
        .stdout(predicates::str::contains("Matches"));
}

#[test]
fn test_replace_command_diff_preview() {
    let temp = TempDir::new().unwrap();

    // Create test file
    temp.child("test.txt").write_str("hello world").unwrap();

    // Run replace with diff preview
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "replace",
            "--no-regex",
            "hello",
            "hi",
            "--preview",
            "diff",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("-hello world"))
        .stdout(predicates::str::contains("+hi world"));
}
