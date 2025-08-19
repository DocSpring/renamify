use crate::history::{create_history_entry, History};
use crate::scanner::Plan;
use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Options for applying a renaming plan
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
            backup_dir: PathBuf::from(".renamify/backups"),
            atomic: true,
            commit: false,
            force: false,
            skip_symlinks: true,
            log_file: Some(PathBuf::from(".renamify/apply.log")),
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

/// Generate individual reverse patch files for each changed file
fn generate_reverse_patches(
    plan: &mut Plan,
    options: &ApplyOptions,
    state: &ApplyState,
    original_contents: &HashMap<PathBuf, String>,
) -> Result<()> {
    // Create backup directory structure including reverse_patches subdirectory
    let backup_base = options.backup_dir.join(&plan.id);
    let reverse_patches_dir = backup_base.join("reverse_patches");
    fs::create_dir_all(&reverse_patches_dir)?;

    // Track created directories
    let mut created_dirs = Vec::new();

    // Process each file that had content changes
    for (original_path, original_content) in original_contents {
        // Find the current path for this file (may have been renamed)
        let current_path = if let Some((_, to)) = state
            .renames_performed
            .iter()
            .find(|(from, _)| from == original_path)
        {
            to.clone()
        } else {
            // Check if any parent directory was renamed
            let mut current = original_path.clone();
            for (from, to) in &state.renames_performed {
                if let Ok(relative) = original_path.strip_prefix(from) {
                    current = to.join(relative);
                    break;
                }
            }
            current
        };

        // Read current content
        let current_content = fs::read_to_string(&current_path).with_context(|| {
            format!(
                "Failed to read current content from {}",
                current_path.display()
            )
        })?;

        // Generate REVERSE diff (new -> old) for undo using diffy
        let reverse_patch =
            diffy::create_patch(current_content.as_str(), original_content.as_str());
        let mut reverse_diff_str = reverse_patch.to_string();

        // On Windows, normalize the entire patch to use CRLF consistently
        // diffy generates patches with LF line endings, but Windows needs CRLF
        #[cfg(windows)]
        {
            // Always convert LF to CRLF on Windows
            // This replaces all standalone LF with CRLF
            reverse_diff_str = reverse_diff_str.replace("\r\n", "\n"); // First normalize any existing CRLF to LF
            reverse_diff_str = reverse_diff_str.replace("\n", "\r\n"); // Then convert all LF to CRLF
        }

        // If there are actual differences, save the reverse patch
        if !reverse_diff_str.is_empty()
            && reverse_diff_str != "--- original\n+++ modified\n"
            && !reverse_diff_str.is_empty()
            && reverse_diff_str != "--- original\r\n+++ modified\r\n"
        {
            // Replace diffy's generic headers with actual file paths
            // For reverse patch: current_path -> original_path
            let patch_with_paths =
                replace_patch_headers(&reverse_diff_str, &current_path, original_path);

            // Generate hash for this file's patch
            let relative_path = make_path_relative(original_path);
            let mut hasher = Sha256::new();
            hasher.update(relative_path.to_string_lossy().as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            // Save the individual patch
            let patch_filename = format!("{}.patch", hash);
            let patch_path = reverse_patches_dir.join(&patch_filename);
            fs::write(&patch_path, &patch_with_paths)?;

            // Update the match entries with the patch hash and file paths
            for match_hunk in &mut plan.matches {
                if match_hunk.file == *original_path {
                    match_hunk.original_file = Some(original_path.clone());
                    match_hunk.renamed_file = if current_path == *original_path {
                        None
                    } else {
                        Some(current_path.clone())
                    };
                    match_hunk.patch_hash = Some(hash.clone());
                }
            }
        }

        // Track any new directories created by renames
        if current_path != *original_path {
            if let Some(parent) = current_path.parent() {
                let mut dir = parent.to_path_buf();
                while !dir.exists() || !created_dirs.contains(&dir) {
                    if !dir.exists() {
                        created_dirs.push(dir.clone());
                    }
                    if let Some(parent) = dir.parent() {
                        dir = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    // Store created directories in the plan
    if !created_dirs.is_empty() {
        plan.created_directories = Some(created_dirs);
    }

    Ok(())
}

/// Convert an absolute path to a relative path from the current working directory
/// If the path is already relative, return it as-is
fn make_path_relative(path: &Path) -> PathBuf {
    // On Windows, strip the \\?\ prefix if present
    #[cfg(windows)]
    let path_buf: PathBuf;
    #[cfg(windows)]
    let path = {
        let path_str = path.to_string_lossy();
        // The actual string contains literal backslashes "\\?\", not the raw string r"\\?\"
        if path_str.starts_with("\\\\?\\") {
            path_buf = PathBuf::from(&path_str[4..]);
            &path_buf
        } else {
            path
        }
    };

    if path.is_relative() {
        return path.to_path_buf();
    }

    // Try to get the current directory and make the path relative to it
    match std::env::current_dir() {
        Ok(current_dir) => {
            // On Windows, also strip the \\?\ prefix from current_dir if present
            #[cfg(windows)]
            let current_dir_buf: PathBuf;
            #[cfg(windows)]
            let current_dir = {
                let dir_str = current_dir.to_string_lossy();
                if dir_str.starts_with("\\\\?\\") || dir_str.starts_with(r"\\?\") {
                    current_dir_buf = PathBuf::from(&dir_str[4..]);
                    &current_dir_buf
                } else {
                    &current_dir
                }
            };
            #[cfg(not(windows))]
            let current_dir = &current_dir;

            // Use pathdiff crate if available, or implement simple relative path logic
            match path.strip_prefix(current_dir) {
                Ok(relative) => relative.to_path_buf(),
                Err(_) => {
                    // If the path is not under the current directory, return the path without \\?\ prefix
                    // On Windows, we've already stripped the prefix above
                    path.to_path_buf()
                },
            }
        },
        Err(_) => {
            // If we can't get the current directory, return the path without \\?\ prefix
            // On Windows, we've already stripped the prefix above
            path.to_path_buf()
        },
    }
}

/// Split a string into lines while preserving whether each line had a trailing newline
/// Unlike `str::lines()`, this preserves the exact line ending structure
fn split_preserving_newlines(s: &str) -> Vec<&str> {
    if s.is_empty() {
        return vec![];
    }

    // Split on \n but keep track of the newlines
    let mut lines = Vec::new();
    let mut start = 0;

    for (i, ch) in s.char_indices() {
        if ch == '\n' {
            // Include the newline in the line
            lines.push(&s[start..=i]);
            start = i + 1;
        }
    }

    // Add the last segment if there is one (line without trailing newline)
    if start < s.len() {
        lines.push(&s[start..]);
    }

    lines
}

/// Replace diffy's generic patch headers with actual file paths
/// Converts absolute paths to relative paths for better portability
fn replace_patch_headers(patch_str: &str, from_path: &Path, to_path: &Path) -> String {
    let mut result = String::new();
    let lines = split_preserving_newlines(patch_str);

    // Convert paths to relative paths for better portability
    let from_relative = make_path_relative(from_path);
    let to_relative = make_path_relative(to_path);

    // On Windows, ensure we never have \\?\ in the path string
    // Check both escaped and unescaped versions to be safe
    // Convert paths to forward slashes for patch compatibility
    // The diffy library cannot parse patches with backslashes in filenames
    let from_str = {
        let s = from_relative.to_string_lossy();
        #[cfg(windows)]
        {
            let mut path_str = if s.starts_with("\\\\?\\") || s.starts_with(r"\\?\") {
                s[4..].to_string()
            } else {
                s.to_string()
            };
            // Replace backslashes with forward slashes for patch compatibility
            path_str = path_str.replace('\\', "/");
            path_str
        }
        #[cfg(not(windows))]
        {
            s.to_string()
        }
    };

    let to_str = {
        let s = to_relative.to_string_lossy();
        #[cfg(windows)]
        {
            let mut path_str = if s.starts_with("\\\\?\\") || s.starts_with(r"\\?\") {
                s[4..].to_string()
            } else {
                s.to_string()
            };
            // Replace backslashes with forward slashes for patch compatibility
            path_str = path_str.replace('\\', "/");
            path_str
        }
        #[cfg(not(windows))]
        {
            s.to_string()
        }
    };

    for line in lines {
        if line.starts_with("--- ") {
            // Replace "--- original" with actual from path (relative)
            // Preserve the original line ending
            write!(result, "--- {}", from_str).unwrap();
            // Add back the line ending from the original line
            if line.ends_with("\r\n") {
                result.push_str("\r\n");
            } else if line.ends_with('\n') {
                result.push('\n');
            }
        } else if line.starts_with("+++ ") {
            // Replace "+++ modified" with actual to path (relative)
            // Preserve the original line ending
            write!(result, "+++ {}", to_str).unwrap();
            // Add back the line ending from the original line
            if line.ends_with("\r\n") {
                result.push_str("\r\n");
            } else if line.ends_with('\n') {
                result.push('\n');
            }
        } else {
            // Keep all other lines as-is (including their newlines)
            result.push_str(line);
        }
    }

    result
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

/// Apply content edits to a file atomically without creating backup
fn apply_content_edits_with_content(
    path: &Path,
    original_content: &str,
    replacements: &[(String, String, usize, usize)],
    state: &mut ApplyState,
) -> Result<()> {
    state.log(&format!(
        "Applying {} edits to {}",
        replacements.len(),
        path.display()
    ))?;

    // Apply replacements (in reverse order to maintain positions)
    let mut modified = original_content.to_string();

    for (before, after, start, end) in replacements.iter().rev() {
        // Validate the replacement matches expected content
        let actual = &original_content[*start..*end];
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
    let temp_path = path.with_extension(format!("{}.renamify.tmp", std::process::id()));

    // Get original file permissions before writing
    let original_metadata = fs::metadata(path)?;
    let original_permissions = original_metadata.permissions();

    {
        let mut temp_file = File::create(&temp_path)?;
        temp_file.write_all(modified.as_bytes())?;
        temp_file.sync_all()?; // fsync
    }

    // Set the same permissions on the temp file before renaming
    fs::set_permissions(&temp_path, original_permissions)?;

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
    let test_lower = path.join(".renamify_case_test");
    let test_upper = path.join(".RENAMIFY_CASE_TEST");

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
        let temp_name = from.with_extension(format!("{}.renamify.tmp", std::process::id()));

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

/// Apply a renaming plan
#[allow(clippy::too_many_lines)]
pub fn apply_plan(plan: &mut Plan, options: &ApplyOptions) -> Result<()> {
    let mut state = ApplyState::new(plan.id.clone(), options.log_file.clone())?;

    state.log(&format!("Starting apply for plan {}", plan.id))?;
    state.log(&format!("Options: {:?}", options))?;

    // Only backup files that will be renamed (content changes use diffs)
    let mut files_to_backup = HashSet::new();

    // Files and directories to be renamed
    for rename in &plan.paths {
        files_to_backup.insert(&rename.path);
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

    // STEP 1: Store original content BEFORE any changes for diff generation
    let mut original_contents: HashMap<PathBuf, String> = HashMap::new();
    if options.create_backups {
        let mut files_with_content_changes: HashSet<&Path> = HashSet::new();
        for hunk in &plan.matches {
            files_with_content_changes.insert(&hunk.file);
        }

        for file_path in files_with_content_changes {
            if let Ok(content) = fs::read_to_string(file_path) {
                original_contents.insert(file_path.to_path_buf(), content);
            }
        }
    }

    // STEP 2: Apply content edits FIRST (before any renames)
    // Group content edits by file
    let mut edits_by_file: BTreeMap<PathBuf, Vec<_>> = BTreeMap::new();

    // Debug Train-Case patterns in plan
    if std::env::var("RENAMIFY_DEBUG_TRAIN_CASE").is_ok() {
        eprintln!("\n=== Plan matches for Train-Case patterns ===");
        for hunk in &plan.matches {
            if hunk.content.contains('-')
                && hunk.content.chars().next().is_some_and(char::is_uppercase)
            {
                eprintln!("  File: {}", hunk.file.display());
                eprintln!("    Before: '{}'", hunk.content);
                eprintln!("    After: '{}'", hunk.replace);
                eprintln!("    Position: [{}, {}]", hunk.start, hunk.end);
            }
        }
    }

    for hunk in &plan.matches {
        edits_by_file.entry(hunk.file.clone()).or_default().push((
            hunk.content.clone(),
            hunk.replace.clone(),
            hunk.start,
            hunk.end,
        ));
    }

    // Apply content edits to files at their ORIGINAL locations (before renames)
    for (path, edits) in edits_by_file {
        // Read the file content
        let file_content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        if let Err(e) = apply_content_edits_with_content(&path, &file_content, &edits, &mut state) {
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

    // STEP 3: Apply renames AFTER content edits
    // Sort renames: directories first (shallowest to deepest), then files (deepest to shallowest)
    // This ensures parent directories are renamed before their contents
    let mut renames = plan.paths.clone();
    renames.sort_by(|a, b| {
        use crate::scanner::RenameKind;
        match (&a.kind, &b.kind) {
            (RenameKind::Dir, RenameKind::File) => std::cmp::Ordering::Less,
            (RenameKind::File, RenameKind::Dir) => std::cmp::Ordering::Greater,
            (RenameKind::Dir, RenameKind::Dir) => {
                // For directories: shallowest first
                a.path
                    .components()
                    .count()
                    .cmp(&b.path.components().count())
            },
            (RenameKind::File, RenameKind::File) => {
                // For files: deepest first
                b.path
                    .components()
                    .count()
                    .cmp(&a.path.components().count())
            },
        }
    });

    for mut rename in renames {
        let is_dir = rename.kind == crate::scanner::RenameKind::Dir;

        // Check if this rename's source path has been affected by a previous directory rename
        // This handles the case where a file inside a renamed directory also needs to be renamed
        let mut adjusted_from = rename.path.clone();
        let mut adjusted_to = rename.new_path.clone();

        // Clone to avoid borrow checker issues
        let previous_renames = state.renames_performed.clone();

        for (prev_from, prev_to) in &previous_renames {
            // Check if the source path needs adjustment
            if let Ok(relative) = rename.path.strip_prefix(prev_from) {
                // This rename's source is inside a directory that was already renamed
                adjusted_from = prev_to.join(relative);
                state.log(&format!(
                    "Adjusted rename source: {} -> {} (due to parent directory rename)",
                    rename.path.display(),
                    adjusted_from.display()
                ))?;
            }

            // Check if the destination path needs adjustment
            if let Ok(relative) = rename.new_path.strip_prefix(prev_from) {
                // This rename's destination is inside a directory that was already renamed
                adjusted_to = prev_to.join(relative);
                state.log(&format!(
                    "Adjusted rename destination: {} -> {} (due to parent directory rename)",
                    rename.new_path.display(),
                    adjusted_to.display()
                ))?;
            }
        }

        if let Err(e) = perform_rename(&adjusted_from, &adjusted_to, is_dir, &mut state) {
            state.log(&format!("Error performing rename: {}", e))?;

            if options.atomic {
                rollback(&mut state)?;
            }

            return Err(e);
        }

        // Override the recorded rename to use the ORIGINAL paths for tracking
        // The perform_rename function records adjusted_from -> adjusted_to
        // But we need original_from -> final_to for proper tracking
        if adjusted_from != rename.path || adjusted_to != rename.new_path {
            // Remove the last entry (which has adjusted paths)
            state.renames_performed.pop();
            // Add the correct entry with original from path and adjusted to path
            state
                .renames_performed
                .push((rename.path.clone(), adjusted_to.clone()));
        }
    }

    // STEP 4: Generate comprehensive patch after all changes are complete
    if options.create_backups {
        state.log("Creating comprehensive patch backup")?;

        // Generate individual reverse patch files
        if let Err(e) = generate_reverse_patches(plan, options, &state, &original_contents) {
            state.log(&format!("Failed to create comprehensive patch: {}", e))?;
            if !options.force {
                return Err(e);
            }
        }

        state.log("Comprehensive patch created successfully")?;
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
            "renamify: rename {} -> {} (#{}))",
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

    // Determine the .renamify directory location
    // If backup_dir is .renamify/backups/plan_id, we want .renamify
    // If backup_dir is .renamify/backups, we want .renamify
    let renamify_dir = if options.backup_dir.ends_with(&plan.id) {
        // backup_dir is .renamify/backups/plan_id
        options.backup_dir
            .parent() // .renamify/backups
            .and_then(|p| p.parent()) // .renamify
            .unwrap_or_else(|| Path::new(".renamify"))
    } else {
        // backup_dir is .renamify/backups
        options.backup_dir
            .parent() // .renamify
            .unwrap_or_else(|| Path::new(".renamify"))
    };

    let mut history = History::load(renamify_dir)?;
    history.add_entry(history_entry)?;

    // Store the complete plan for redo functionality
    let plans_dir = renamify_dir.join("plans");
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
            paths: vec![],
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
    #[serial]
    fn test_apply_creates_correct_history_entry() {
        // Test that apply creates history entries with correct backup paths
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn old_name() {}").unwrap();

        let mut plan = Plan {
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
                content: "old_name".to_string(),
                replace: "new_name".to_string(),
                start: 3,
                end: 11,
                line_before: Some("fn old_name() {}".to_string()),
                line_after: Some("fn new_name() {}".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            }],
            paths: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant: HashMap::new(),
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let options = ApplyOptions {
            backup_dir: temp_dir.path().join(".renamify/backups"),
            create_backups: true,
            ..Default::default()
        };

        // Apply the plan
        apply_plan(&mut plan, &options).unwrap();

        // Check the file was modified
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "fn new_name() {}");

        // Check reverse patches directory was created
        let reverse_patches_dir = options
            .backup_dir
            .join("test_plan_456")
            .join("reverse_patches");
        assert!(
            reverse_patches_dir.exists(),
            "Reverse patches directory should exist at {:?}",
            reverse_patches_dir
        );

        // Check that at least one patch file was created
        let entries: Vec<_> = fs::read_dir(&reverse_patches_dir)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(!entries.is_empty(), "Should have at least one patch file");

        // Read the first patch file and check it contains the changes
        let patch_file = entries[0].path();
        let patch_content = fs::read_to_string(&patch_file).unwrap();
        assert!(patch_content.contains("old_name"));
        assert!(patch_content.contains("new_name"));

        // Load history and check the entry
        let history = History::load(temp_dir.path().join(".renamify").as_path()).unwrap();
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

        // Apply edits with pre-read content
        let original_content = fs::read_to_string(&test_file).unwrap();
        apply_content_edits_with_content(&test_file, &original_content, &replacements, &mut state)
            .unwrap();

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

        let mut plan = Plan {
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
                content: "old".to_string(),
                replace: "new".to_string(),
                start: 3,
                end: 6,
                line_before: Some("fn old() {}".to_string()),
                line_after: Some("fn new() {}".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            }],
            paths: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant: HashMap::new(),
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let options = ApplyOptions {
            backup_dir: temp_dir.path().join(".renamify/backups"),
            create_backups: true,
            ..Default::default()
        };

        // Apply should work
        apply_plan(&mut plan, &options).unwrap();

        // Load history and verify affected_files is populated
        let history = History::load(temp_dir.path().join(".renamify").as_path()).unwrap();
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

    #[test]
    #[cfg(windows)]
    fn test_make_path_relative_windows_long_path() {
        // Test with Windows long path prefix
        let long_path = Path::new(r"\\?\C:\Users\test\project\src\file.rs");
        let relative = make_path_relative(long_path);

        // Should strip the \\?\ prefix
        assert!(!relative.to_string_lossy().starts_with("\\\\?\\"));

        // Test with regular Windows path
        let normal_path = Path::new(r"C:\Users\test\project\src\file.rs");
        let relative_normal = make_path_relative(normal_path);

        // Should not have \\?\ prefix
        assert!(!relative_normal.to_string_lossy().starts_with("\\\\?\\"));
    }

    #[test]
    fn test_replace_patch_headers() {
        use std::path::PathBuf;

        let diffy_patch = "--- original\n+++ modified\n@@ -1,3 +1,3 @@\n-old content\n+new content\n\\ No newline at end of file\n";

        let result = replace_patch_headers(
            diffy_patch,
            &PathBuf::from("/path/to/old.rs"),
            &PathBuf::from("/path/to/new.rs"),
        );

        // Since these paths are not under the current directory, they should remain absolute
        // The input patch has a trailing newline, so the output should too
        let expected = "--- /path/to/old.rs
+++ /path/to/new.rs
@@ -1,3 +1,3 @@
-old content
+new content
\\ No newline at end of file
";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_patch_headers_with_newline_markers() {
        use std::path::PathBuf;

        let diffy_patch = "--- original\n+++ modified\n@@ -1 +1 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n";

        let result = replace_patch_headers(
            diffy_patch,
            &PathBuf::from("file.txt"),
            &PathBuf::from("file.txt"),
        );

        // The input patch has a trailing newline, so the output should too
        let expected = "--- file.txt
+++ file.txt
@@ -1 +1 @@
-old
\\ No newline at end of file
+new
\\ No newline at end of file
";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_patch_headers_multiple_hunks() {
        use std::path::PathBuf;

        let diffy_patch = "--- original\n+++ modified\n@@ -1,2 +1,2 @@\n-line1\n-line2\n+new1\n+new2\n@@ -5,1 +5,1 @@\n-line5\n+new5\n";

        let result = replace_patch_headers(
            diffy_patch,
            &PathBuf::from("src/lib.rs"),
            &PathBuf::from("src/lib.rs"),
        );

        // The input patch has a trailing newline, so the output should too
        let expected = "--- src/lib.rs
+++ src/lib.rs
@@ -1,2 +1,2 @@
-line1
-line2
+new1
+new2
@@ -5,1 +5,1 @@
-line5
+new5
";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_make_path_relative() {
        use std::env;

        // Test relative paths remain relative
        let relative = PathBuf::from("src/lib.rs");
        assert_eq!(make_path_relative(&relative), PathBuf::from("src/lib.rs"));

        // Test absolute path conversion
        if let Ok(current_dir) = env::current_dir() {
            let test_file = current_dir.join("test.rs");
            let relative_result = make_path_relative(&test_file);
            assert_eq!(relative_result, PathBuf::from("test.rs"));

            let nested_file = current_dir.join("src").join("lib.rs");
            let relative_nested = make_path_relative(&nested_file);
            assert_eq!(relative_nested, PathBuf::from("src/lib.rs"));
        }
    }

    #[test]
    fn test_replace_patch_headers_converts_absolute_to_relative() {
        use std::env;

        let diffy_patch = "--- original\n+++ modified\n@@ -1 +1 @@\n-old\n+new\n";

        // Test with absolute paths under current directory
        if let Ok(current_dir) = env::current_dir() {
            let from_path = current_dir.join("src").join("old.rs");
            let to_path = current_dir.join("src").join("new.rs");

            let result = replace_patch_headers(diffy_patch, &from_path, &to_path);

            // The input patch has a trailing newline, so the output should too
            let expected = "--- src/old.rs
+++ src/new.rs
@@ -1 +1 @@
-old
+new
";

            assert_eq!(result, expected);
        }
    }
}
