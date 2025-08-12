use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_rename_command_with_train_case_patterns() {
    // E2E test for Train-Case patterns like "Refaktor-Core-Engine"
    let temp_dir = TempDir::new().unwrap();

    // Create test file with Train-Case patterns in various contexts
    let test_content = r#"# Documentation

## Configuration
- Refaktor-Specific-Settings for configuration
- Use Refaktor-Core-Engine for processing  
- The Refaktor-Based-Solution works well

## In String Literals
    // - "Refaktor-Specific-Settings" -> "Smart-Search-And-Replace-Specific-Settings"
    // - "Refaktor-Core-Engine" -> "Smart-Search-And-Replace-Core-Engine"
    // - "Refaktor-Based-Solution" -> "Smart-Search-And-Replace-Based-Solution"

## Mixed Patterns
- refaktor-specific (lowercase kebab)
- Refaktor-CLI (mixed)
- REFAKTOR-DEBUG (screaming)
"#;

    temp_dir.child("test.md").write_str(test_content).unwrap();

    // Run rename command with -y to auto-approve
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "refaktor", "smart_search_and_replace", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // Read the modified content
    let updated_content = std::fs::read_to_string(temp_dir.path().join("test.md")).unwrap();

    // Debug: Print the updated content to see what happened
    eprintln!("=== Updated content ===\n{}", updated_content);

    // Verify Train-Case patterns were replaced
    assert!(
        !updated_content.contains("Refaktor-Core-Engine"),
        "Should not contain 'Refaktor-Core-Engine' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-Core-Engine"),
        "Should contain 'Smart-Search-And-Replace-Core-Engine' after replacement"
    );

    assert!(
        !updated_content.contains("Refaktor-Specific-Settings"),
        "Should not contain 'Refaktor-Specific-Settings' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-Specific-Settings"),
        "Should contain 'Smart-Search-And-Replace-Specific-Settings' after replacement"
    );

    assert!(
        !updated_content.contains("Refaktor-Based-Solution"),
        "Should not contain 'Refaktor-Based-Solution' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-Based-Solution"),
        "Should contain 'Smart-Search-And-Replace-Based-Solution' after replacement"
    );

    // Check other case variants too
    assert!(
        !updated_content.contains("refaktor-specific"),
        "Should not contain 'refaktor-specific' after replacement"
    );

    assert!(
        updated_content.contains("smart-search-and-replace-specific"),
        "Should contain 'smart-search-and-replace-specific' after replacement"
    );

    assert!(
        !updated_content.contains("Refaktor-CLI"),
        "Should not contain 'Refaktor-CLI' after replacement"
    );

    assert!(
        updated_content.contains("SmartSearchAndReplace-CLI"),
        "Should contain 'SmartSearchAndReplace-CLI' after replacement"
    );

    assert!(
        !updated_content.contains("REFAKTOR-DEBUG"),
        "Should not contain 'REFAKTOR-DEBUG' after replacement"
    );

    assert!(
        updated_content.contains("SMART-SEARCH-AND-REPLACE-DEBUG"),
        "Should contain 'SMART-SEARCH-AND-REPLACE-DEBUG' after replacement"
    );
}

