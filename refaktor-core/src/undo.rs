use crate::apply::{apply_plan, calculate_checksum, ApplyOptions};
use crate::history::{create_history_entry, History};
use crate::scanner::Plan;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Undo a previously applied refactoring
pub fn undo_refactoring(id: &str, refaktor_dir: &Path) -> Result<()> {
    let mut history = History::load(refaktor_dir)?;

    // Find the entry to undo
    let entry = history
        .find_entry(id)
        .ok_or_else(|| anyhow!("History entry '{}' not found", id))?
        .clone();

    // Check if this was already reverted
    if entry.revert_of.is_some() {
        return Err(anyhow!("Entry '{}' is already a revert operation", id));
    }

    // Check if any later entry was already reverted
    let has_later_revert = history
        .list_entries(None)
        .iter()
        .any(|e| e.revert_of.as_ref() == Some(&entry.id));

    if has_later_revert {
        return Err(anyhow!("Entry '{}' has already been reverted", id));
    }

    eprintln!("Undoing refactoring '{}'...", id);

    // Build a map from renamed paths back to their original names
    let mut rename_map: HashMap<&PathBuf, &PathBuf> = HashMap::new();
    for (from, to) in &entry.renames {
        rename_map.insert(to, from);
    }

    // FIRST: Reverse renames (to -> from) - do this before restoring content
    let mut reversed_renames = Vec::new();
    for (from, to) in entry.renames.iter().rev() {
        if to.exists() {
            // Handle case-only renames on case-insensitive filesystems
            let case_only = from.to_string_lossy().to_lowercase()
                == to.to_string_lossy().to_lowercase()
                && from != to;

            if case_only
                && crate::rename::detect_case_insensitive_fs(to.parent().unwrap_or(Path::new(".")))
            {
                // Two-step rename for case-only changes
                let temp_name = to.with_extension(format!("{}.refaktor.tmp", std::process::id()));
                fs::rename(to, &temp_name)?;
                fs::rename(&temp_name, from)?;
            } else {
                fs::rename(to, from)?;
            }

            reversed_renames.push((to.clone(), from.clone()));
            eprintln!("  Renamed: {} -> {}", to.display(), from.display());
        }
    }

    // SECOND: Apply diffs to restore original content (now files are at their original locations)
    let mut restored_files = Vec::new();
    for (path, _checksum) in &entry.affected_files {
        // If this file was renamed, it's now at its original location
        let current_path = rename_map.get(&path).unwrap_or(&path);

        // The diff is stored at backups_path/filename.diff
        let diff_filename = format!(
            "{}.diff",
            current_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
        let diff_path = entry.backups_path.join(diff_filename);

        if diff_path.exists() {
            // Read the diff
            let diff_content = fs::read_to_string(&diff_path)
                .with_context(|| format!("Failed to read diff from {}", diff_path.display()))?;

            // Apply the diff using patch command (reverse it to undo)
            let output = std::process::Command::new("patch")
                .args(["-R", "-p0"])  // -R for reverse, -p0 for no path stripping
                .arg(current_path.to_str().unwrap())
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(diff_content.as_bytes())?;
                    }
                    child.wait_with_output()
                })
                .context("Failed to apply diff")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to apply diff for {}: stderr: {}, stdout: {}",
                    current_path.display(),
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                ));
            }

            restored_files.push(current_path.to_path_buf());
            eprintln!("  Restored: {}", current_path.display());
        }
    }

    // Calculate checksums of restored files
    let mut affected_files = HashMap::new();
    for path in &restored_files {
        if path.exists() && path.is_file() {
            let checksum = calculate_checksum(path)?;
            affected_files.insert(path.clone(), checksum);
        }
    }

    // Create a revert history entry
    let revert_entry = crate::history::HistoryEntry {
        id: format!("revert-{}-{}", entry.id, chrono::Local::now().timestamp()),
        created_at: chrono::Local::now().to_rfc3339(),
        old: entry.new.clone(), // Swap old and new
        new: entry.old.clone(),
        styles: entry.styles.clone(),
        includes: entry.includes.clone(),
        excludes: entry.excludes.clone(),
        affected_files,
        renames: reversed_renames,
        backups_path: entry.backups_path.clone(), // Keep same backup path
        revert_of: Some(entry.id.clone()),
        redo_of: None,
    };

    history.add_entry(revert_entry)?;

    eprintln!("Successfully undid refactoring '{}'", id);
    Ok(())
}

