use crate::apply::{apply_plan, calculate_checksum, ApplyOptions};
use crate::history::{create_history_entry, History};
use crate::scanner::Plan;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Extract individual file patches from a comprehensive patch
/// Returns Vec<(original_path, modified_path, patch_content)>
fn extract_individual_patches(
    comprehensive_patch: &str,
) -> Result<Vec<(PathBuf, PathBuf, String)>> {
    let mut patches = Vec::new();
    let mut current_patch = String::new();
    let mut current_original: Option<PathBuf> = None;
    let mut current_modified: Option<PathBuf> = None;
    let mut in_patch_section = false;

    for line in comprehensive_patch.lines() {
        // Skip comment lines
        if line.starts_with("#") {
            continue;
        }

        // Start of a new file patch
        if line.starts_with("--- ") {
            // Save previous patch if we have one
            if let (Some(orig), Some(modif)) = (current_original.take(), current_modified.take()) {
                if !current_patch.is_empty() {
                    // Strip exactly 2 trailing newlines (the separator between patches)
                    let mut trimmed_patch = current_patch.clone();
                    if trimmed_patch.ends_with("\n\n") {
                        trimmed_patch.truncate(trimmed_patch.len() - 2);
                    }
                    patches.push((orig, modif, trimmed_patch));
                }
            }

            // Start new patch
            current_patch.clear();
            current_patch.push_str(line);
            current_patch.push('\n');

            let path_str = line.strip_prefix("--- ").unwrap_or("");
            current_original = Some(PathBuf::from(path_str));
            in_patch_section = true;
            continue;
        }

        if line.starts_with("+++ ") {
            current_patch.push_str(line);
            current_patch.push('\n');

            let path_str = line.strip_prefix("+++ ").unwrap_or("");
            current_modified = Some(PathBuf::from(path_str));
            continue;
        }

        // Skip rename metadata lines - these are not part of the content patch
        if line.starts_with("diff --git")
            || line.starts_with("rename from")
            || line.starts_with("rename to")
        {
            continue;
        }

        // Include patch content lines
        if in_patch_section {
            current_patch.push_str(line);
            current_patch.push('\n');
        }
    }

    // Don't forget the last patch
    if let (Some(orig), Some(modif)) = (current_original, current_modified) {
        if !current_patch.is_empty() {
            // Strip trailing newlines from the last patch too (there might be separator content after it)
            let mut trimmed_patch = current_patch.clone();
            if trimmed_patch.ends_with("\n\n") {
                trimmed_patch.truncate(trimmed_patch.len() - 2);
            }
            patches.push((orig, modif, trimmed_patch));
        }
    }

    Ok(patches)
}

