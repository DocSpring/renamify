use crate::history::{create_history_entry, History};
use crate::scanner::Plan;
use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Options for applying a refactoring plan
#[derive(Debug, Clone)]
pub struct ApplyOptions {
    /// Create backups before applying changes
    pub create_backups: bool,
    /// Path to backup directory
    pub backup_dir: PathBuf,
    /// Perform atomic operations
    pub atomic: bool,
    /// Commit changes to git after successful apply
    pub commit: bool,
    /// Force apply even with conflicts
    pub force: bool,
    /// Skip symlinks
    pub skip_symlinks: bool,
    /// Log file path
    pub log_file: Option<PathBuf>,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        Self {
            create_backups: true,
            backup_dir: PathBuf::from(".refaktor/backups"),
            atomic: true,
            commit: false,
            force: false,
            skip_symlinks: true,
            log_file: Some(PathBuf::from(".refaktor/apply.log")),
        }
    }
}

/// Represents a backup of a file
#[derive(Debug, Clone)]
struct FileBackup {
    original_path: PathBuf,
    backup_path: PathBuf,
    checksum: String,
    permissions: Option<fs::Permissions>,
    modified_time: SystemTime,
}

/// Tracks the state of an apply operation
pub struct ApplyState {
    plan_id: String,
    backups: Vec<FileBackup>,
    content_edits_applied: Vec<PathBuf>,
    renames_performed: Vec<(PathBuf, PathBuf)>,
    log_file: Option<File>,
}

impl ApplyState {
    fn new(plan_id: String, log_file: Option<PathBuf>) -> Result<Self> {
        let log_file = if let Some(path) = log_file {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            Some(OpenOptions::new().create(true).append(true).open(&path)?)
        } else {
            None
        };

        Ok(Self {
            plan_id,
            backups: Vec::new(),
            content_edits_applied: Vec::new(),
            renames_performed: Vec::new(),
            log_file,
        })
    }

    fn log(&mut self, message: &str) -> Result<()> {
        if let Some(ref mut file) = self.log_file {
            writeln!(
                file,
                "[{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                message
            )?;
            file.flush()?;
        }
        Ok(())
    }
}

/// Create a diff-based backup of a file
fn create_diff_backup(
    original_path: &Path,
    modified_content: &str,
    backup_dir: &Path,
    plan_id: &str,
) -> Result<PathBuf> {
    // Create backup directory structure
    let backup_base = backup_dir.join(plan_id);
    fs::create_dir_all(&backup_base)?;

    // Generate diff path using the filename + .diff extension
    let diff_filename = format!(
        "{}.diff",
        original_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    let diff_path = backup_base.join(diff_filename);

    // Create a temporary file with the modified content
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("refaktor_{}.tmp", std::process::id()));
    fs::write(&temp_file, modified_content)?;

    // Generate diff using git diff
    // Note: git diff returns exit code 1 when files differ, which is expected
    let output = std::process::Command::new("git")
        .args([
            "diff",
            "--no-index",
            "--no-prefix",
            original_path.to_str().unwrap(),
            temp_file.to_str().unwrap(),
        ])
        .output();

    // Always clean up temp file, even if git diff failed
    let _ = fs::remove_file(&temp_file);

    // Check if git diff succeeded
    let output = output.context("Failed to run git diff")?;

    // Save the diff (even if empty, which might happen if files are identical)
    fs::write(&diff_path, &output.stdout)?;

    Ok(diff_path)
}

/// Create a backup of a file or directory (for renames only)
fn backup_file_metadata(path: &Path, backup_dir: &Path, plan_id: &str) -> Result<FileBackup> {
    // Read file metadata
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

    // Skip if it's a symlink and we're configured to skip them
    if metadata.is_symlink() {
        return Err(anyhow!("Cannot backup symlink: {}", path.display()));
    }

    // Calculate checksum of the original file (skip for directories)
    let checksum = if metadata.is_file() {
        calculate_checksum(path)?
    } else {
        // For directories, use a placeholder checksum
        "directory".to_string()
    };

    // Create backup directory structure
    let backup_base = backup_dir.join(plan_id);
    fs::create_dir_all(&backup_base)?;

    // For diff-based backups, we don't copy the file, just track metadata
    let backup_path = backup_base.join(path.file_name().unwrap_or(path.as_os_str()));

    Ok(FileBackup {
        original_path: path.to_path_buf(),
        backup_path,
        checksum,
        permissions: Some(metadata.permissions()),
        modified_time: metadata.modified()?,
    })
}

