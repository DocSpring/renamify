use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_rename_command_with_train_case_patterns() {
    // E2E test for Train-Case patterns like "Rename-Tool-Core-Engine"
    let temp_dir = TempDir::new().unwrap();

    // Create test file with Train-Case patterns in various contexts
    let test_content = r#"# Documentation

## Configuration
- Rename-Tool-Specific-Settings for configuration
- Use Rename-Tool-Core-Engine for processing  
- The Rename-Tool-Based-Solution works well

## In String Literals
    // - "Rename-Tool-Specific-Settings" -> "Smart-Search-And-Replace-Specific-Settings"
    // - "Rename-Tool-Core-Engine" -> "Smart-Search-And-Replace-Core-Engine"
    // - "Rename-Tool-Based-Solution" -> "Smart-Search-And-Replace-Based-Solution"

## Mixed Patterns
- rename-tool-specific (lowercase kebab)
- Rename-Tool-CLI (mixed)
- RENAME-TOOL-DEBUG (screaming)
"#;

    temp_dir.child("test.md").write_str(test_content).unwrap();

    // Run rename command with -y to auto-approve
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["rename", "rename_tool", "smart_search_and_replace", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

    // Read the modified content
    let updated_content = std::fs::read_to_string(temp_dir.path().join("test.md")).unwrap();

    // Debug: Print the updated content to see what happened
    eprintln!("=== Updated content ===\n{}", updated_content);

    // Verify Train-Case patterns were replaced
    assert!(
        !updated_content.contains("Rename-Tool-Core-Engine"),
        "Should not contain 'Rename-Tool-Core-Engine' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-Core-Engine"),
        "Should contain 'Smart-Search-And-Replace-Core-Engine' after replacement"
    );

    assert!(
        !updated_content.contains("Rename-Tool-Specific-Settings"),
        "Should not contain 'Rename-Tool-Specific-Settings' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-Specific-Settings"),
        "Should contain 'Smart-Search-And-Replace-Specific-Settings' after replacement"
    );

    assert!(
        !updated_content.contains("Rename-Tool-Based-Solution"),
        "Should not contain 'Rename-Tool-Based-Solution' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-Based-Solution"),
        "Should contain 'Smart-Search-And-Replace-Based-Solution' after replacement"
    );

    // Check other case variants too
    assert!(
        !updated_content.contains("rename-tool-specific"),
        "Should not contain 'rename-tool-specific' after replacement"
    );

    assert!(
        updated_content.contains("smart-search-and-replace-specific"),
        "Should contain 'smart-search-and-replace-specific' after replacement"
    );

    assert!(
        !updated_content.contains("Rename-Tool-CLI"),
        "Should not contain 'Rename-Tool-CLI' after replacement"
    );

    assert!(
        updated_content.contains("Smart-Search-And-Replace-CLI"),
        "Should contain 'Smart-Search-And-Replace-CLI' after replacement"
    );

    assert!(
        !updated_content.contains("RENAME-TOOL-DEBUG"),
        "Should not contain 'RENAME-TOOL-DEBUG' after replacement"
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
            r#"# Rename-Tool-Core-Engine Documentation

The Rename-Tool-Core-Engine is the main processing unit.

## Rename-Tool-Specific-Settings

Configure using Rename-Tool-Specific-Settings:
- Setting 1: Rename-Tool-Based-Solution
- Setting 2: Use Rename-Tool-Core-Engine
"#,
        )
        .unwrap();

    temp_dir
        .child("test.rs")
        .write_str(
            r#"// Tests for Rename-Tool-Core-Engine
fn test_renamify() {
    let engine = "Rename-Tool-Core-Engine";
    let settings = "Rename-Tool-Specific-Settings";
    println!("Using Rename-Tool-Based-Solution");
}
"#,
        )
        .unwrap();

    // First create a plan
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["plan", "rename_tool", "smart_search_and_replace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("docs/README.md"))
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("Rename-Tool-Core-Engine"));

    // Then apply the plan
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(["apply"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied"));

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
        !readme_content.contains("Rename-Tool-"),
        "README should not contain any 'Rename-Tool-' patterns"
    );
    assert!(
        !test_content.contains("Rename-Tool-"),
        "test.rs should not contain any 'Rename-Tool-' patterns"
    );
}

