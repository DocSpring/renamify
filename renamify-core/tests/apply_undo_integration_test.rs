use renamify_core::{
    apply_plan, undo_renaming, ApplyOptions, MatchHunk, Plan, Rename, RenameKind, Stats,
};
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
fn test_apply_undo_content_changes_only() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Write initial content
    let initial_content = "fn old_name() {\n    println!(\"old_name\");\n}";
    fs::write(&test_file, initial_content).unwrap();

    // Create plan with content edits only
    let mut plan = create_test_plan("content_only", "old_name", "new_name");
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
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify changes were applied
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("fn new_name()"));
    assert!(content.contains("println!(\"new_name\")"));
    assert!(!content.contains("old_name"));

    // Undo the changes
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify undo worked
    let undone_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(undone_content, initial_content);
}

#[test]
fn test_apply_undo_file_rename_only() {
    let temp_dir = TempDir::new().unwrap();
    let old_file = temp_dir.path().join("old_name.txt");

    // Write initial content
    let content = "This content does not change";
    fs::write(&old_file, content).unwrap();

    // Create plan with file rename only
    let mut plan = create_test_plan("file_rename_only", "old_name", "new_name");
    plan.renames.push(Rename {
        from: old_file.clone(),
        to: temp_dir.path().join("new_name.txt"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify rename happened
    assert!(!old_file.exists());
    let new_file = temp_dir.path().join("new_name.txt");
    assert!(new_file.exists());
    assert_eq!(fs::read_to_string(&new_file).unwrap(), content);

    // Undo the rename
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify undo worked
    assert!(old_file.exists());
    assert!(!new_file.exists());
    assert_eq!(fs::read_to_string(&old_file).unwrap(), content);
}

#[test]
fn test_apply_undo_dir_rename_only() {
    let temp_dir = TempDir::new().unwrap();
    let old_dir = temp_dir.path().join("old_name_dir");
    fs::create_dir(&old_dir).unwrap();

    // Create a file inside the directory
    let inner_file = old_dir.join("file.txt");
    let content = "Content inside directory";
    fs::write(&inner_file, content).unwrap();

    // Create plan with directory rename only
    let mut plan = create_test_plan("dir_rename_only", "old_name", "new_name");
    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_dir"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify rename happened
    assert!(!old_dir.exists());
    let new_dir = temp_dir.path().join("new_name_dir");
    assert!(new_dir.exists());
    assert!(new_dir.join("file.txt").exists());
    assert_eq!(
        fs::read_to_string(new_dir.join("file.txt")).unwrap(),
        content
    );

    // Undo the rename
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify undo worked
    assert!(old_dir.exists());
    assert!(!new_dir.exists());
    assert!(inner_file.exists());
    assert_eq!(fs::read_to_string(&inner_file).unwrap(), content);
}

#[test]
fn test_apply_undo_content_and_file_rename() {
    let temp_dir = TempDir::new().unwrap();
    let old_file = temp_dir.path().join("old_name.rs");

    // Write initial content
    let initial_content = "fn old_name() {\n    println!(\"old_name\");\n}";
    fs::write(&old_file, initial_content).unwrap();

    // Create plan with content changes AND file rename
    let mut plan = create_test_plan("content_and_file", "old_name", "new_name");

    // File rename
    plan.renames.push(Rename {
        from: old_file.clone(),
        to: temp_dir.path().join("new_name.rs"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    // Content changes
    plan.matches.push(MatchHunk {
        file: old_file.clone(),
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
        file: old_file.clone(),
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
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify changes
    assert!(!old_file.exists());
    let new_file = temp_dir.path().join("new_name.rs");
    assert!(new_file.exists());
    let content = fs::read_to_string(&new_file).unwrap();
    assert!(content.contains("fn new_name()"));
    assert!(content.contains("println!(\"new_name\")"));

    // Undo the changes
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify undo worked
    assert!(old_file.exists());
    assert!(!new_file.exists());
    let undone_content = fs::read_to_string(&old_file).unwrap();
    assert_eq!(undone_content, initial_content);
}

#[test]
fn test_apply_undo_content_and_dir_rename() {
    let temp_dir = TempDir::new().unwrap();
    let old_dir = temp_dir.path().join("old_name_dir");
    fs::create_dir(&old_dir).unwrap();

    let test_file = old_dir.join("test.rs");
    let initial_content = "fn old_name() {\n    println!(\"old_name\");\n}";
    fs::write(&test_file, initial_content).unwrap();

    // Create plan with content changes AND directory rename
    let mut plan = create_test_plan("content_and_dir", "old_name", "new_name");

    // Directory rename
    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_dir"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    // Content changes
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
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify changes
    assert!(!old_dir.exists());
    let new_dir = temp_dir.path().join("new_name_dir");
    assert!(new_dir.exists());
    let new_file = new_dir.join("test.rs");
    assert!(new_file.exists());
    let content = fs::read_to_string(&new_file).unwrap();
    assert!(content.contains("fn new_name()"));
    assert!(content.contains("println!(\"new_name\")"));

    // Undo the changes
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify undo worked
    assert!(old_dir.exists());
    assert!(!new_dir.exists());
    assert!(test_file.exists());
    let undone_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(undone_content, initial_content);
}

#[test]
fn test_apply_undo_file_and_dir_rename() {
    let temp_dir = TempDir::new().unwrap();
    let old_dir = temp_dir.path().join("old_name_dir");
    fs::create_dir(&old_dir).unwrap();

    let old_file = old_dir.join("old_name.txt");
    let content = "Content that doesn't change";
    fs::write(&old_file, content).unwrap();

    // Create plan with file rename AND directory rename
    let mut plan = create_test_plan("file_and_dir", "old_name", "new_name");

    // Directory rename
    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_dir"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    // File rename - should use the new directory path
    plan.renames.push(Rename {
        from: old_file.clone(),
        to: temp_dir.path().join("new_name_dir").join("new_name.txt"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify changes
    assert!(!old_dir.exists());
    let new_dir = temp_dir.path().join("new_name_dir");
    assert!(new_dir.exists());
    let new_file = new_dir.join("new_name.txt");
    assert!(new_file.exists());
    assert_eq!(fs::read_to_string(&new_file).unwrap(), content);

    // Undo the changes
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify undo worked
    assert!(old_dir.exists());
    assert!(!new_dir.exists());
    assert!(old_file.exists());
    assert_eq!(fs::read_to_string(&old_file).unwrap(), content);
}

#[test]
fn test_apply_undo_all_changes() {
    let temp_dir = TempDir::new().unwrap();
    let old_dir = temp_dir.path().join("old_name_dir");
    fs::create_dir(&old_dir).unwrap();

    let old_file = old_dir.join("old_name.rs");
    let initial_content = "fn old_name() {\n    println!(\"old_name\");\n}";
    fs::write(&old_file, initial_content).unwrap();

    // Another file with just content changes
    let stable_file = temp_dir.path().join("stable.rs");
    let stable_initial = "use old_name;\nfn main() { old_name(); }";
    fs::write(&stable_file, stable_initial).unwrap();

    // Create plan with ALL types of changes
    let mut plan = create_test_plan("all_changes", "old_name", "new_name");

    // Directory rename
    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_dir"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    // File rename - should use the new directory path
    plan.renames.push(Rename {
        from: old_file.clone(),
        to: temp_dir.path().join("new_name_dir").join("new_name.rs"),
        kind: RenameKind::File,
        coercion_applied: None,
    });

    // Content changes in renamed file
    plan.matches.push(MatchHunk {
        file: old_file.clone(),
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
        file: old_file.clone(),
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

    // Content changes in stable file
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

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify all changes
    assert!(!old_dir.exists());
    let new_dir = temp_dir.path().join("new_name_dir");
    assert!(new_dir.exists());
    let new_file = new_dir.join("new_name.rs");
    assert!(new_file.exists());

    let renamed_content = fs::read_to_string(&new_file).unwrap();
    assert!(renamed_content.contains("fn new_name()"));
    assert!(renamed_content.contains("println!(\"new_name\")"));

    let stable_content = fs::read_to_string(&stable_file).unwrap();
    assert!(stable_content.contains("use new_name;"));
    assert!(stable_content.contains("new_name()"));

    // Undo ALL the changes
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify complete undo
    assert!(old_dir.exists());
    assert!(!new_dir.exists());
    assert!(old_file.exists());

    let undone_content = fs::read_to_string(&old_file).unwrap();
    assert_eq!(undone_content, initial_content);

    let undone_stable = fs::read_to_string(&stable_file).unwrap();
    assert_eq!(undone_stable, stable_initial);
}

#[test]
fn test_apply_undo_multiple_files_in_renamed_dir() {
    let temp_dir = TempDir::new().unwrap();
    let old_dir = temp_dir.path().join("old_name_lib");
    fs::create_dir(&old_dir).unwrap();

    // Create multiple files with content changes
    let file1 = old_dir.join("file1.rs");
    let file1_initial = "struct OldName { old_name: String }";
    fs::write(&file1, file1_initial).unwrap();

    let file2 = old_dir.join("file2.rs");
    let file2_initial = "fn old_name() -> OldName { OldName::new() }";
    fs::write(&file2, file2_initial).unwrap();

    // Create plan
    let mut plan = create_test_plan("multi_files", "old_name", "new_name");

    // Directory rename
    plan.renames.push(Rename {
        from: old_dir.clone(),
        to: temp_dir.path().join("new_name_lib"),
        kind: RenameKind::Dir,
        coercion_applied: None,
    });

    // Content changes in file1
    plan.matches.push(MatchHunk {
        file: file1.clone(),
        line: 1,
        col: 18,
        variant: "old_name".to_string(),
        before: "old_name".to_string(),
        after: "new_name".to_string(),
        start: 17, // Position of "old_name" in "struct OldName { old_name: String }"
        end: 25,
        line_before: None,
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    // Content changes in file2
    plan.matches.push(MatchHunk {
        file: file2.clone(),
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

    // Apply
    let apply_options = ApplyOptions {
        backup_dir: temp_dir.path().join(".renamify/backups"),
        create_backups: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    // Verify
    let new_dir = temp_dir.path().join("new_name_lib");
    assert!(new_dir.exists());
    assert!(!old_dir.exists());

    // Undo
    let renamify_dir = temp_dir.path().join(".renamify");
    undo_renaming(&plan.id, &renamify_dir).unwrap();

    // Verify complete restoration
    assert!(old_dir.exists());
    assert!(!new_dir.exists());
    assert_eq!(fs::read_to_string(&file1).unwrap(), file1_initial);
    assert_eq!(fs::read_to_string(&file2).unwrap(), file2_initial);
}
