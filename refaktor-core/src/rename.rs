use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::scanner::{PlanOptions, Rename, RenameKind};

/// Windows reserved filenames that cannot be used
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

#[derive(Debug, Clone)]
pub struct RenameConflict {
    pub sources: Vec<PathBuf>,
    pub target: PathBuf,
    pub kind: ConflictKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictKind {
    /// Multiple sources map to the same target
    MultipleToOne,
    /// Case-only change on case-insensitive filesystem
    CaseInsensitive,
    /// Target is a Windows reserved name
    WindowsReserved,
}

#[derive(Debug, Clone)]
pub struct RenamePlan {
    pub renames: Vec<Rename>,
    pub conflicts: Vec<RenameConflict>,
    pub case_insensitive_fs: bool,
    pub requires_staging: bool,
}

/// Check if the filesystem at the given path is case-insensitive
pub fn detect_case_insensitive_fs(path: &Path) -> bool {
    // Try to create a temporary directory in the target location
    let temp_dir = match TempDir::new_in(path) {
        Ok(dir) => dir,
        Err(_) => return false, // Assume case-sensitive if we can't test
    };

    let test_file_lower = temp_dir.path().join("test_case_a");
    let test_file_upper = temp_dir.path().join("test_case_A");

    // Create file with lowercase name
    if fs::write(&test_file_lower, b"test").is_err() {
        return false;
    }

    // Try to access it with uppercase name
    // On case-insensitive FS, this will succeed
    fs::metadata(&test_file_upper).is_ok()
}

/// Check if a filename is a Windows reserved name
/// Always checks regardless of platform to ensure cross-platform compatibility
pub fn is_windows_reserved(name: &str) -> bool {
    // Get the base name without extension
    let base = name
        .split('.')
        .next()
        .unwrap_or(name)
        .to_uppercase();

    WINDOWS_RESERVED.contains(&base.as_str())
}

/// Check if a filename is a Windows reserved name (only on Windows)
fn is_windows_reserved_on_windows(name: &str) -> bool {
    if !cfg!(windows) {
        return false;
    }
    is_windows_reserved(name)
}

/// Plan renames for files and directories based on variant mapping
pub fn plan_renames_with_conflicts(
    root: &Path,
    mapping: &BTreeMap<String, String>,
    options: &PlanOptions,
) -> Result<RenamePlan> {
    let mut collected_renames = Vec::new();
    let case_insensitive_fs = detect_case_insensitive_fs(root);

    // Collect all potential renames
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip if not renaming files/dirs based on options
        if path.is_dir() && !options.rename_dirs {
            continue;
        }
        if path.is_file() && !options.rename_files {
            continue;
        }

        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            
            // Check each variant mapping
            for (old, new) in mapping {
                if file_name_str.contains(old) {
                    let new_name = file_name_str.replace(old, new);
                    let new_path = path.with_file_name(&new_name);
                    
                    let kind = if path.is_dir() {
                        RenameKind::Dir
                    } else {
                        RenameKind::File
                    };
                    
                    collected_renames.push(Rename {
                        from: path.to_path_buf(),
                        to: new_path,
                        kind,
                    });
                    break; // Only apply first matching variant
                }
            }
        }
    }

    // Sort renames: directories by depth (deepest first), then files
    collected_renames.sort_by(|a, b| {
        let a_is_dir = matches!(a.kind, RenameKind::Dir);
        let b_is_dir = matches!(b.kind, RenameKind::Dir);
        let a_depth = a.from.components().count();
        let b_depth = b.from.components().count();
        
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => b_depth.cmp(&a_depth), // Deeper dirs first
            (false, false) => a.from.cmp(&b.from), // Stable sort for files
        }
    });

    // Detect conflicts
    let mut conflicts = Vec::new();
    let mut target_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut requires_staging = false;

    for rename in &collected_renames {
        // Check for Windows reserved names
        if let Some(file_name) = rename.to.file_name() {
            if is_windows_reserved(&file_name.to_string_lossy()) {
                conflicts.push(RenameConflict {
                    sources: vec![rename.from.clone()],
                    target: rename.to.clone(),
                    kind: ConflictKind::WindowsReserved,
                });
                continue;
            }
        }

        // Check for case-only changes on case-insensitive filesystem
        if case_insensitive_fs {
            let from_lower = rename.from.to_string_lossy().to_lowercase();
            let to_lower = rename.to.to_string_lossy().to_lowercase();
            
            if from_lower != to_lower {
                // Not a case-only change, it's a real rename
            } else if rename.from != rename.to {
                // Case-only change detected
                requires_staging = true;
                conflicts.push(RenameConflict {
                    sources: vec![rename.from.clone()],
                    target: rename.to.clone(),
                    kind: ConflictKind::CaseInsensitive,
                });
            }
        }

        // Track targets for collision detection
        target_map
            .entry(rename.to.clone())
            .or_default()
            .push(rename.from.clone());
    }

    // Find multiple-to-one conflicts
    for (target, sources) in target_map {
        if sources.len() > 1 {
            conflicts.push(RenameConflict {
                sources,
                target,
                kind: ConflictKind::MultipleToOne,
            });
        }
    }

    // Filter out renames that have conflicts (except case-insensitive ones which can be staged)
    let conflict_targets: HashSet<PathBuf> = conflicts
        .iter()
        .filter(|c| c.kind != ConflictKind::CaseInsensitive)
        .map(|c| c.target.clone())
        .collect();

    let valid_renames: Vec<Rename> = collected_renames
        .into_iter()
        .filter(|r| !conflict_targets.contains(&r.to))
        .collect();

    Ok(RenamePlan {
        renames: valid_renames,
        conflicts,
        case_insensitive_fs,
        requires_staging,
    })
}