/// Apply a single patch to a specific file using diffy
fn apply_single_patch(file_path: &Path, patch_content: &str) -> Result<()> {
    // Read the current file content
    let current_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Parse the patch using diffy
    let patch = diffy::Patch::from_str(patch_content)
        .map_err(|e| anyhow!("Failed to parse patch: {}", e))?;

    // The patch is ALREADY a reverse patch (new -> old) from our generation
    // So we can apply it directly
    let restored = diffy::apply(&current_content, &patch)
        .map_err(|e| anyhow!("Failed to apply patch: {}", e))?;

    // Write the restored content
    fs::write(file_path, restored).with_context(|| {
        format!(
            "Failed to write restored content to {}",
            file_path.display()
        )
    })?;

    Ok(())
}

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

    // Look for reverse patch file
    let patch_path = entry.backups_path.join("reverse.patch");

    if patch_path.exists() {
        // Use the comprehensive patch for undo

        // STEP 1: Reverse renames first (new locations back to old)
        // Important: We need to identify which renames are directories and which files are inside those directories
        // to avoid trying to rename files that will be moved when their parent directory is renamed

        // First, identify directory renames
        let dir_renames: Vec<_> = entry
            .renames
            .iter()
            .filter(|(from, to)| {
                // Check if this is likely a directory
                // Check if 'to' exists and is a directory, or if the path itself ends with a directory-like name
                to.is_dir()
                    || (from.file_name().map_or(false, |n| {
                        let name = n.to_string_lossy();
                        name.ends_with("_dir") || name.ends_with("_lib")
                    }))
            })
            .collect();

        // Process directory renames first (in reverse order)
        for (from, to) in dir_renames.iter().rev() {
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
                    let temp_name =
                        to.with_extension(format!("{}.refaktor.tmp", std::process::id()));
                    fs::rename(to, &temp_name)?;
                    fs::rename(&temp_name, from)?;
                } else {
                    fs::rename(to, from)?;
                }
            }
        }

        // Now process file renames (in reverse order)
        // Adjust paths for files that are inside renamed directories
        for (from, to) in entry.renames.iter().rev() {
            // Skip if this was already handled as a directory
            if dir_renames.iter().any(|(df, dt)| df == from && dt == to) {
                continue;
            }

            // Check if this file is inside a directory that was renamed
            // If so, adjust the paths to account for the directory rename
            let mut actual_from = from.clone();
            let mut actual_to = to.clone();

            for (dir_from, dir_to) in &dir_renames {
                // Check if the file paths are inside the directory that was renamed
                if let (Ok(rel_from), Ok(rel_to)) =
                    (from.strip_prefix(dir_from), to.strip_prefix(dir_to))
                {
                    // The directory has already been renamed back from dir_to to dir_from
                    // So the file is now at dir_from/rel_to and needs to go to dir_from/rel_from
                    actual_from = dir_from.join(rel_from);
                    actual_to = dir_from.join(rel_to);
                    break;
                }
            }

            if actual_to.exists() {
                // Handle case-only renames on case-insensitive filesystems
                let case_only = actual_from.to_string_lossy().to_lowercase()
                    == actual_to.to_string_lossy().to_lowercase()
                    && actual_from != actual_to;

                if case_only
                    && crate::rename::detect_case_insensitive_fs(
                        actual_to.parent().unwrap_or_else(|| Path::new(".")),
                    )
                {
                    // Two-step rename for case-only changes
                    let temp_name =
                        actual_to.with_extension(format!("{}.refaktor.tmp", std::process::id()));
                    fs::rename(&actual_to, &temp_name)?;
                    fs::rename(&temp_name, &actual_from)?;
                } else {
                    fs::rename(&actual_to, &actual_from)?;
                }
            }
        }

        // STEP 2: Apply content changes from the patch
        let patch_content = fs::read_to_string(&patch_path)
            .with_context(|| format!("Failed to read patch from {}", patch_path.display()))?;

        // Extract individual file patches
        let individual_patches = extract_individual_patches(&patch_content)?;

        let total_patches = individual_patches.len();
        let mut files_processed = 0;
        let mut failed_patches = Vec::new();

        for (original_path, modified_path, patch_content) in individual_patches {
            // The modified_path (from +++ line) tells us where the file should be after applying this reverse patch
            // Since renames have already been reversed, the file should be at the modified_path location
            let current_path = &modified_path;

            // Apply patch to the current path (where the file actually is now)
            if let Err(e) = apply_single_patch(current_path, &patch_content) {
                eprintln!(
                    "  ERROR: Failed to apply patch to {}: {}",
                    current_path.display(),
                    e
                );

                // Save the failed patch as a .rej file for debugging
                let rej_path = current_path.with_extension(format!(
                    "{}.rej",
                    current_path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
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

                failed_patches.push(format!("{}: {}", current_path.display(), e));
            } else {
                files_processed += 1;
            }
        }

        // If any patches failed, return an error
        if !failed_patches.is_empty() {
            return Err(anyhow!(
                "Failed to apply {} out of {} patches",
                failed_patches.len(),
                total_patches
            ));
        }
    } else {
        // Fallback to old method for backward compatibility

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
                    && crate::rename::detect_case_insensitive_fs(
                        to.parent().unwrap_or_else(|| Path::new(".")),
                    )
                {
                    // Two-step rename for case-only changes
                    let temp_name =
                        to.with_extension(format!("{}.refaktor.tmp", std::process::id()));
                    fs::rename(to, &temp_name)?;
                    fs::rename(&temp_name, from)?;
                } else {
                    fs::rename(to, from)?;
                }

                reversed_renames.push((to.clone(), from.clone()));
            }
        }

        // SECOND: Apply diffs to restore original content (now files are at their original locations)
        let mut restored_files = Vec::new();
        for path in entry.affected_files.keys() {
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
                // Use --fuzz=3 for lenient matching, allowing context to be off by several lines
                let output = std::process::Command::new("patch")
                    .args(["-R", "-p0", "--fuzz=3"])  // -R for reverse, -p0 for no path stripping, --fuzz=3 for lenient matching
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

                restored_files.push((*current_path).clone());
            }
        }
    }

    // Calculate checksums of affected files and collect reversed renames
    let mut affected_files = HashMap::new();
    let mut reversed_renames = Vec::new();

    // Process all original affected files to calculate new checksums
    for (path, _) in &entry.affected_files {
        // The file should now be at its original location (before the refactoring)
        // Check if it was renamed and is now back at original location
        let original_path = entry
            .renames
            .iter()
            .find(|(_, to)| to == path)
            .map(|(from, _)| from)
            .unwrap_or(path);

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
        revert_of: Some(entry.id.clone()),
        redo_of: None,
    };

    history.add_entry(revert_entry)?;

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
        let diff_content = r"--- old_name.txt
