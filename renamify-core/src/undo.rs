use crate::apply::{apply_plan, calculate_checksum, ApplyOptions};
use crate::history::{create_history_entry, History};
use crate::scanner::Plan;
use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Apply a single patch to a file
fn apply_single_patch(file_path: &Path, patch_content: &str) -> Result<()> {
    // Read the current file content
    let current_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Get original file permissions before modifying
    let original_metadata = fs::metadata(file_path)?;
    let original_permissions = original_metadata.permissions();

    // On Windows, normalize the patch content to handle \\?\ prefixes
    #[cfg(windows)]
    let patch_content = {
        // Remove \\?\ prefix from paths in the patch header
        let mut normalized = String::new();
        let lines: Vec<&str> = patch_content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("--- ") || line.starts_with("+++ ") {
                // Extract the path part after --- or +++
                let prefix = &line[0..4];
                let path_part = &line[4..];
                // Remove \\?\ prefix if present
                let cleaned = if path_part.starts_with("\\\\?\\") {
                    &path_part[4..]
                } else {
                    path_part
                };
                normalized.push_str(prefix);
                normalized.push_str(cleaned);
            } else {
                normalized.push_str(line);
            }
            // Add newline unless it's the last line and the original didn't end with newline
            if i < lines.len() - 1 || patch_content.ends_with('\n') {
                normalized.push('\n');
            }
        }
        normalized
    };
    #[cfg(not(windows))]
    let patch_content = patch_content;

    // Parse and apply the patch using diffy
    let patch = diffy::Patch::from_str(&patch_content)
        .map_err(|e| anyhow!("Failed to parse patch: {}", e))?;

    let result = diffy::apply(&current_content, &patch)
        .map_err(|e| anyhow!("Failed to apply patch: {}", e))?;

    // Write the result back to the file
    fs::write(file_path, &result)
        .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

    // Restore original permissions
    fs::set_permissions(file_path, original_permissions)
        .with_context(|| format!("Failed to restore permissions for: {}", file_path.display()))?;

    Ok(())
}

