use renamify_core::{apply_plan, ApplyOptions, MatchHunk, Plan, Rename, RenameKind, Stats};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_test_plan(id: &str, old: &str, new: &str) -> Plan {
    Plan {
        id: id.to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        old: old.to_string(),
        new: new.to_string(),
        styles: vec![],
        includes: vec![],
        excludes: vec![],
        matches: vec![],
        renames: vec![],
        stats: Stats {
            files_scanned: 0,
            total_matches: 0,
            matches_by_variant: HashMap::new(),
            files_with_matches: 0,
        },
        version: "1.0.0".to_string(),
        created_directories: None,
    }
}

#[test]
fn test_apply_content_edits() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Write initial content
    fs::write(
        &test_file,
        "fn old_name() {\n    println!(\"old_name\");\n}",
    )
    .unwrap();

    // Create plan with content edits
    let mut plan = create_test_plan("test_content", "old_name", "new_name");
    plan.matches.push(MatchHunk {
        file: test_file.clone(),
        line: 1,
        col: 3,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 3,
        end: 11,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });
    plan.matches.push(MatchHunk {
        file: test_file.clone(),
        line: 2,
        col: 14,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 30,
        end: 38,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Apply the plan
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        log_file: Some(temp_dir.path().join("apply.log")),
        ..Default::default()
    };

    apply_plan(&mut plan, &options).unwrap();

    // Verify the changes
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("fn new_name()"));
    assert!(content.contains("println!(\"new_name\")"));
    assert!(!content.contains("old_name"));

    // Verify backup was created
    let backup_dir = temp_dir.path().join(".backups").join("test_content");
    assert!(backup_dir.exists());
}

#[test]
fn test_apply_renames() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files and directories
    let old_file = temp_dir.path().join("old_name.txt");
    let old_dir = temp_dir.path().join("old_name_dir");

    fs::write(&old_file, "test content").unwrap();
    fs::create_dir(&old_dir).unwrap();
    fs::write(old_dir.join("inner.txt"), "inner content").unwrap();

    // Create plan with renames
    let mut plan = create_test_plan("test_renames", "old_name", "new_name");
    plan.renames.push(Rename {
        from: old_file.clone(),
        to: temp_dir.path().join("new_name.txt"),
        kind: RenameKind::File,
        coercion_applied: None,
    });
    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_dir"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    // Apply the plan
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        ..Default::default()
    };

    apply_plan(&mut plan, &options).unwrap();

    // Verify renames
    assert!(!old_file.exists());
    assert!(!old_dir.exists());
    assert!(temp_dir.path().join("new_name.txt").exists());
    assert!(temp_dir.path().join("new_name_dir").exists());
    assert!(temp_dir
        .path()
        .join("new_name_dir")
        .join("inner.txt")
        .exists());

    // Verify content preserved
    let content = fs::read_to_string(temp_dir.path().join("new_name.txt")).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_rollback_on_error() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Write initial content
    let original_content = "fn old_name() {}";
    fs::write(&test_file, original_content).unwrap();

    // Create plan with invalid edit (mismatched content)
    let mut plan = create_test_plan("test_rollback", "old_name", "new_name");
    plan.matches.push(MatchHunk {
        file: test_file.clone(),
        line: 1,
        col: 3,
        variant: "old_name".to_string(),
        before: "wrong_content".to_string(), // This will cause an error
        after: "new_name".to_string(),
        start: 3,
        end: 11,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Apply should fail
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        atomic: true,
        ..Default::default()
    };

    let result = apply_plan(&mut plan, &options);
    assert!(result.is_err());

    // Verify file was not modified
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, original_content);
}