#[test]
fn test_undo_train_case_replacements() {
    // Test that undo works correctly with Train-Case replacements
    let temp_dir = TempDir::new().unwrap();

    // Create a file with Train-Case patterns
    let original_content = r#"# Rename-Tool-Core-Engine

Using Rename-Tool-Specific-Settings for configuration.
The Rename-Tool-Based-Solution is working.
"#;

    temp_dir
        .child("doc.md")
        .write_str(original_content)
        .unwrap();

    // Change to the temp directory for the commands
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Apply rename using the core rename operation directly
    use renamify_core::rename_operation;
    rename_operation(
        "rename_tool",
        "smart_search_and_replace",
        vec![], // paths (empty = current dir)
        &[],    // include
        &[],    // exclude
        0,      // unrestricted_level
        true,   // rename_files
        true,   // rename_dirs
        &[],    // exclude_styles
        &[],    // include_styles
        &[],    // only_styles
        &[],    // exclude_match
        None,   // exclude_matching_lines
        None,   // preview_format
        false,  // commit
        false,  // large
        false,  // force_with_conflicts
        false,  // rename_root
        false,  // no_rename_root
        false,  // dry_run
        false,  // no_acronyms
        &[],    // include_acronyms
        &[],    // exclude_acronyms
        &[],    // only_acronyms
        true,   // auto_approve
        true,   // use_color
    )
    .unwrap();

    // Verify changes were applied
    let changed_content = std::fs::read_to_string("doc.md").unwrap();
    assert!(changed_content.contains("Smart-Search-And-Replace-Core-Engine"));
    assert!(!changed_content.contains("Rename-Tool-Core-Engine"));

    // Undo the changes using the core undo operation directly
    use renamify_core::undo_operation;
    undo_operation("latest", None).unwrap();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // Verify content is back to original
    let restored_content = std::fs::read_to_string(temp_dir.path().join("doc.md")).unwrap();

    // On Windows, line endings will be CRLF after write_str
    #[cfg(windows)]
    let expected_content = original_content.replace("\n", "\r\n");
    #[cfg(not(windows))]
    let expected_content = original_content.to_string();

    assert_eq!(
        restored_content, expected_content,
        "Content should be restored to original after undo"
    );
    assert!(restored_content.contains("Rename-Tool-Core-Engine"));
    assert!(restored_content.contains("Rename-Tool-Specific-Settings"));
    assert!(restored_content.contains("Rename-Tool-Based-Solution"));
}

#[test]
fn test_screaming_train_exclusion_fallback() {
    let temp_dir = TempDir::new().unwrap();

    let test_content = r#"# Test File
- RENAME-TOOL-DEBUG pattern here
"#;

    temp_dir.child("test.md").write_str(test_content).unwrap();

    // Run rename command excluding ScreamingTrain style, should fallback to different behavior
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.current_dir(temp_dir.path())
        .args([
            "rename",
            "rename_tool",
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

    // When ScreamingTrain is excluded, no other style can handle RENAME-TOOL-DEBUG
    // so it should remain unchanged (this is correct behavior)
    assert!(
        updated_content.contains("RENAME-TOOL-DEBUG"),
        "Should still contain original 'RENAME-TOOL-DEBUG' when ScreamingTrain is excluded (no other style can handle it)"
    );

    // Should NOT use ScreamingTrain style since it's excluded
    assert!(
        !updated_content.contains("SMART-SEARCH-AND-REPLACE-DEBUG"),
        "Should NOT contain ScreamingTrain style when excluded"
    );
}