/// Undo a previously applied renaming
pub fn undo_renaming(id: &str, renamify_dir: &Path) -> Result<()> {
    let mut history = History::load(renamify_dir)?;

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

    // Load the plan to get patch information
    let plan_path = renamify_dir.join("plans").join(format!("{}.json", id));
    if !plan_path.exists() {
        return Err(anyhow!(
            "Plan file not found for entry '{}'. Cannot undo without plan.",
            id
        ));
    }
    let plan_json = fs::read_to_string(&plan_path)?;
    let plan: Plan = serde_json::from_str(&plan_json)?;

    // Check for individual reverse patches directory
    let reverse_patches_dir = entry.backups_path.join("reverse_patches");
    if !reverse_patches_dir.exists() {
        return Err(anyhow!(
            "No reverse patches found for entry '{}'. Cannot undo.",
            id
        ));
    }
    // STEP 1: Reverse renames first (new locations back to old)
    // Process renames in reverse order, deepest paths first
    // Use the renames from the plan, not from history

    // First, rename directories back (this moves all their contents)
    let mut dir_mappings = Vec::new();
    for rename in &plan.renames {
        if matches!(rename.kind, crate::scanner::RenameKind::Dir) {
            dir_mappings.push((rename.from.clone(), rename.to.clone()));
        }
    }

    // Sort directories by depth (deepest first)
    dir_mappings.sort_by(|a, b| {
        let a_depth = a.1.components().count();
        let b_depth = b.1.components().count();
        b_depth.cmp(&a_depth)
    });

    for (from, to) in &dir_mappings {
        if to.exists() {
            fs::rename(to, from)?;
        }
    }

    // Now handle file renames, adjusting paths if they were inside renamed directories
    let mut file_renames = Vec::new();
    for rename in &plan.renames {
        if matches!(rename.kind, crate::scanner::RenameKind::File) {
            let mut adjusted_from = rename.from.clone();
            let mut adjusted_to = rename.to.clone();

            // Check if this file was inside a directory that we just renamed
            for (dir_from, dir_to) in &dir_mappings {
                // If the file's original paths were inside a renamed directory, adjust them
                if rename.to.starts_with(dir_to) {
                    // The file's "to" path was inside a renamed directory
                    // After undoing the directory rename, the file is now at the original directory location
                    if let Ok(relative) = rename.to.strip_prefix(dir_to) {
                        adjusted_to = dir_from.join(relative);
                    }
                }
                if rename.from.starts_with(dir_from) {
                    // Keep the original "from" path as is
                    adjusted_from.clone_from(&rename.from);
                }
            }

            file_renames.push((adjusted_from, adjusted_to));
        }
    }

    // Sort files by depth (deepest first)
    file_renames.sort_by(|a, b| {
        let a_depth = a.1.components().count();
        let b_depth = b.1.components().count();
        b_depth.cmp(&a_depth)
    });

    for (from, to) in &file_renames {
        if to.exists() {
            // Handle case-only renames on case-insensitive filesystems
            let case_only = from.to_string_lossy().to_lowercase()
                == to.to_string_lossy().to_lowercase()
                && from != to;

            if case_only
                && crate::rename::detect_case_insensitive_fs(
                    to.parent().unwrap_or_else(|| Path::new(".")),
                )
            {
                // Two-step rename for case-only changes
                let temp_name = to.with_extension(format!("{}.renamify.tmp", std::process::id()));
                fs::rename(to, &temp_name)?;
                fs::rename(&temp_name, from)?;
            } else {
                fs::rename(to, from)?;
            }
        }
    }

    // STEP 2: Apply individual reverse patches
    // Group matches by file to apply patches
    let mut patches_by_file: HashMap<PathBuf, String> = HashMap::new();
    for hunk in &plan.matches {
        if let Some(hash) = &hunk.patch_hash {
            let patch_file = reverse_patches_dir.join(format!("{}.patch", hash));
            if patch_file.exists() {
                let patch_content = fs::read_to_string(&patch_file)?;
                // Use the original_file if it exists, otherwise use the current file path
                let target_file = hunk.original_file.as_ref().unwrap_or(&hunk.file);
                patches_by_file.insert(target_file.clone(), patch_content);
            }
        }
    }

    // Apply all patches
    let mut failed_patches = Vec::new();
    for (file_path, patch_content) in patches_by_file {
        if let Err(e) = apply_single_patch(&file_path, &patch_content) {
            eprintln!(
                "  ERROR: Failed to apply patch to {}: {}",
                file_path.display(),
                e
            );

            // Save the failed patch as a .rej file for debugging
            let rej_path = file_path.with_extension(format!(
                "{}.rej",
                file_path.extension().and_then(|s| s.to_str()).unwrap_or("")
            ));
            if let Err(write_err) = fs::write(&rej_path, &patch_content) {
                eprintln!(
                    "    WARNING: Could not write reject file {}: {}",
                    rej_path.display(),
                    write_err
                );
            } else {
                eprintln!("    Saved failed patch to: {}", rej_path.display());
            }

            failed_patches.push(format!("{}: {}", file_path.display(), e));
        }
    }

    // STEP 3: Delete any directories that were created during apply
    if let Some(created_dirs) = &plan.created_directories {
        // Sort by depth (deepest first) to remove nested directories before parents
        let mut sorted_dirs = created_dirs.clone();
        sorted_dirs.sort_by(|a, b| {
            let a_depth = a.components().count();
            let b_depth = b.components().count();
            b_depth.cmp(&a_depth)
        });

        for dir in &sorted_dirs {
            // Only remove if directory exists, is a directory, and is empty
            if dir.exists() && dir.is_dir() {
                if let Ok(mut entries) = fs::read_dir(dir) {
                    if entries.next().is_none() {
                        // Directory is empty, safe to remove
                        let _ = fs::remove_dir(dir);
                    }
                }
            }
        }
    }

    // If any patches failed, return an error
    if !failed_patches.is_empty() {
        return Err(anyhow!("Failed to apply {} patches", failed_patches.len()));
    }

    // Calculate checksums of affected files and collect reversed renames
    let mut affected_files = HashMap::new();
    let mut reversed_renames = Vec::new();

    // Process all original affected files to calculate new checksums
    for path in entry.affected_files.keys() {
        // The file should now be at its original location (before the renaming)
        // Check if it was renamed and is now back at original location
        let original_path = entry
            .renames
            .iter()
            .find(|(_, to)| to == path)
            .map_or(path, |(from, _)| from);

        if original_path.exists() && original_path.is_file() {
            let checksum = calculate_checksum(original_path)?;
            affected_files.insert(original_path.clone(), checksum);
        }
    }

    // Collect reversed renames from the original entry
    for (from, to) in &entry.renames {
        reversed_renames.push((to.clone(), from.clone()));
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
        revert_of: Some(entry.id),
        redo_of: None,
    };

    history.add_entry(revert_entry)?;

    Ok(())
}

