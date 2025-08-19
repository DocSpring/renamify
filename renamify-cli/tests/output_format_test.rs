use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn test_rename_output_json() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() { let old_name = 42; }")
        .unwrap();

    // Run rename with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check JSON structure
    assert!(
        json.get("success").is_some(),
        "JSON should have 'success' field"
    );
    assert!(
        json.get("plan_id").is_some(),
        "JSON should have 'plan_id' field"
    );
    assert!(
        json.get("operation").is_some(),
        "JSON should have 'operation' field"
    );
    assert_eq!(json["operation"], "rename");

    // Should have summary
    assert!(
        json.get("summary").is_some(),
        "JSON should have 'summary' field"
    );
    let summary = &json["summary"];
    assert!(summary.get("files_changed").is_some());
    assert!(summary.get("replacements").is_some());

    // Verify the file was actually changed
    let content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(content.contains("new_name"));
    assert!(!content.contains("old_name"));
}

#[test]
fn test_plan_output_json() {
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check it's a Plan result object
    assert!(
        json.get("plan_id").is_some(),
        "Plan should have 'plan_id' field"
    );
    assert!(
        json.get("search").is_some(),
        "Plan should have 'search' field"
    );
    assert!(
        json.get("replace").is_some(),
        "Plan should have 'replace' field"
    );
    assert!(
        json.get("summary").is_some(),
        "Plan should have 'summary' field"
    );
    assert_eq!(json["search"], "old_name");
    assert_eq!(json["replace"], "new_name");
}

#[test]
fn test_search_output_json() {
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn search_term() {}")
        .unwrap();

    // Run search with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["search", "search_term", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check it's a Plan result object (search returns Plan structure)
    assert!(json.get("plan_id").is_some());
    assert!(json.get("search").is_some());
    assert_eq!(json["search"], "search_term");
    assert_eq!(json["replace"], "");
}

#[test]
fn test_history_output_json() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize .renamify directory
    std::fs::create_dir_all(temp_dir.path().join(".renamify")).unwrap();

    // Run history with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["history", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Should be an array (even if empty)
    assert!(json.is_array(), "History output should be an array");
}

#[test]
fn test_status_output_json() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize .renamify directory
    std::fs::create_dir_all(temp_dir.path().join(".renamify")).unwrap();

    // Run status with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["status", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check status structure
    assert!(
        json.get("pending_plan").is_some(),
        "Status should have 'pending_plan' field"
    );
    assert!(
        json.get("history_count").is_some(),
        "Status should have 'history_count' field"
    );
}

#[test]
fn test_quiet_option_suppresses_output() {
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with --quiet
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--quiet"])
        .output()
        .expect("Failed to execute command");

    // Should have no output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "Quiet mode should produce no output"
    );
}

#[test]
fn test_quiet_overrides_preview() {
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with both --preview table and --quiet
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args([
            "plan",
            "old_name",
            "new_name",
            "--preview",
            "table",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    // Should have no output (quiet overrides preview)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "Quiet should override preview"
    );
}

#[test]
fn test_output_json_with_quiet_shows_json() {
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan with both --output json and --quiet
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args([
            "plan", "old_name", "new_name", "--output", "json", "--quiet",
        ])
        .output()
        .expect("Failed to execute command");

    // Should still output JSON (--output json takes precedence over --quiet)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .expect("With --output json, should still produce JSON even with --quiet");

    assert!(json.get("plan_id").is_some());
}

#[test]
fn test_apply_output_json() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test file
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // First create a plan
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name"])
        .assert()
        .success();

    // Now apply with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["apply", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check JSON structure
    assert!(json.get("success").is_some());
    assert!(json.get("plan_id").is_some());
    assert_eq!(json["operation"], "apply");
}

#[test]
fn test_undo_output_json() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file and apply a rename
    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Apply a rename first
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "old_name", "new_name", "-y"])
        .assert()
        .success();

    // Now undo with --output json
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .args(["undo", "latest", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check JSON structure
    assert!(json.get("success").is_some());
    assert!(json.get("operation").is_some());
    assert_eq!(json["operation"], "undo");
}

#[test]
fn test_output_default_is_summary() {
    let temp_dir = TempDir::new().unwrap();

    temp_dir
        .child("test.rs")
        .write_str("fn old_name() {}")
        .unwrap();

    // Run plan without --output flag (should default to summary)
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "old_name", "new_name", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Renamify plan:"))
        .stdout(predicate::str::contains("Edits:"));
}