#[test]
fn test_plan_and_apply_with_train_case_patterns() {
    // Test the full plan -> apply flow with Train-Case patterns
    let temp_dir = TempDir::new().unwrap();

    // Create test files with Train-Case patterns
    temp_dir.child("docs").create_dir_all().unwrap();
    temp_dir
        .child("docs/README.md")
        .write_str(
            r#"# Refaktor-Core-Engine Documentation

The Refaktor-Core-Engine is the main processing unit.

## Refaktor-Specific-Settings

Configure using Refaktor-Specific-Settings:
- Setting 1: Refaktor-Based-Solution
- Setting 2: Use Refaktor-Core-Engine
"#,
        )
        .unwrap();

    temp_dir
        .child("test.rs")
        .write_str(
            r#"// Tests for Refaktor-Core-Engine
fn test_refaktor() {
    let engine = "Refaktor-Core-Engine";
    let settings = "Refaktor-Specific-Settings";
    println!("Using Refaktor-Based-Solution");
}
"#,
        )
        .unwrap();

    // First create a plan
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "refaktor", "smart_search_and_replace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("docs/README.md"))
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("Refaktor-Core-Engine"));

    // Then apply the plan
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Plan applied successfully!"));

    // Verify the files were modified correctly
    let readme_content = std::fs::read_to_string(temp_dir.path().join("docs/README.md")).unwrap();
    assert!(
        readme_content.contains("Smart-Search-And-Replace-Core-Engine"),
        "README should contain 'Smart-Search-And-Replace-Core-Engine'"
    );
    assert!(
        readme_content.contains("Smart-Search-And-Replace-Specific-Settings"),
        "README should contain 'Smart-Search-And-Replace-Specific-Settings'"
    );
    assert!(
        readme_content.contains("Smart-Search-And-Replace-Based-Solution"),
        "README should contain 'Smart-Search-And-Replace-Based-Solution'"
    );

    let test_content = std::fs::read_to_string(temp_dir.path().join("test.rs")).unwrap();
    assert!(
        test_content.contains("Smart-Search-And-Replace-Core-Engine"),
        "test.rs should contain 'Smart-Search-And-Replace-Core-Engine'"
    );
    assert!(
        test_content.contains("Smart-Search-And-Replace-Specific-Settings"),
        "test.rs should contain 'Smart-Search-And-Replace-Specific-Settings'"
    );
    assert!(
        test_content.contains("Smart-Search-And-Replace-Based-Solution"),
        "test.rs should contain 'Smart-Search-And-Replace-Based-Solution'"
    );

    // Ensure no old patterns remain
    assert!(
        !readme_content.contains("Refaktor-"),
        "README should not contain any 'Refaktor-' patterns"
    );
    assert!(
        !test_content.contains("Refaktor-"),
        "test.rs should not contain any 'Refaktor-' patterns"
    );
}

#[test]
fn test_undo_train_case_replacements() {
    // Test that undo works correctly with Train-Case replacements
    let temp_dir = TempDir::new().unwrap();

    // Create a file with Train-Case patterns
    let original_content = r#"# Refaktor-Core-Engine

Using Refaktor-Specific-Settings for configuration.
The Refaktor-Based-Solution is working.
"#;

    temp_dir
        .child("doc.md")
        .write_str(original_content)
        .unwrap();

    // Apply rename
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "refaktor", "smart_search_and_replace", "-y"])
        .assert()
        .success();

    // Verify changes were applied
    let changed_content = std::fs::read_to_string(temp_dir.path().join("doc.md")).unwrap();
    assert!(changed_content.contains("Smart-Search-And-Replace-Core-Engine"));
    assert!(!changed_content.contains("Refaktor-Core-Engine"));

    // Undo the changes
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["undo", "latest"])
        .assert()
        .success();

    // Verify content is back to original
    let restored_content = std::fs::read_to_string(temp_dir.path().join("doc.md")).unwrap();
    assert_eq!(
        restored_content, original_content,
        "Content should be restored to original after undo"
    );
    assert!(restored_content.contains("Refaktor-Core-Engine"));
    assert!(restored_content.contains("Refaktor-Specific-Settings"));
    assert!(restored_content.contains("Refaktor-Based-Solution"));
}

#[test]
fn test_screaming_train_exclusion_fallback() {
    let temp_dir = TempDir::new().unwrap();

    let test_content = r#"# Test File
- REFAKTOR-DEBUG pattern here
"#;

    temp_dir.child("test.md").write_str(test_content).unwrap();

    // Run rename command excluding ScreamingTrain style, should fallback to different behavior
    let mut cmd = Command::cargo_bin("refaktor").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "rename",
            "refaktor",
            "smart_search_and_replace",
            "--exclude-styles",
            "screaming-train",
            "-y",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // Read the modified content
    let updated_content = std::fs::read_to_string(temp_dir.path().join("test.md")).unwrap();

    // Debug: Print the updated content
    eprintln!(
        "=== Updated content with ScreamingTrain excluded ===\n{}",
        updated_content
    );

    // When ScreamingTrain is excluded, no other style can handle REFAKTOR-DEBUG
    // so it should remain unchanged (this is correct behavior)
    assert!(
        updated_content.contains("REFAKTOR-DEBUG"),
        "Should still contain original 'REFAKTOR-DEBUG' when ScreamingTrain is excluded (no other style can handle it)"
    );

    // Should NOT use ScreamingTrain style since it's excluded
    assert!(
        !updated_content.contains("SMART-SEARCH-AND-REPLACE-DEBUG"),
        "Should NOT contain ScreamingTrain style when excluded"
    );
}