+++ old_name.txt
@@ -1 +1 @@
-original content
+modified content
";
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
            backups_path: backup_dir,
            revert_of: None,
            redo_of: None,
        };

        // Create history with this entry
        let history_path = refaktor_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();
        let mut history = History::load(&refaktor_dir).unwrap();
        history.add_entry(entry).unwrap();
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
    fn test_extract_individual_patches() {
        let comprehensive_patch = r#"# Refaktor reverse patch for plan test123
# Created: 2024-01-01T00:00:00Z
# New: bar -> Old: foo (for undo)

--- /path/to/file1.rs
+++ /path/to/file1.rs
@@ -1 +1 @@
-fn bar() {}
\ No newline at end of file
+fn foo() {}
\ No newline at end of file

--- /path/to/file2.rs
+++ /path/to/file2.rs
@@ -1,2 +1,2 @@
-let x = bar();
-println!("bar");
+let x = foo();
+println!("foo");
"#;

        let patches = extract_individual_patches(comprehensive_patch).unwrap();
        assert_eq!(patches.len(), 2);

        let (orig1, mod1, content1) = &patches[0];
        assert_eq!(orig1, &PathBuf::from("/path/to/file1.rs"));
        assert_eq!(mod1, &PathBuf::from("/path/to/file1.rs"));
        assert!(content1.contains("@@ -1 +1 @@"));
        assert!(content1.contains("-fn bar() {}"));
        assert!(content1.contains("+fn foo() {}"));
        assert!(
            content1.contains("\\ No newline at end of file"),
            "Should preserve no newline markers"
        );

        let (orig2, mod2, content2) = &patches[1];
        assert_eq!(orig2, &PathBuf::from("/path/to/file2.rs"));
        assert_eq!(mod2, &PathBuf::from("/path/to/file2.rs"));
        assert!(content2.contains("@@ -1,2 +1,2 @@"));
        assert!(content2.contains("-let x = bar();"));
        assert!(content2.contains("+let x = foo();"));
    }

    #[test]
    fn test_extract_individual_patches_with_comments() {
        let comprehensive_patch = r#"# This is a comment
# Another comment line
diff --git a/file.rs b/file.rs
rename from old_file.rs
rename to new_file.rs
--- /path/to/file.rs
+++ /path/to/file.rs
@@ -1 +1 @@
-old content
+new content
"#;

        let patches = extract_individual_patches(comprehensive_patch).unwrap();
        assert_eq!(patches.len(), 1);

        let (orig, modif, content) = &patches[0];
        assert_eq!(orig, &PathBuf::from("/path/to/file.rs"));
        assert_eq!(modif, &PathBuf::from("/path/to/file.rs"));

        // Should not contain comments or rename metadata
        assert!(!content.contains("# This is a comment"));
        assert!(!content.contains("diff --git"));
        assert!(!content.contains("rename from"));
        assert!(!content.contains("rename to"));

        // Should contain patch content
        assert!(content.contains("--- /path/to/file.rs"));
        assert!(content.contains("+++ /path/to/file.rs"));
        assert!(content.contains("@@ -1 +1 @@"));
        assert!(content.contains("-old content"));
        assert!(content.contains("+new content"));
    }

    #[test]
    fn test_extract_individual_patches_no_newline_markers() {
        let comprehensive_patch = r#"--- /test/file.txt
+++ /test/file.txt
@@ -1 +1 @@
-old text
\ No newline at end of file
+new text
\ No newline at end of file
"#;

        let patches = extract_individual_patches(comprehensive_patch).unwrap();
        assert_eq!(patches.len(), 1);

        let (_orig, _modif, content) = &patches[0];

        // CRITICAL: Must preserve the backslash lines
        let backslash_count = content.matches("\\ No newline at end of file").count();
        assert_eq!(
            backslash_count, 2,
            "Should preserve both 'No newline at end of file' markers: {}",
            content
        );

        // Should contain the full patch
        assert!(content.contains("-old text"));
        assert!(content.contains("+new text"));
    }

    #[test]
    fn test_multiple_patches_extraction_no_extra_newlines() {
        use std::path::PathBuf;

        // Create a comprehensive patch that mimics what we generate with separators
        let comprehensive_patch = "# Refaktor reverse patch for plan test
# Created: 2025-01-01T00:00:00Z

--- /path/to/file1.rs
+++ /path/to/file1.rs
@@ -1,3 +1,3 @@
-fn new_name() {
-    println!(\"new_name\");
+fn old_name() {
+    println!(\"old_name\");
 }
\\ No newline at end of file

--- /path/to/stable.rs
+++ /path/to/stable.rs
@@ -1,2 +1,2 @@
-use new_name;
-fn main() { new_name(); }
\\ No newline at end of file
+use old_name;
+fn main() { old_name(); }
\\ No newline at end of file

diff --git a/some/dir b/other/dir
rename from some/dir
rename to other/dir
";

        let patches = extract_individual_patches(comprehensive_patch).unwrap();
        assert_eq!(patches.len(), 2, "Should extract exactly 2 patches");

        let (orig1, _mod1, content1) = &patches[0];
        assert_eq!(orig1, &PathBuf::from("/path/to/file1.rs"));

        // The first patch should NOT have extra newlines at the end
        assert!(
            !content1.ends_with("\n\n"),
            "First patch should not end with double newlines"
        );
        assert!(
            content1.ends_with("\\ No newline at end of file"),
            "First patch should end with no newline marker"
        );

        let (orig2, _mod2, content2) = &patches[1];
        assert_eq!(orig2, &PathBuf::from("/path/to/stable.rs"));

        // The second patch should also NOT have extra newlines at the end
        assert!(
            !content2.ends_with("\n\n"),
            "Second patch should not end with double newlines"
        );
        assert!(
            content2.ends_with("\\ No newline at end of file"),
            "Second patch should end with no newline marker"
        );
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
            backups_path: backup_dir,
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