/// Calculate SHA256 checksum of a file
pub fn calculate_checksum(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Apply content edits to a file atomically
fn apply_content_edits(
    path: &Path,
    replacements: &[(String, String, usize, usize)],
    state: &mut ApplyState,
    backup_dir: &Path,
    plan_id: &str,
) -> Result<()> {
    state.log(&format!(
        "Applying {} edits to {}",
        replacements.len(),
        path.display()
    ))?;

    // Read the original file content (for diff generation)
    let original_content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    // Apply replacements (in reverse order to maintain positions)
    let mut modified = original_content.clone();

    // Debug Train-Case replacements
    if std::env::var("REFAKTOR_DEBUG_TRAIN_CASE").is_ok() {
        eprintln!("\n=== Applying replacements to {} ===", path.display());
        eprintln!("Total replacements: {}", replacements.len());
        for (before, after, start, end) in replacements {
            if before.contains('-') && before.chars().next().map_or(false, |c| c.is_uppercase()) {
                eprintln!(
                    "  Train-Case: '{}' -> '{}' at [{}, {}]",
                    before, after, start, end
                );
            }
        }
    }

    for (before, after, start, end) in replacements.iter().rev() {
        // Validate the replacement matches expected content
        let actual = &original_content[*start..*end];
        if actual != before {
            if std::env::var("REFAKTOR_DEBUG_TRAIN_CASE").is_ok() {
                eprintln!("ERROR: Content mismatch!");
                eprintln!("  Expected: '{}'", before);
                eprintln!("  Found: '{}'", actual);
                eprintln!("  Position: [{}, {}]", start, end);
            }
            return Err(anyhow!(
                "Content mismatch in {}: expected '{}', found '{}'",
                path.display(),
                before,
                actual
            ));
        }

        // Apply the replacement
        modified.replace_range(*start..*end, after);
    }

    // Create a diff backup before applying changes
    let diff_path = create_diff_backup(path, &modified, backup_dir, plan_id)?;
    state.log(&format!("Created diff backup at {}", diff_path.display()))?;

    // Write to temporary file in the same directory (for atomicity)
    let temp_path = path.with_extension(format!("{}.refaktor.tmp", std::process::id()));

    {
        let mut temp_file = File::create(&temp_path)?;
        temp_file.write_all(modified.as_bytes())?;
        temp_file.sync_all()?; // fsync
    }

    // Atomic rename
    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to atomically replace {}", path.display()))?;

    // Sync parent directory on Unix
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            let dir = File::open(parent)?;
            dir.sync_all()?;
        }
    }

    state.content_edits_applied.push(path.to_path_buf());
    state.log(&format!("Successfully applied edits to {}", path.display()))?;

    Ok(())
}

/// Check if filesystem is case-insensitive
fn is_case_insensitive_fs(path: &Path) -> bool {
    // Create a test file with lowercase name
    let test_lower = path.join(".refaktor_case_test");
    let test_upper = path.join(".REFAKTOR_CASE_TEST");

    // Try to create the lowercase file
    if File::create(&test_lower).is_ok() {
        // Check if uppercase path exists
        let case_insensitive = test_upper.exists();

        // Clean up
        let _ = fs::remove_file(&test_lower);

        case_insensitive
    } else {
        // Assume case-sensitive if we can't test
        false
    }
}

/// Perform a file or directory rename
fn perform_rename(from: &Path, to: &Path, is_dir: bool, state: &mut ApplyState) -> Result<()> {
    state.log(&format!("Renaming {} -> {}", from.display(), to.display()))?;

    // Check if this is a case-only rename on a case-insensitive filesystem
    let case_only_rename =
        from.to_string_lossy().to_lowercase() == to.to_string_lossy().to_lowercase() && from != to;

    if case_only_rename && is_case_insensitive_fs(from.parent().unwrap_or_else(|| Path::new("."))) {
        // Two-step rename for case-only changes
        let temp_name = from.with_extension(format!("{}.refaktor.tmp", std::process::id()));

        state.log(&format!(
            "Case-only rename detected, using temp: {}",
            temp_name.display()
        ))?;

        fs::rename(from, &temp_name)
            .with_context(|| format!("Failed to rename {} to temp", from.display()))?;

        fs::rename(&temp_name, to)
            .with_context(|| format!("Failed to rename temp to {}", to.display()))?;
    } else {
        // Regular rename
        fs::rename(from, to)
            .with_context(|| format!("Failed to rename {} to {}", from.display(), to.display()))?;
    }

    state
        .renames_performed
        .push((from.to_path_buf(), to.to_path_buf()));
    state.log(&format!(
        "Successfully renamed {} -> {}",
        from.display(),
        to.display()
    ))?;

    Ok(())
}

