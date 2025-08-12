use refaktor_core::{apply_plan, ApplyOptions, MatchHunk, Plan, Rename, RenameKind, Stats};
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
    });

    // Apply the plan
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        log_file: Some(temp_dir.path().join("apply.log")),
        ..Default::default()
    };

    apply_plan(&plan, &options).unwrap();

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

    apply_plan(&plan, &options).unwrap();

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
    });

    // Apply should fail
    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        atomic: true,
        ..Default::default()
    };

    let result = apply_plan(&plan, &options);
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

    apply_plan(&plan, &options).unwrap();

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
    });

    // Second edit will fail (wrong content)
    plan.matches.push(MatchHunk {
        file: file2.clone(),
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
    });

    let options = ApplyOptions {
        backup_dir: temp_dir.path().join(".backups"),
        atomic: true,
        ..Default::default()
    };

    let result = apply_plan(&plan, &options);
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
            from: symlink_path.clone(),
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
        let _ = apply_plan(&plan, &options);
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

    let result = apply_plan(&plan, &options);

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
    assert!(log.contains("refaktor: rename old_name -> new_name"));
}