/// Compatibility wrapper for existing API
pub fn plan_renames(
    root: &Path,
    mapping: &BTreeMap<String, String>,
    options: &PlanOptions,
) -> Result<Vec<Rename>> {
    let plan = plan_renames_with_conflicts(root, mapping, options)?;
    
    if !plan.conflicts.is_empty() {
        let conflict_count = plan.conflicts.len();
        let conflict_msg = plan
            .conflicts
            .iter()
            .map(|c| format!("{:?}: {:?} -> {:?}", c.kind, c.sources, c.target))
            .collect::<Vec<_>>()
            .join("\n");
        
        return Err(anyhow!(
            "Found {} rename conflicts:\n{}",
            conflict_count,
            conflict_msg
        ));
    }
    
    Ok(plan.renames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_windows_reserved_names() {
        // The public function always checks regardless of platform
        assert!(is_windows_reserved("CON"));
        assert!(is_windows_reserved("con"));
        assert!(is_windows_reserved("CON.txt"));
        assert!(is_windows_reserved("nul.log"));
        assert!(!is_windows_reserved("CONSOLE"));
        assert!(!is_windows_reserved("my_con"));
        
        // The internal function only returns true on Windows
        if cfg!(windows) {
            assert!(is_windows_reserved_on_windows("CON"));
        } else {
            assert!(!is_windows_reserved_on_windows("CON"));
        }
    }

    #[test]
    fn test_case_insensitive_fs_detection() {
        let temp_dir = TempDir::new().unwrap();
        let is_case_insensitive = detect_case_insensitive_fs(temp_dir.path());
        
        // Result depends on the actual filesystem
        // On macOS default FS, this is typically true
        // On Linux, this is typically false
        // We just verify it doesn't panic
        println!("Filesystem case-insensitive: {}", is_case_insensitive);
    }

    #[test]
    fn test_rename_depth_sorting() {
        let renames = vec![
            Rename {
                from: PathBuf::from("/a/b/c/file.txt"),
                to: PathBuf::from("/a/b/c/new.txt"),
                kind: RenameKind::File,
            },
            Rename {
                from: PathBuf::from("/a/dir"),
                to: PathBuf::from("/a/newdir"),
                kind: RenameKind::Dir,
            },
            Rename {
                from: PathBuf::from("/a/b/deep_dir"),
                to: PathBuf::from("/a/b/new_deep"),
                kind: RenameKind::Dir,
            },
            Rename {
                from: PathBuf::from("/file.txt"),
                to: PathBuf::from("/new.txt"),
                kind: RenameKind::File,
            },
        ];

        let mut sorted = renames.clone();
        sorted.sort_by(|a, b| {
            let a_is_dir = matches!(a.kind, RenameKind::Dir);
            let b_is_dir = matches!(b.kind, RenameKind::Dir);
            let a_depth = a.from.components().count();
            let b_depth = b.from.components().count();
            
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => b_depth.cmp(&a_depth),
                (false, false) => a.from.cmp(&b.from),
            }
        });

        // Verify directories come first, deepest first
        assert!(matches!(sorted[0].kind, RenameKind::Dir));
        assert_eq!(sorted[0].from, PathBuf::from("/a/b/deep_dir"));
        assert!(matches!(sorted[1].kind, RenameKind::Dir));
        assert_eq!(sorted[1].from, PathBuf::from("/a/dir"));
        
        // Then files
        assert!(matches!(sorted[2].kind, RenameKind::File));
        assert!(matches!(sorted[3].kind, RenameKind::File));
    }

    #[test]
    fn test_collision_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        std::fs::write(temp_dir.path().join("old_name1.txt"), "test1").unwrap();
        std::fs::write(temp_dir.path().join("old_name2.txt"), "test2").unwrap();
        
        let mut mapping = BTreeMap::new();
        // Both files will map to "new_name.txt"
        mapping.insert("old_name1".to_string(), "new_name".to_string());
        mapping.insert("old_name2".to_string(), "new_name".to_string());
        
        let opts = PlanOptions::default();
        let plan = plan_renames_with_conflicts(temp_dir.path(), &mapping, &opts).unwrap();
        
        // Should detect collision
        assert!(!plan.conflicts.is_empty());
        let collision = plan.conflicts.iter()
            .find(|c| c.kind == ConflictKind::MultipleToOne);
        assert!(collision.is_some());
    }

    #[test]
    fn test_empty_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let mapping = BTreeMap::new();
        let opts = PlanOptions::default();
        
        let plan = plan_renames_with_conflicts(temp_dir.path(), &mapping, &opts).unwrap();
        assert!(plan.renames.is_empty());
        assert!(plan.conflicts.is_empty());
    }

    #[test]
    fn test_file_only_rename_option() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a directory and a file
        let dir_path = temp_dir.path().join("old_dir");
        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(temp_dir.path().join("old_file.txt"), "test").unwrap();
        
        let mut mapping = BTreeMap::new();
        mapping.insert("old".to_string(), "new".to_string());
        
        let mut opts = PlanOptions::default();
        opts.rename_files = true;
        opts.rename_dirs = false;
        
        let plan = plan_renames_with_conflicts(temp_dir.path(), &mapping, &opts).unwrap();
        
        // Should only rename the file, not the directory
        assert_eq!(plan.renames.len(), 1);
        assert!(plan.renames[0].from.to_string_lossy().contains("old_file"));
    }
}