/// Rollback all applied changes
fn rollback(state: &mut ApplyState) -> Result<()> {
    state.log("Starting rollback due to error")?;

    let mut errors = Vec::new();

    // Revert renames in reverse order
    let renames_to_revert: Vec<_> = state.renames_performed.iter().rev().cloned().collect();
    for (from, to) in renames_to_revert {
        state.log(&format!(
            "Reverting rename: {} -> {}",
            to.display(),
            from.display()
        ))?;
        if let Err(e) = fs::rename(&to, &from) {
            errors.push(format!(
                "Failed to revert rename {} -> {}: {}",
                to.display(),
                from.display(),
                e
            ));
        }
    }

    // Note: Content edits are not rolled back here since we use diffs for undo
    // The rollback only handles reverting renames during a failed apply

    if !errors.is_empty() {
        return Err(anyhow!(
            "Rollback encountered errors:\n{}",
            errors.join("\n")
        ));
    }

    state.log("Rollback completed successfully")?;
    Ok(())
}

/// Apply a refactoring plan
pub fn apply_plan(plan: &Plan, options: &ApplyOptions) -> Result<()> {
    let mut state = ApplyState::new(plan.id.clone(), options.log_file.clone())?;

    state.log(&format!("Starting apply for plan {}", plan.id))?;
    state.log(&format!("Options: {:?}", options))?;

    // Only backup files that will be renamed (content changes use diffs)
    let mut files_to_backup = HashSet::new();

    // Files and directories to be renamed
    for rename in &plan.renames {
        files_to_backup.insert(&rename.from);
    }

    // Create metadata backups for renames if requested
    if options.create_backups && !files_to_backup.is_empty() {
        state.log(&format!(
            "Creating metadata backups for {} items being renamed",
            files_to_backup.len()
        ))?;

        for path in files_to_backup {
            match backup_file_metadata(path, &options.backup_dir, &plan.id) {
                Ok(backup) => {
                    state.log(&format!("Backed up metadata for {}", path.display()))?;
                    state.backups.push(backup);
                },
                Err(e) => {
                    state.log(&format!("Failed to backup {}: {}", path.display(), e))?;
                    if !options.force {
                        return Err(e);
                    }
                },
            }
        }
    }

    // Group content edits by file
    let mut edits_by_file: BTreeMap<&Path, Vec<_>> = BTreeMap::new();

    // Debug Train-Case patterns in plan
    if std::env::var("REFAKTOR_DEBUG_TRAIN_CASE").is_ok() {
        eprintln!("\n=== Plan matches for Train-Case patterns ===");
        for hunk in &plan.matches {
            if hunk.before.contains('-')
                && hunk
                    .before
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_uppercase())
            {
                eprintln!("  File: {}", hunk.file.display());
                eprintln!("    Before: '{}'", hunk.before);
                eprintln!("    After: '{}'", hunk.after);
                eprintln!("    Position: [{}, {}]", hunk.start, hunk.end);
            }
        }
    }

    for hunk in &plan.matches {
        edits_by_file.entry(&hunk.file).or_default().push((
            hunk.before.clone(),
            hunk.after.clone(),
            hunk.start,
            hunk.end,
        ));
    }

    // Apply content edits
    for (path, edits) in edits_by_file {
        if let Err(e) = apply_content_edits(path, &edits, &mut state, &options.backup_dir, &plan.id)
        {
            state.log(&format!(
                "Error applying edits to {}: {}",
                path.display(),
                e
            ))?;

            if options.atomic {
                rollback(&mut state)?;
            }

            return Err(e);
        }
    }

    // Sort renames by depth (deepest first) for proper ordering
    let mut renames = plan.renames.clone();
    renames.sort_by_key(|r| std::cmp::Reverse(r.from.components().count()));

    // Apply renames
    for rename in &renames {
        let is_dir = rename.kind == crate::scanner::RenameKind::Dir;

        if let Err(e) = perform_rename(&rename.from, &rename.to, is_dir, &mut state) {
            state.log(&format!("Error performing rename: {}", e))?;

            if options.atomic {
                rollback(&mut state)?;
            }

            return Err(e);
        }
    }

    // Commit to git if requested
    if options.commit {
        state.log("Creating git commit")?;

        let output = std::process::Command::new("git")
            .args(["add", "-A"])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to stage changes: {}", error));
        }

        let commit_message = format!(
            "refaktor: rename {} -> {} (#{}))",
            plan.old, plan.new, plan.id
        );

        let output = std::process::Command::new("git")
            .args(["commit", "-m", &commit_message])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to commit changes: {}", error));
        }

        state.log(&format!("Created git commit: {}", commit_message))?;
    }

    // Record in history
    state.log("Recording in history")?;

    // Calculate checksums for all affected files
    let mut affected_files = HashMap::new();
    for path in &state.content_edits_applied {
        if path.exists() {
            let checksum = calculate_checksum(path)?;
            affected_files.insert(path.clone(), checksum);
        }
    }

    // Also include renamed files
    for (_, to) in &state.renames_performed {
        if to.exists() && to.is_file() {
            let checksum = calculate_checksum(to)?;
            affected_files.insert(to.clone(), checksum);
        }
    }

    // Pass the correct backup path to history entry
    // If backup_dir already includes the plan_id, use it as-is
    // Otherwise, append the plan_id
    let backups_path = if options.backup_dir.ends_with(&plan.id) {
        options.backup_dir.clone()
    } else {
        options.backup_dir.join(&plan.id)
    };

    let history_entry = create_history_entry(
        plan,
        affected_files,
        state.renames_performed.clone(),
        backups_path,
        None, // Not a revert
        None, // Not a redo
    );

    // Determine the .refaktor directory location
    // If backup_dir is .refaktor/backups/plan_id, we want .refaktor
    // If backup_dir is .refaktor/backups, we want .refaktor
    let refaktor_dir = if options.backup_dir.ends_with(&plan.id) {
        // backup_dir is .refaktor/backups/plan_id
        options.backup_dir
            .parent() // .refaktor/backups
            .and_then(|p| p.parent()) // .refaktor
            .unwrap_or_else(|| Path::new(".refaktor"))
    } else {
        // backup_dir is .refaktor/backups
        options.backup_dir
            .parent() // .refaktor
            .unwrap_or_else(|| Path::new(".refaktor"))
    };

    let mut history = History::load(refaktor_dir)?;
    history.add_entry(history_entry)?;

    // Store the complete plan for redo functionality
    let plans_dir = refaktor_dir.join("plans");
    fs::create_dir_all(&plans_dir)?;
    let plan_path = plans_dir.join(format!("{}.json", plan.id));
    let plan_json = serde_json::to_string_pretty(plan)?;
    fs::write(&plan_path, plan_json)?;
    state.log(&format!("Stored plan at {}", plan_path.display()))?;

    state.log("Apply completed successfully")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]
    use super::*;
    use crate::scanner::{MatchHunk, Rename, RenameKind, Stats};
    use serial_test::serial;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_plan() -> Plan {
        Plan {
            id: "test123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
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
    #[serial]
    fn test_apply_creates_correct_history_entry() {
        // Test that apply creates history entries with correct backup paths
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn old_name() {}").unwrap();

        let plan = Plan {
            id: "test_plan_456".to_string(),
            created_at: "2024-01-01".to_string(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![MatchHunk {
                file: test_file.clone(),
                line: 1,
                col: 3,
                variant: "old_name".to_string(),
                before: "old_name".to_string(),
                after: "new_name".to_string(),
                start: 3,
                end: 11,
                line_before: Some("fn old_name() {}".to_string()),
                line_after: Some("fn new_name() {}".to_string()),
                coercion_applied: None,
            }],
            renames: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant: HashMap::new(),
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
        };

        let options = ApplyOptions {
            backup_dir: temp_dir.path().join(".refaktor/backups"),
            create_backups: true,
            ..Default::default()
        };

        // Apply the plan
        apply_plan(&plan, &options).unwrap();

        // Check the file was modified
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "fn new_name() {}");

        // Check diff backup was created at the right path
        let expected_diff = options
            .backup_dir
            .join("test_plan_456")
            .join("test.rs.diff");
        assert!(
            expected_diff.exists(),
            "Diff backup should exist at {:?}",
            expected_diff
        );
        // The diff should contain the changes
        let diff_content = fs::read_to_string(&expected_diff).unwrap();
        assert!(diff_content.contains("old_name"));
        assert!(diff_content.contains("new_name"));

        // Load history and check the entry
        let history = History::load(temp_dir.path().join(".refaktor").as_path()).unwrap();
        let entries = history.list_entries(None);
        assert_eq!(entries.len(), 1);

        let entry = &entries[0];
        assert_eq!(entry.id, "test_plan_456");
        assert_eq!(entry.backups_path, options.backup_dir.join("test_plan_456"));
        assert!(
            !entry.affected_files.is_empty(),
            "affected_files should not be empty"
        );
    }

    #[test]
    #[serial]
    fn test_state_tracks_content_edits() {
        // Test that ApplyState correctly tracks content_edits_applied
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn old_name() {}").unwrap();

        let mut state = ApplyState {
            plan_id: "test".to_string(),
            content_edits_applied: Vec::new(),
            renames_performed: Vec::new(),
            backups: Vec::new(),
            log_file: None,
        };

        let replacements = vec![("old_name".to_string(), "new_name".to_string(), 3, 11)];

        // Apply edits (now requires backup_dir and plan_id)
        let backup_dir = temp_dir.path().join(".refaktor/backups");
        fs::create_dir_all(&backup_dir).unwrap();
        apply_content_edits(&test_file, &replacements, &mut state, &backup_dir, "test").unwrap();

        // Check state was updated
        assert_eq!(
            state.content_edits_applied.len(),
            1,
            "Should have tracked 1 file edit"
        );
        assert_eq!(
            state.content_edits_applied[0], test_file,
            "Should track the correct file"
        );

        // Check file was actually modified
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "fn new_name() {}");
    }

    #[test]
    #[serial]
    fn test_apply_plan_populates_affected_files() {
        // Test that apply_plan results in non-empty affected_files in history
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn old() {}").unwrap();

        let plan = Plan {
            id: "test_affected".to_string(),
            created_at: "2024-01-01".to_string(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![MatchHunk {
                file: test_file.clone(),
                line: 1,
                col: 3,
                variant: "old".to_string(),
                before: "old".to_string(),
                after: "new".to_string(),
                start: 3,
                end: 6,
                line_before: Some("fn old() {}".to_string()),
                line_after: Some("fn new() {}".to_string()),
                coercion_applied: None,
            }],
            renames: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant: HashMap::new(),
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
        };

        let options = ApplyOptions {
            backup_dir: temp_dir.path().join(".refaktor/backups"),
            create_backups: true,
            ..Default::default()
        };

        // Apply should work
        apply_plan(&plan, &options).unwrap();

        // Load history and verify affected_files is populated
        let history = History::load(temp_dir.path().join(".refaktor").as_path()).unwrap();
        let entries = history.list_entries(None);
        assert_eq!(entries.len(), 1);

        let entry = &entries[0];
        assert!(
            !entry.affected_files.is_empty(),
            "affected_files should NOT be empty after apply! State tracking is broken!"
        );
        assert!(
            entry.affected_files.contains_key(&test_file),
            "affected_files should contain the modified file"
        );
    }

    #[test]
    fn test_checksum_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let checksum = calculate_checksum(&test_file).unwrap();
        assert!(!checksum.is_empty());

        // Same content should produce same checksum
        let checksum2 = calculate_checksum(&test_file).unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_case_insensitive_detection() {
        let temp_dir = TempDir::new().unwrap();

        // This test may behave differently on different filesystems
        let is_ci = is_case_insensitive_fs(temp_dir.path());

        #[cfg(target_os = "macos")]
        assert!(is_ci); // macOS is typically case-insensitive

        #[cfg(target_os = "linux")]
        assert!(!is_ci); // Linux is typically case-sensitive
    }
}