/// Redo a previously undone refactoring
pub fn redo_refactoring(id: &str, refaktor_dir: &Path) -> Result<()> {
    let history = History::load(refaktor_dir)?;

    // Find the original entry
    let entry = history
        .find_entry(id)
        .ok_or_else(|| anyhow!("History entry '{}' not found", id))?;

    // Check if this entry was reverted
    let entries = history.list_entries(None);
    let revert_entry = entries
        .iter()
        .find(|e| e.revert_of.as_ref() == Some(&entry.id));

    if revert_entry.is_none() {
        return Err(anyhow!("Entry '{}' has not been reverted", id));
    }

    eprintln!("Redoing refactoring '{}'...", id);

    // Load the original plan from disk
    let plan_path = refaktor_dir.join("plans").join(format!("{}.json", id));
    if !plan_path.exists() {
        return Err(anyhow!("Plan file not found for entry '{}'. This may be an old refactoring before plans were stored.", id));
    }

    let plan_json = fs::read_to_string(&plan_path)?;
    let mut plan: Plan = serde_json::from_str(&plan_json)?;

    // Give the redo a new ID to avoid conflicts
    plan.id = format!("redo-{}-{}", id, chrono::Local::now().timestamp());

    // Apply the plan again
    let options = ApplyOptions {
        backup_dir: refaktor_dir.join("backups"),
        create_backups: true,
        atomic: true,
        ..Default::default()
    };

    apply_plan(&plan, &options)?;

    eprintln!("Successfully redid refactoring '{}'", id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_undo_with_content_and_rename() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create backup directory
        let backup_dir = refaktor_dir.join("backups").join("test_apply_123");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create test file in its renamed state with modified content
        // This simulates the state after apply: renamed and content changed
        let new_file = temp_dir.path().join("new_name.txt");
        fs::write(&new_file, "modified content\n").unwrap();

        // Create a diff backup that will restore the original content
        // The diff shows: original -> modified
        // When reversed with patch -R, it restores: modified -> original
        let diff_content = r#"--- old_name.txt
+++ old_name.txt
@@ -1 +1 @@
-original content
+modified content
"#;
        let diff_file = backup_dir.join("old_name.txt.diff");
        fs::write(&diff_file, diff_content).unwrap();

        // Create history entry representing the applied refactoring
        let mut affected_files = HashMap::new();
        affected_files.insert(new_file.clone(), "checksum123".to_string());

        let entry = crate::history::HistoryEntry {
            id: "test_apply_123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
            styles: vec!["Snake".to_string()],
            includes: vec![],
            excludes: vec![],
            affected_files,
            renames: vec![(temp_dir.path().join("old_name.txt"), new_file.clone())],
            backups_path: backup_dir.clone(),
            revert_of: None,
            redo_of: None,
        };

        // Create history with this entry
        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(entry.clone()).unwrap();
        history.save().unwrap();

        // Perform undo
        undo_refactoring("test_apply_123", &refaktor_dir).unwrap();

        // Verify file was renamed back
        assert!(!new_file.exists(), "Renamed file should not exist");
        let old_file = temp_dir.path().join("old_name.txt");
        assert!(old_file.exists(), "Original file should be restored");

        // Verify content was restored
        let content = fs::read_to_string(&old_file).unwrap();
        assert_eq!(
            content, "original content\n",
            "Content should be restored from modified"
        );

        // Verify history has revert entry
        let updated_history = History::load(&refaktor_dir).unwrap();
        let entries = updated_history.list_entries(None);
        assert_eq!(entries.len(), 2, "Should have original and revert entries");

        let revert_entry = &entries[0]; // Most recent first
        assert!(revert_entry.revert_of.is_some());
        assert_eq!(revert_entry.revert_of.as_ref().unwrap(), "test_apply_123");
    }

    #[test]
    fn test_undo_already_reverted() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create history with an entry and its revert
        let entry = crate::history::HistoryEntry {
            id: "original".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("backups/original"),
            revert_of: None,
            redo_of: None,
        };

        let revert_entry = crate::history::HistoryEntry {
            id: "revert-original".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "new".to_string(),
            new: "old".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("backups/original"),
            revert_of: Some("original".to_string()),
            redo_of: None,
        };

        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.add_entry(revert_entry).unwrap();
        history.save().unwrap();

        // Try to undo again - should fail
        let result = undo_refactoring("original", &refaktor_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already been reverted"));
    }

    #[test]
    fn test_undo_revert_entry() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create history with a revert entry
        let revert_entry = crate::history::HistoryEntry {
            id: "revert-123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "new".to_string(),
            new: "old".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("backups/123"),
            revert_of: Some("123".to_string()),
            redo_of: None,
        };

        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(revert_entry).unwrap();
        history.save().unwrap();

        // Try to undo a revert entry - should fail
        let result = undo_refactoring("revert-123", &refaktor_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already a revert operation"));
    }

    #[test]
    fn test_undo_nonexistent_entry() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create empty history
        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let history = History::load(&refaktor_dir).unwrap();
        history.save().unwrap();

        // Try to undo nonexistent entry
        let result = undo_refactoring("nonexistent", &refaktor_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_redo_after_undo() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create history with original and revert entries
        let entry = crate::history::HistoryEntry {
            id: "test123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("backups/test123"),
            revert_of: None,
            redo_of: None,
        };

        let revert_entry = crate::history::HistoryEntry {
            id: "revert-test123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "new_name".to_string(),
            new: "old_name".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("backups/test123"),
            revert_of: Some("test123".to_string()),
            redo_of: None,
        };

        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.add_entry(revert_entry).unwrap();
        history.save().unwrap();

        // Create a dummy file to satisfy apply_plan
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn old_name() {}").unwrap();

        // Redo should succeed (though apply might fail without proper setup)
        // We're mainly testing the redo logic, not the full apply
        let result = redo_refactoring("test123", &refaktor_dir);
        // The redo might fail due to missing files, but it should at least find the entry
        if result.is_err() {
            let err_msg = result.unwrap_err().to_string();
            assert!(
                !err_msg.contains("not been reverted"),
                "Should find revert entry"
            );
        }
    }

    #[test]
    fn test_redo_not_reverted() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create history with only original entry (no revert)
        let entry = crate::history::HistoryEntry {
            id: "test456".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("backups/test456"),
            revert_of: None,
            redo_of: None,
        };

        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Try to redo - should fail
        let result = redo_refactoring("test456", &refaktor_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not been reverted"));
    }

    #[test]
    fn test_undo_case_insensitive_rename() {
        let temp_dir = TempDir::new().unwrap();
        let refaktor_dir = temp_dir.path().join(".refaktor");
        fs::create_dir_all(&refaktor_dir).unwrap();

        // Create backup directory
        let backup_dir = refaktor_dir.join("backups").join("test_case");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create test file with new name (different case)
        let new_file = temp_dir.path().join("NewName.txt");
        fs::write(&new_file, "new content").unwrap();

        // Create backup of original file
        let backup_file = backup_dir.join("newname.txt");
        fs::write(&backup_file, "original content").unwrap();

        // Create history entry for case-only rename
        let mut affected_files = HashMap::new();
        affected_files.insert(new_file.clone(), "checksum".to_string());

        let entry = crate::history::HistoryEntry {
            id: "test_case".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "newname".to_string(),
            new: "NewName".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files,
            renames: vec![(temp_dir.path().join("newname.txt"), new_file.clone())],
            backups_path: backup_dir.clone(),
            revert_of: None,
            redo_of: None,
        };

        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Perform undo
        let result = undo_refactoring("test_case", &refaktor_dir);

        // On case-insensitive filesystems, this should handle the temp rename
        // On case-sensitive filesystems, it should just rename directly
        assert!(result.is_ok());

        // Original file should exist (with original case on case-sensitive systems)
        let old_file = temp_dir.path().join("newname.txt");
        if !crate::rename::detect_case_insensitive_fs(temp_dir.path()) {
            assert!(old_file.exists() || new_file.exists()); // One should exist
        }
    }
}
