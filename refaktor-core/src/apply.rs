use crate::scanner::Plan;
use crate::history::{create_history_entry, History};
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use sha2::{Digest, Sha256};

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
            Some(OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?)
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
            writeln!(file, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message)?;
            file.flush()?;
        }
        Ok(())
    }
}

/// Create a backup of a file or directory
fn backup_file(path: &Path, backup_dir: &Path, plan_id: &str) -> Result<FileBackup> {
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
    
    // Generate backup path preserving relative structure
    let backup_path = if path.is_absolute() {
        // Strip leading slash and use as relative path in backup dir
        let relative = path.strip_prefix("/").unwrap_or(path);
        backup_base.join(relative)
    } else {
        backup_base.join(path)
    };
    
    // Create parent directories for backup
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Handle directories differently from files
    if metadata.is_dir() {
        // For directories, we don't actually need to backup the entire contents
        // Just record that it was a directory - the rename tracking handles restoration
        fs::create_dir_all(&backup_path)?;
    } else {
        // Always copy for backups (hard links would share the same data and get modified)
        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to backup {} to {}", path.display(), backup_path.display()))?;
    }
    
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

/// Restore a file from backup
fn restore_backup(backup: &FileBackup) -> Result<()> {
    // Skip checksum verification for directories
    if backup.checksum != "directory" {
        // Verify backup exists and matches checksum
        let backup_checksum = calculate_checksum(&backup.backup_path)?;
        if backup_checksum != backup.checksum {
            return Err(anyhow!(
                "Backup checksum mismatch for {}: expected {}, got {}",
                backup.original_path.display(),
                backup.checksum,
                backup_checksum
            ));
        }
        
        // Remove current file if it exists
        if backup.original_path.exists() {
            fs::remove_file(&backup.original_path)?;
        }
        
        // Copy backup back to original location
        fs::copy(&backup.backup_path, &backup.original_path)?;
        
        // Restore permissions
        if let Some(ref perms) = backup.permissions {
            fs::set_permissions(&backup.original_path, perms.clone())?;
        }
    }
    // For directories, the rename rollback will handle restoration
    
    Ok(())
}

/// Apply content edits to a file atomically
fn apply_content_edits(
    path: &Path,
    replacements: &[(String, String, usize, usize)],
    state: &mut ApplyState,
) -> Result<()> {
    state.log(&format!("Applying {} edits to {}", replacements.len(), path.display()))?;
    
    // Read the file content
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    // Apply replacements (in reverse order to maintain positions)
    let mut modified = content.clone();
    for (before, after, start, end) in replacements.iter().rev() {
        // Validate the replacement matches expected content
        let actual = &content[*start..*end];
        if actual != before {
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
fn perform_rename(
    from: &Path,
    to: &Path,
    is_dir: bool,
    state: &mut ApplyState,
) -> Result<()> {
    state.log(&format!("Renaming {} -> {}", from.display(), to.display()))?;
    
    // Check if this is a case-only rename on a case-insensitive filesystem
    let case_only_rename = from.to_string_lossy().to_lowercase() == to.to_string_lossy().to_lowercase()
        && from != to;
    
    if case_only_rename && is_case_insensitive_fs(from.parent().unwrap_or(Path::new("."))) {
        // Two-step rename for case-only changes
        let temp_name = from.with_extension(format!("{}.refaktor.tmp", std::process::id()));
        
        state.log(&format!("Case-only rename detected, using temp: {}", temp_name.display()))?;
        
        fs::rename(from, &temp_name)
            .with_context(|| format!("Failed to rename {} to temp", from.display()))?;
        
        fs::rename(&temp_name, to)
            .with_context(|| format!("Failed to rename temp to {}", to.display()))?;
    } else {
        // Regular rename
        fs::rename(from, to)
            .with_context(|| format!("Failed to rename {} to {}", from.display(), to.display()))?;
    }
    
    state.renames_performed.push((from.to_path_buf(), to.to_path_buf()));
    state.log(&format!("Successfully renamed {} -> {}", from.display(), to.display()))?;
    
    Ok(())
}

/// Rollback all applied changes
fn rollback(state: &mut ApplyState) -> Result<()> {
    state.log("Starting rollback due to error")?;
    
    let mut errors = Vec::new();
    
    // Revert renames in reverse order
    let renames_to_revert: Vec<_> = state.renames_performed.iter().rev().cloned().collect();
    for (from, to) in renames_to_revert {
        state.log(&format!("Reverting rename: {} -> {}", to.display(), from.display()))?;
        if let Err(e) = fs::rename(&to, &from) {
            errors.push(format!("Failed to revert rename {} -> {}: {}", to.display(), from.display(), e));
        }
    }
    
    // Restore content edits from backups
    let backups_to_restore: Vec<_> = state.backups.clone();
    let edits_applied = state.content_edits_applied.clone();
    
    for backup in &backups_to_restore {
        if edits_applied.contains(&backup.original_path) {
            state.log(&format!("Restoring {} from backup", backup.original_path.display()))?;
            if let Err(e) = restore_backup(backup) {
                errors.push(format!("Failed to restore {}: {}", backup.original_path.display(), e));
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(anyhow!("Rollback encountered errors:\n{}", errors.join("\n")));
    }
    
    state.log("Rollback completed successfully")?;
    Ok(())
}

/// Apply a refactoring plan
pub fn apply_plan(plan: &Plan, options: &ApplyOptions) -> Result<()> {
    let mut state = ApplyState::new(plan.id.clone(), options.log_file.clone())?;
    
    state.log(&format!("Starting apply for plan {}", plan.id))?;
    state.log(&format!("Options: {:?}", options))?;
    
    // Collect all files that will be modified
    let mut files_to_backup = HashSet::new();
    
    // Files with content changes
    for hunk in &plan.matches {
        files_to_backup.insert(&hunk.file);
    }
    
    // Files and directories to be renamed
    for rename in &plan.renames {
        files_to_backup.insert(&rename.from);
    }
    
    // Create backups if requested
    if options.create_backups {
        state.log(&format!("Creating backups for {} files", files_to_backup.len()))?;
        
        for path in files_to_backup {
            match backup_file(path, &options.backup_dir, &plan.id) {
                Ok(backup) => {
                    state.log(&format!("Backed up {} to {}", path.display(), backup.backup_path.display()))?;
                    state.backups.push(backup);
                }
                Err(e) => {
                    state.log(&format!("Failed to backup {}: {}", path.display(), e))?;
                    if !options.force {
                        return Err(e);
                    }
                }
            }
        }
    }
    
    // Group content edits by file
    let mut edits_by_file: BTreeMap<&Path, Vec<_>> = BTreeMap::new();
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
        if let Err(e) = apply_content_edits(path, &edits, &mut state) {
            state.log(&format!("Error applying edits to {}: {}", path.display(), e))?;
            
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
        
        let commit_message = format!("refaktor: rename {} -> {} (#{}))", plan.old, plan.new, plan.id);
        
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
    
    let history_entry = create_history_entry(
        plan,
        affected_files,
        state.renames_performed.clone(),
        options.backup_dir.join(&plan.id),
        None, // Not a revert
        None, // Not a redo
    );
    
    let mut history = History::load(&options.backup_dir.parent().unwrap_or(Path::new(".refaktor")))?;
    history.add_entry(history_entry)?;
    
    state.log("Apply completed successfully")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{MatchHunk, Rename, RenameKind, Stats};
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
    fn test_backup_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        
        let backup_dir = temp_dir.path().join("backups");
        let backup = backup_file(&test_file, &backup_dir, "test_plan").unwrap();
        
        assert!(backup.backup_path.exists());
        assert_eq!(fs::read_to_string(&backup.backup_path).unwrap(), "test content");
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
    fn test_restore_backup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "original content").unwrap();
        
        let backup_dir = temp_dir.path().join("backups");
        let backup = backup_file(&test_file, &backup_dir, "test_plan").unwrap();
        
        // Modify the original file
        fs::write(&test_file, "modified content").unwrap();
        
        // Restore from backup
        restore_backup(&backup).unwrap();
        
        assert_eq!(fs::read_to_string(&test_file).unwrap(), "original content");
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