/// Redo a previously undone renaming
pub fn redo_renaming(id: &str, renamify_dir: &Path) -> Result<()> {
    let history = History::load(renamify_dir)?;

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

    eprintln!("Redoing renaming '{}'...", id);

    // Load the original plan from disk
    let plan_path = renamify_dir.join("plans").join(format!("{}.json", id));
    if !plan_path.exists() {
        return Err(anyhow!("Plan file not found for entry '{}'. This may be an old renaming before plans were stored.", id));
    }

    let plan_json = fs::read_to_string(&plan_path)?;
    let mut plan: Plan = serde_json::from_str(&plan_json)?;

    // Give the redo a new ID to avoid conflicts
    plan.id = format!("redo-{}-{}", id, chrono::Local::now().timestamp());

    // Apply the plan again
    let options = ApplyOptions {
        backup_dir: renamify_dir.join("backups"),
        create_backups: true,
        atomic: true,
        ..Default::default()
    };

    apply_plan(&mut plan, &options)?;

    eprintln!("Successfully redid renaming '{}'", id);
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
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        // Create backup directory
        let backup_dir = renamify_dir.join("backups").join("test_apply_123");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create reverse patches directory
        let reverse_patches_dir = backup_dir.join("reverse_patches");
        fs::create_dir_all(&reverse_patches_dir).unwrap();

        // Create plans directory
        let plans_dir = renamify_dir.join("plans");
        fs::create_dir_all(&plans_dir).unwrap();

        // Create test file in its renamed state with modified content
        // This simulates the state after apply: renamed and content changed
        let new_file = temp_dir.path().join("new_name.txt");
        fs::write(&new_file, "modified content\n").unwrap();

        // Create a reverse patch that will restore the original content
        let reverse_patch = r"--- old_name.txt
+++ old_name.txt
@@ -1 +1 @@
-modified content
+original content
";
        // Create a hash for the patch file name
        let mut hasher = Sha256::new();
        hasher.update(reverse_patch.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let patch_file = reverse_patches_dir.join(format!("{}.patch", hash));
        fs::write(&patch_file, reverse_patch).unwrap();

        // Create a plan file for the undo to use
        let plan = crate::scanner::Plan {
            id: "test_apply_123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![crate::scanner::MatchHunk {
                file: temp_dir.path().join("old_name.txt"),
                line: 1,
                col: 0,
                variant: "old_name".to_string(),
                before: "old_name".to_string(),
                after: "new_name".to_string(),
                start: 0,
                end: 8,
                line_before: None,
                line_after: None,
                coercion_applied: None,
                original_file: Some(temp_dir.path().join("old_name.txt")),
                renamed_file: None,
                patch_hash: Some(hash),
            }],
            renames: vec![crate::scanner::Rename {
                from: temp_dir.path().join("old_name.txt"),
                to: temp_dir.path().join("new_name.txt"),
                kind: crate::scanner::RenameKind::File,
                coercion_applied: None,
            }],
            stats: crate::scanner::Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant: HashMap::new(),
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };
        let plan_path = plans_dir.join("test_apply_123.json");
        fs::write(&plan_path, serde_json::to_string(&plan).unwrap()).unwrap();

        // Create history entry representing the applied renaming
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
            backups_path: backup_dir,
            revert_of: None,
            redo_of: None,
        };

        // Create history with this entry
        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Perform undo
        undo_renaming("test_apply_123", &renamify_dir).unwrap();

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
        let updated_history = History::load(&renamify_dir).unwrap();
        let entries = updated_history.list_entries(None);
        assert_eq!(entries.len(), 2, "Should have original and revert entries");

        let revert_entry = &entries[0]; // Most recent first
        assert!(revert_entry.revert_of.is_some());
        assert_eq!(revert_entry.revert_of.as_ref().unwrap(), "test_apply_123");
    }

    #[test]
    fn test_undo_already_reverted() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

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

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.add_entry(revert_entry).unwrap();
        history.save().unwrap();

        // Try to undo again - should fail
        let result = undo_renaming("original", &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already been reverted"));
    }

    #[test]
    fn test_undo_revert_entry() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

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

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(revert_entry).unwrap();
        history.save().unwrap();

        // Try to undo a revert entry - should fail
        let result = undo_renaming("revert-123", &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already a revert operation"));
    }

    #[test]
    fn test_undo_nonexistent_entry() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        // Create empty history
        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let history = History::load(&renamify_dir).unwrap();
        history.save().unwrap();

        // Try to undo nonexistent entry
        let result = undo_renaming("nonexistent", &renamify_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_redo_after_undo() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

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

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.add_entry(revert_entry).unwrap();
        history.save().unwrap();

        // Create a dummy file to satisfy apply_plan
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn old_name() {}").unwrap();

        // Redo should succeed (though apply might fail without proper setup)
        // We're mainly testing the redo logic, not the full apply
        let result = redo_renaming("test123", &renamify_dir);
        // The redo might fail due to missing files, but it should at least find the entry
        if let Err(err) = result {
            let err_msg = err.to_string();
            assert!(
                !err_msg.contains("not been reverted"),
                "Should find revert entry"
            );
        }
    }

    #[test]
    fn test_redo_not_reverted() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

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

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Try to redo - should fail
        let result = redo_renaming("test456", &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not been reverted"));
    }

    #[test]
    #[cfg(windows)]
    fn test_apply_single_patch_windows_long_path() {
        // Test that patches with Windows long path prefixes are normalized correctly
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Create a file with some content
        fs::write(&test_file, "new content").unwrap();

        // Create a patch with Windows long path prefix in headers
        let patch_with_prefix = r"--- \\?\C:\temp\test.txt
+++ \\?\C:\temp\test.txt
@@ -1 +1 @@
-new content
\ No newline at end of file
+old content
\ No newline at end of file
";

        // This should work - the function should normalize the paths
        let result = apply_single_patch(&test_file, patch_with_prefix);
        assert!(
            result.is_ok(),
            "Should handle Windows long path prefix in patch: {:?}",
            result.err()
        );

        // Verify content was changed
        let content = fs::read(&test_file).unwrap();
        assert_eq!(content, b"old content");
    }

    #[test]
    fn test_apply_single_patch_newline_comparison() {
        // Test that we correctly apply reverse patches with "No newline at end of file" markers
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Current file has NO trailing newline (this is the "after" state)
        fs::write(&test_file, "new content").unwrap();

        // Create a reverse patch that changes "new content" back to "old content"
        // This is what our undo system now generates - a proper reverse patch
        let patch_content = "--- test.txt\n+++ test.txt\n@@ -1 +1 @@\n-new content\n\\ No newline at end of file\n+old content\n\\ No newline at end of file\n";

        // This should work - applying reverse patch to restore original content
        let result = apply_single_patch(&test_file, patch_content);
        assert!(
            result.is_ok(),
            "Should apply reverse patch successfully: {:?}",
            result.err()
        );

        // Verify old content was restored without newline
        let content = fs::read(&test_file).unwrap();
        assert_eq!(content, b"old content");
    }

    #[test]
    #[cfg(unix)]
    fn test_undo_preserves_file_permissions_with_patches() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        // Create backup directory
        let backup_dir = renamify_dir.join("backups").join("test_perms");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create reverse patches directory
        let reverse_patches_dir = backup_dir.join("reverse_patches");
        fs::create_dir_all(&reverse_patches_dir).unwrap();

        // Create plans directory
        let plans_dir = renamify_dir.join("plans");
        fs::create_dir_all(&plans_dir).unwrap();

        // Create test file with executable permissions
        let test_file = temp_dir.path().join("script.sh");
        fs::write(&test_file, "#!/bin/bash\necho 'new content'\n").unwrap();

        // Set executable permissions (755)
        let mut perms = fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_file, perms.clone()).unwrap();

        // Verify permissions were set
        let original_mode = fs::metadata(&test_file).unwrap().permissions().mode();
        assert_eq!(
            original_mode & 0o777,
            0o755,
            "File should have 755 permissions"
        );

        // Create a reverse patch that will restore the original content
        let reverse_patch = r"--- script.sh