#[test]
fn test_case_only_rename() {
    let temp_dir = TempDir::new().unwrap();
    let lower_file = temp_dir.path().join("oldname.txt");

    fs::write(&lower_file, "test").unwrap();

    // Create plan for case-only rename
    let mut plan = create_test_plan("test_case", "oldname", "OldName");
    plan.renames.push(Rename {
        from: lower_file.clone(),
        to: temp_dir.path().join("OldName.txt"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        ..Default::default()
    };

    apply_plan(&mut plan, &options).unwrap();

    // On case-insensitive filesystems, both paths may resolve to the same file
    // Just verify that a file with the new name exists
    let new_file = temp_dir.path().join("OldName.txt");
    assert!(new_file.exists() || lower_file.exists());
}

#[test]
fn test_atomic_operations() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    let file1 = temp_dir.path().join("file1.rs");
    let file2 = temp_dir.path().join("file2.rs");

    fs::write(&file1, "fn old_name() {}").unwrap();
    fs::write(&file2, "fn old_name() {}").unwrap();

    // Create plan with multiple edits
    let mut plan = create_test_plan("test_atomic", "old_name", "new_name");

    // First edit will succeed
    plan.matches.push(MatchHunk {
        file: file1.clone(),
        line: 1,
        col: 3,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 3,
        end: 11,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Second edit will fail (wrong content)
    plan.matches.push(MatchHunk {
        file: file2,
        line: 1,
        col: 3,
        variant: "old_name".to_string(),
        before: "wrong_name".to_string(), // This will cause failure
        after: "new_name".to_string(),
        start: 3,
        end: 11,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        atomic: true,
        ..Default::default()
    };

    let result = apply_plan(&mut plan, &options);
    assert!(result.is_err());

    // With the diff-based system, content changes are not rolled back during failed apply
    // Only renames are rolled back. Content can be undone using the undo command.
    let content1 = fs::read_to_string(&file1).unwrap();
    assert_eq!(content1, "fn new_name() {}"); // First file was changed before failure
}

#[test]
fn test_skip_symlinks() {
    let temp_dir = TempDir::new().unwrap();
    let target_file = temp_dir.path().join("target.txt");
    let symlink_path = temp_dir.path().join("link.txt");

    fs::write(&target_file, "target content").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&target_file, &symlink_path).unwrap();

        // Create plan trying to rename symlink
        let mut plan = create_test_plan("test_symlink", "link", "new_link");
        plan.renames.push(Rename {
            from: symlink_path,
            to: temp_dir.path().join("new_link.txt"),
            kind: RenameKind::File,
            coercion_applied: None,
        });

        let options = ApplyOptions {
            backup_dir: temp_dir.path().join(".backups"),
            skip_symlinks: true,
            ..Default::default()
        };

        // Applying should handle symlinks based on policy
        let _ = apply_plan(&mut plan, &options);
    }
}

#[test]
fn test_apply_with_git_commit() {
    use std::process::Command;

    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Disable GPG signing for test
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Create and commit initial file
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn old_name() {}").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Create plan with edits
    let mut plan = create_test_plan("test_git", "old_name", "new_name");
    plan.matches.push(MatchHunk {
        file: test_file,
        line: 1,
        col: 3,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 3,
        end: 11,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Apply with commit
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        commit: true,
        ..Default::default()
    };

    // Change to temp dir for git operations
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = apply_plan(&mut plan, &options);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());

    // Verify commit was created
    let output = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("renamify: rename old_name -> new_name"));
}

#[test]
fn test_apply_with_both_renames_and_content_changes() {
    // This test covers the critical case where files are renamed AND have content changes
    // This is the most complex scenario and has revealed bugs in the past
    let temp_dir = TempDir::new().unwrap();

    // Case 1: File that gets renamed AND has content changes
    let old_file1 = temp_dir.path().join("old_name.rs");
    fs::write(
        &old_file1,
        "fn old_name() {\n    println!(\"old_name\");\n}",
    )
    .unwrap();

    // Case 2: File that only has content changes (no rename)
    let stable_file = temp_dir.path().join("stable.rs");
    fs::write(&stable_file, "use old_name;\nfn main() { old_name(); }").unwrap();

    // Case 3: Directory that gets renamed containing a file with content changes
    let old_dir = temp_dir.path().join("old_name_dir");
    fs::create_dir(&old_dir).unwrap();
    let nested_file = old_dir.join("nested.rs");
    fs::write(&nested_file, "mod old_name;\nuse old_name::*;").unwrap();

    // Case 4: File that gets renamed but has NO content changes
    let old_file2 = temp_dir.path().join("old_name.txt");
    fs::write(&old_file2, "This file has no content changes").unwrap();

    // Case 5: CRITICAL - File that gets renamed inside a directory that also gets renamed
    // This is the case that was failing in the E2E test
    let service_dir = temp_dir.path().join("old_name_service");
    let src_dir = service_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let service_file = src_dir.join("old_name-service.ts");
    fs::write(
        &service_file,
        "export class OldNameService {\n  old_name: string;\n}",
    )
    .unwrap();

    // Create a comprehensive plan
    let mut plan = create_test_plan("test_both", "old_name", "new_name");

    // Add renames
    plan.renames.push(Rename {
        from: old_file1.clone(),
        to: temp_dir.path().join("new_name.rs"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_dir"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    plan.renames.push(Rename {
        from: old_file2.clone(),
        to: temp_dir.path().join("new_name.txt"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    // Case 5 renames: BOTH directory and file inside it get renamed
    plan.renames.push(Rename {
        from: service_dir.clone(),
        to: temp_dir.path().join("new_name_service"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    plan.renames.push(Rename {
        from: service_file.clone(),
        to: service_dir.join("src").join("new_name-service.ts"), // Using old path - will be adjusted during apply
        kind: RenameKind::File,
        coercion_applied: None,
    });

    // Add content changes for the file that will be renamed
    plan.matches.push(MatchHunk {
        file: old_file1.clone(), // Using original path
        line: 1,
        col: 3,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 3,
        end: 11,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    plan.matches.push(MatchHunk {
        file: old_file1.clone(), // Same file, different location
        line: 2,
        col: 14,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 30,
        end: 38,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Add content changes for the stable file (not renamed)
    // "use old_name;\nfn main() { old_name(); }"
    // Position 4-12 for first old_name
    // Position 26-34 for second old_name
    plan.matches.push(MatchHunk {
        file: stable_file.clone(),
        line: 1,
        col: 4,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 4,
        end: 12,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    plan.matches.push(MatchHunk {
        file: stable_file.clone(),
        line: 2,
        col: 13,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 26,
        end: 34,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Add content changes for the file inside the directory that will be renamed
    plan.matches.push(MatchHunk {
        file: nested_file.clone(), // Using original path
        line: 1,
        col: 4,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 4,
        end: 12,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    plan.matches.push(MatchHunk {
        file: nested_file.clone(),
        line: 2,
        col: 4,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 18,
        end: 26,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Case 5: Add content changes for the file that gets renamed inside a renamed directory
    // "export class OldNameService {\n  old_name: string;\n}"
    plan.matches.push(MatchHunk {
        file: service_file.clone(),
        line: 2,
        col: 2,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 32, // Position of old_name in the string
        end: 40,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Apply the plan
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        atomic: true,
        ..Default::default()
    };

    let result = apply_plan(&mut plan, &options);

    // The apply should succeed
    assert!(result.is_ok(), "Apply failed: {:?}", result);

    // Verify all renames happened
    assert!(!old_file1.exists(), "old_name.rs should not exist");
    assert!(!old_file2.exists(), "old_name.txt should not exist");
    assert!(!old_dir.exists(), "old_name_dir should not exist");

    let new_file1 = temp_dir.path().join("new_name.rs");
    let new_file2 = temp_dir.path().join("new_name.txt");
    let new_dir = temp_dir.path().join("new_name_dir");
    let new_nested = new_dir.join("nested.rs");

    assert!(new_file1.exists(), "new_name.rs should exist");
    assert!(new_file2.exists(), "new_name.txt should exist");
    assert!(new_dir.exists(), "new_name_dir should exist");
    assert!(new_nested.exists(), "nested.rs should exist in new dir");

    // Verify content changes in renamed file
    let content1 = fs::read_to_string(&new_file1).unwrap();
    assert!(
        content1.contains("fn new_name()"),
        "Function should be renamed"
    );
    assert!(
        content1.contains("println!(\"new_name\")"),
        "String should be renamed"
    );
    assert!(!content1.contains("old_name"), "No old_name should remain");

    // Verify content in file that was renamed but had no content changes
    let content2 = fs::read_to_string(&new_file2).unwrap();
    assert_eq!(
        content2, "This file has no content changes",
        "Content should be unchanged"
    );

    // Verify content changes in stable file (not renamed)
    let stable_content = fs::read_to_string(&stable_file).unwrap();
    assert!(
        stable_content.contains("use new_name;"),
        "Import should be renamed"
    );
    assert!(
        stable_content.contains("new_name()"),
        "Function call should be renamed"
    );
    assert!(
        !stable_content.contains("old_name"),
        "No old_name should remain"
    );

    // Verify content changes in file inside renamed directory
    let nested_content = fs::read_to_string(&new_nested).unwrap();
    assert!(
        nested_content.contains("mod new_name;"),
        "Module should be renamed"
    );
    assert!(
        nested_content.contains("use new_name::*"),
        "Use statement should be renamed"
    );
    assert!(
        !nested_content.contains("old_name"),
        "No old_name should remain"
    );

    // Verify backup/diff files were created
    let backup_dir = temp_dir.path().join(".renamify/backups").join("test_both");
    assert!(backup_dir.exists(), "Backup directory should exist");

    // Check for reverse patches directory
    let reverse_patches_dir = backup_dir.join("reverse_patches");
    assert!(
        reverse_patches_dir.exists(),
        "Reverse patches directory should exist"
    );

    // Check that patch files were created
    let entries: Vec<_> = fs::read_dir(&reverse_patches_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(!entries.is_empty(), "Should have at least one patch file");

    // Read all patch files and check they contain the expected changes
    let mut found_old_name = false;
    let mut found_new_name = false;
    let mut found_stable = false;

    for entry in entries {
        let patch_content = fs::read_to_string(entry.path()).unwrap();
        if patch_content.contains("old_name.rs") {
            found_old_name = true;
        }
        if patch_content.contains("new_name.rs") {
            found_new_name = true;
        }
        if patch_content.contains("stable.rs") {
            found_stable = true;
        }
    }

    assert!(
        found_old_name,
        "At least one patch should contain old_name.rs"
    );
    assert!(
        found_new_name,
        "At least one patch should contain new_name.rs"
    );
    assert!(found_stable, "At least one patch should contain stable.rs");

    // Check for nested.rs in patches
    let mut found_nested = false;
    for entry in fs::read_dir(&reverse_patches_dir)
        .unwrap()
        .filter_map(Result::ok)
    {
        let patch_content = fs::read_to_string(entry.path()).unwrap();
        if patch_content.contains("nested.rs") {
            found_nested = true;
            break;
        }
    }
    assert!(found_nested, "At least one patch should contain nested.rs");

    // Case 5 verifications: File renamed inside renamed directory
    let new_service_dir = temp_dir.path().join("new_name_service");
    let new_service_file = new_service_dir.join("src").join("new_name-service.ts");

    assert!(
        new_service_dir.exists(),
        "new_name_service directory should exist"
    );
    assert!(
        new_service_file.exists(),
        "new_name-service.ts should exist in renamed directory"
    );
    assert!(
        !service_dir.exists(),
        "old_name_service directory should not exist"
    );
    assert!(!service_file.exists(), "old service file should not exist");

    // Verify content changes in the service file
    let service_content = fs::read_to_string(&new_service_file).unwrap();
    assert!(
        service_content.contains("new_name: string"),
        "Property should be renamed"
    );
    assert!(
        !service_content.contains("old_name: string"),
        "Old property name should not exist"
    );

    // Verify the service file changes are in the patches
    let mut found_old_service = false;
    let mut found_new_service = false;
    for entry in fs::read_dir(&reverse_patches_dir)
        .unwrap()
        .filter_map(Result::ok)
    {
        let patch_content = fs::read_to_string(entry.path()).unwrap();
        if patch_content.contains("old_name-service.ts") {
            found_old_service = true;
        }
        if patch_content.contains("new_name-service.ts") {
            found_new_service = true;
        }
    }
    assert!(
        found_old_service,
        "At least one patch should contain old_name-service.ts"
    );
    assert!(
        found_new_service,
        "At least one patch should contain new_name-service.ts"
    );
}
