use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_search_command_basic() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("hello_world").unwrap();
    temp.child("test2.txt").write_str("helloWorld").unwrap();

    // Search for a pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world"])
        .assert()
        .success()
        .stdout(predicates::str::contains("hello_world"))
        .stdout(predicates::str::contains("helloWorld"));
}

#[test]
fn test_search_command_with_styles() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt")
        .write_str("hello_world and HELLO_WORLD")
        .unwrap();

    // Search with specific styles
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "search",
            "hello_world",
            "--only-styles",
            "snake,screaming-snake",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("hello_world"))
        .stdout(predicates::str::contains("HELLO_WORLD"));
}

#[test]
fn test_search_command_json_output() {
    let temp = TempDir::new().unwrap();

    temp.child("test.txt").write_str("hello_world").unwrap();

    // Search with JSON output
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--output", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"total_matches\""));
}

#[test]
fn test_search_command_with_includes() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("hello_world").unwrap();
    temp.child("test.rs").write_str("hello_world").unwrap();
    temp.child("test.md").write_str("hello_world").unwrap();

    // Search only in .txt files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--include", "*.txt"])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"))
        .stdout(predicates::str::contains("test.rs").not());
}

#[test]
fn test_search_command_with_excludes() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("hello_world").unwrap();
    temp.child("test.rs").write_str("hello_world").unwrap();
    temp.child("test.md").write_str("hello_world").unwrap();

    // Search excluding .md files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--exclude", "*.md"])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"))
        .stdout(predicates::str::contains("test.rs"))
        .stdout(predicates::str::contains("test.md").not());
}

#[test]
fn test_search_command_quiet_mode() {
    let temp = TempDir::new().unwrap();

    temp.child("test.txt").write_str("hello_world").unwrap();

    // Search with quiet mode
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--quiet"])
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}

#[test]
fn test_search_command_no_matches() {
    let temp = TempDir::new().unwrap();

    temp.child("test.txt").write_str("foo bar").unwrap();

    // Search for non-existent pattern
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world"])
        .assert()
        .success()
        .stdout(predicates::str::contains("0 matches"));
}

#[test]
fn test_search_command_with_acronyms() {
    let temp = TempDir::new().unwrap();

    temp.child("test.txt")
        .write_str("HTTP_API and httpApi")
        .unwrap();

    // Search with acronym support
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "http_api", "--include-acronyms", "HTTP,API"])
        .assert()
        .success()
        .stdout(predicates::str::contains("HTTP_API"))
        .stdout(predicates::str::contains("httpApi"));
}

#[test]
fn test_search_command_unrestricted() {
    let temp = TempDir::new().unwrap();

    // Create .gitignore and ignored file
    temp.child(".gitignore").write_str("ignored.txt").unwrap();
    temp.child("ignored.txt").write_str("hello_world").unwrap();
    temp.child("normal.txt").write_str("hello_world").unwrap();

    // Without -u, ignored file shouldn't be searched
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world"])
        .assert()
        .success()
        .stdout(predicates::str::contains("normal.txt"))
        .stdout(predicates::str::contains("ignored.txt").not());

    // With -u, ignored file should be searched
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "-u"])
        .assert()
        .success()
        .stdout(predicates::str::contains("normal.txt"))
        .stdout(predicates::str::contains("ignored.txt"));
}

#[test]
fn test_search_command_preview_formats() {
    let temp = TempDir::new().unwrap();

    temp.child("test.txt").write_str("hello_world").unwrap();

    // Test different preview formats
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--preview", "matches"])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--preview", "table"])
        .assert()
        .success()
        .stdout(predicates::str::contains("File"));

    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world", "--preview", "summary"])
        .assert()
        .success();
}

#[test]
fn test_search_command_multiple_files() {
    let temp = TempDir::new().unwrap();

    // Create multiple files with matches
    temp.child("file1.txt").write_str("hello_world").unwrap();
    temp.child("file2.txt").write_str("helloWorld").unwrap();
    temp.child("dir/file3.txt")
        .write_str("HELLO_WORLD")
        .unwrap();

    // Search across all files
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world"])
        .assert()
        .success()
        .stdout(predicates::str::contains("file1.txt"))
        .stdout(predicates::str::contains("file2.txt"))
        .stdout(predicates::str::contains("file3.txt"));
}

#[test]
fn test_search_command_with_fixed_table_width() {
    let temp = TempDir::new().unwrap();

    temp.child("test.txt").write_str("hello_world").unwrap();

    // Search with fixed table width flag
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "search",
            "hello_world",
            "--preview",
            "table",
            "--fixed-table-width",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("File"));
}

#[test]
fn test_search_command_with_multiple_excludes() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("hello_world").unwrap();
    temp.child("test.rs").write_str("hello_world").unwrap();
    temp.child("test.md").write_str("hello_world").unwrap();
    temp.child("test.py").write_str("hello_world").unwrap();

    // Search excluding multiple patterns
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "search",
            "hello_world",
            "--exclude",
            "*.md",
            "--exclude",
            "*.py",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"))
        .stdout(predicates::str::contains("test.rs"))
        .stdout(predicates::str::contains("test.md").not())
        .stdout(predicates::str::contains("test.py").not());
}

#[test]
fn test_search_command_with_multiple_includes() {
    let temp = TempDir::new().unwrap();

    // Create test files
    temp.child("test.txt").write_str("hello_world").unwrap();
    temp.child("test.rs").write_str("hello_world").unwrap();
    temp.child("test.md").write_str("hello_world").unwrap();
    temp.child("test.py").write_str("hello_world").unwrap();

    // Search only in specific file types
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "search",
            "hello_world",
            "--include",
            "*.rs",
            "--include",
            "*.py",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt").not())
        .stdout(predicates::str::contains("test.rs"))
        .stdout(predicates::str::contains("test.md").not())
        .stdout(predicates::str::contains("test.py"));
}

#[test]
fn test_search_command_case_variations() {
    let temp = TempDir::new().unwrap();

    // Create file with various case styles
    temp.child("test.txt")
        .write_str("hello_world helloWorld HelloWorld HELLO_WORLD hello-world Hello-World")
        .unwrap();

    // Search for all variations
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp.path())
        .args(["search", "hello_world"])
        .assert()
        .success()
        .stdout(predicates::str::contains("6 matches"));
}