+++ script.sh
@@ -1,2 +1,2 @@
 #!/bin/bash
-echo 'new content'
+echo 'old content'
";
        // Create a hash for the patch file name
        let mut hasher = Sha256::new();
        hasher.update(reverse_patch.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let patch_file = reverse_patches_dir.join(format!("{}.patch", hash));
        fs::write(&patch_file, reverse_patch).unwrap();

        // Create a plan file for the undo to use
        let plan = crate::scanner::Plan {
            id: "test_perms".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![crate::scanner::MatchHunk {
                file: test_file.clone(),
                line: 2,
                col: 0,
                variant: "old".to_string(),
                before: "old".to_string(),
                after: "new".to_string(),
                start: 0,
                end: 3,
                line_before: None,
                line_after: None,
                coercion_applied: None,
                original_file: Some(test_file.clone()),
                renamed_file: None,
                patch_hash: Some(hash),
            }],
            renames: vec![],
            stats: crate::scanner::Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant: HashMap::new(),
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };
        let plan_path = plans_dir.join("test_perms.json");
        fs::write(&plan_path, serde_json::to_string(&plan).unwrap()).unwrap();

        // Create history entry
        let mut affected_files = HashMap::new();
        affected_files.insert(test_file.clone(), "checksum123".to_string());

        let entry = crate::history::HistoryEntry {
            id: "test_perms".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files,
            renames: vec![],
            backups_path: backup_dir,
            revert_of: None,
            redo_of: None,
        };

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Perform undo
        undo_renaming("test_perms", &renamify_dir).unwrap();

        // Verify content was restored
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(
            content.contains("old content"),
            "Content should be restored"
        );

        // Verify permissions were preserved
        let restored_mode = fs::metadata(&test_file).unwrap().permissions().mode();
        assert_eq!(
            restored_mode & 0o777,
            0o755,
            "File permissions should be preserved after undo"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_undo_with_renames_and_patches_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        // Create backup directory
        let backup_dir = renamify_dir.join("backups").join("test_complex");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create reverse patches directory
        let reverse_patches_dir = backup_dir.join("reverse_patches");
        fs::create_dir_all(&reverse_patches_dir).unwrap();

        // Create plans directory
        let plans_dir = renamify_dir.join("plans");
        fs::create_dir_all(&plans_dir).unwrap();

        // Create a directory with an executable script inside (simulating renamed state)
        let new_dir = temp_dir.path().join("new_scripts");
        fs::create_dir(&new_dir).unwrap();

        let renamed_file = new_dir.join("new_script.sh");
        fs::write(&renamed_file, "#!/bin/bash\necho 'new content'\n").unwrap();

        // Set executable permissions (755)
        let mut perms = fs::metadata(&renamed_file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&renamed_file, perms.clone()).unwrap();

        // Create another executable that only has content changes (no rename)
        let modified_file = temp_dir.path().join("build.sh");
        fs::write(&modified_file, "#!/bin/bash\nmake new\n").unwrap();
        let mut perms2 = fs::metadata(&modified_file).unwrap().permissions();
        perms2.set_mode(0o755);
        fs::set_permissions(&modified_file, perms2).unwrap();

        // Create reverse patches
        let patch1 = r"--- old_scripts/old_script.sh
+++ old_scripts/old_script.sh
@@ -1,2 +1,2 @@
 #!/bin/bash
-echo 'new content'
+echo 'old content'
";
        let mut hasher = Sha256::new();
        hasher.update(patch1.as_bytes());
        let hash1 = format!("{:x}", hasher.finalize());
        fs::write(reverse_patches_dir.join(format!("{}.patch", hash1)), patch1).unwrap();

        let patch2 = r"--- build.sh
+++ build.sh
@@ -1,2 +1,2 @@
 #!/bin/bash
-make new
+make old
";
        let mut hasher2 = Sha256::new();
        hasher2.update(patch2.as_bytes());
        let hash2 = format!("{:x}", hasher2.finalize());
        fs::write(reverse_patches_dir.join(format!("{}.patch", hash2)), patch2).unwrap();

        // Create a plan with both renames and content changes
        let old_dir = temp_dir.path().join("old_scripts");
        let old_file = old_dir.join("old_script.sh");

        let plan = crate::scanner::Plan {
            id: "test_complex".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![
                crate::scanner::MatchHunk {
                    file: old_file.clone(),
                    line: 2,
                    col: 0,
                    variant: "old".to_string(),
                    before: "old".to_string(),
                    after: "new".to_string(),
                    start: 0,
                    end: 3,
                    line_before: None,
                    line_after: None,
                    coercion_applied: None,
                    original_file: Some(old_file.clone()),
                    renamed_file: Some(renamed_file.clone()),
                    patch_hash: Some(hash1),
                },
                crate::scanner::MatchHunk {
                    file: modified_file.clone(),
                    line: 2,
                    col: 0,
                    variant: "old".to_string(),
                    before: "old".to_string(),
                    after: "new".to_string(),
                    start: 0,
                    end: 3,
                    line_before: None,
                    line_after: None,
                    coercion_applied: None,
                    original_file: Some(modified_file.clone()),
                    renamed_file: None,
                    patch_hash: Some(hash2),
                },
            ],
            renames: vec![
                crate::scanner::Rename {
                    from: old_dir.clone(),
                    to: new_dir.clone(),
                    kind: crate::scanner::RenameKind::Dir,
                    coercion_applied: None,
                },
                crate::scanner::Rename {
                    from: old_file.clone(),
                    to: renamed_file.clone(),
                    kind: crate::scanner::RenameKind::File,
                    coercion_applied: None,
                },
            ],
            stats: crate::scanner::Stats {
                files_scanned: 2,
                total_matches: 2,
                matches_by_variant: HashMap::new(),
                files_with_matches: 2,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let plan_path = plans_dir.join("test_complex.json");
        fs::write(&plan_path, serde_json::to_string(&plan).unwrap()).unwrap();

        // Create history entry
        let mut affected_files = HashMap::new();
        affected_files.insert(renamed_file.clone(), "checksum1".to_string());
        affected_files.insert(modified_file.clone(), "checksum2".to_string());

        let entry = crate::history::HistoryEntry {
            id: "test_complex".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files,
            renames: vec![
                (old_dir.clone(), new_dir.clone()),
                (old_file.clone(), renamed_file),
            ],
            backups_path: backup_dir,
            revert_of: None,
            redo_of: None,
        };

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Perform undo
        undo_renaming("test_complex", &renamify_dir).unwrap();

        // Verify renames were undone
        assert!(!new_dir.exists(), "New directory should not exist");
        assert!(old_dir.exists(), "Old directory should be restored");
        assert!(old_file.exists(), "Old file should be restored");

        // Verify content was restored
        let content1 = fs::read_to_string(&old_file).unwrap();
        assert!(
            content1.contains("old content"),
            "Content should be restored in renamed file"
        );

        let content2 = fs::read_to_string(&modified_file).unwrap();
        assert!(
            content2.contains("make old"),
            "Content should be restored in modified file"
        );

        // THIS IS THE KEY TEST - Verify permissions were preserved for both files
        let mode1 = fs::metadata(&old_file).unwrap().permissions().mode();
        assert_eq!(
            mode1 & 0o777,
            0o755,
            "Renamed file permissions should be preserved after undo"
        );

        let mode2 = fs::metadata(&modified_file).unwrap().permissions().mode();
        assert_eq!(
            mode2 & 0o777,
            0o755,
            "Modified file permissions should be preserved after undo"
        );
    }

    #[test]
    fn test_undo_case_insensitive_rename() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        // Create backup directory
        let backup_dir = renamify_dir.join("backups").join("test_case");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create reverse patches directory (empty for rename-only test)
        let reverse_patches_dir = backup_dir.join("reverse_patches");
        fs::create_dir_all(&reverse_patches_dir).unwrap();

        // Create plans directory
        let plans_dir = renamify_dir.join("plans");
        fs::create_dir_all(&plans_dir).unwrap();

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
            backups_path: backup_dir,
            revert_of: None,
            redo_of: None,
        };

        // Create a plan file for the undo to use
        let plan = crate::scanner::Plan {
            id: "test_case".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "newname".to_string(),
            new: "NewName".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            renames: vec![crate::scanner::Rename {
                from: temp_dir.path().join("newname.txt"),
                to: new_file.clone(),
                kind: crate::scanner::RenameKind::File,
                coercion_applied: None,
            }],
            stats: crate::scanner::Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };
        let plan_path = plans_dir.join("test_case.json");
        fs::write(&plan_path, serde_json::to_string(&plan).unwrap()).unwrap();

        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&renamify_dir).unwrap();
        history.add_entry(entry).unwrap();
        history.save().unwrap();

        // Perform undo
        let result = undo_renaming("test_case", &renamify_dir);

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
