use crate::scanner::build_globset;
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::ambiguity::{AmbiguityContext, AmbiguityResolver};
use crate::case_constraints::filter_compatible_styles;
use crate::case_model::{parse_to_tokens, to_style, Style};
use crate::scanner::{PlanOptions, Rename, RenameKind};

/// Normalize a path by removing Windows long path prefix if present
fn normalize_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("\\\\?\\") {
            PathBuf::from(&path_str[4..])
        } else {
            path.to_path_buf()
        }
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

/// Windows reserved filenames that cannot be used
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Determine the best replacement for a filename using the ambiguity resolver
fn determine_filename_replacement(
    filename: &str,
    _search: &str,
    replace: &str,
    mapping: &BTreeMap<String, String>,
    ambiguity_resolver: &AmbiguityResolver,
) -> Option<String> {
    // Check if the filename contains the search pattern
    // We need to find which variant from the mapping matches this filename
    let mut matching_variant = None;
    for old_variant in mapping.keys() {
        if filename.contains(old_variant) {
            matching_variant = Some(old_variant.clone());
            break;
        }
    }

    let matching_variant = matching_variant?;

    // Check if this is an ambiguous identifier
    if !crate::ambiguity::is_ambiguous(&matching_variant, &Style::all_styles()) {
        // Not ambiguous - just use the variant map directly
        let new_variant = mapping.get(&matching_variant)?;
        return Some(filename.replace(&matching_variant, new_variant));
    }

    // For ambiguous identifiers, use the ambiguity resolver
    let replacement_possible_styles = filter_compatible_styles(replace, &Style::all_styles());

    // Create a minimal ambiguity context for the filename
    // We don't have file content or line content for filenames
    let ambiguity_context = AmbiguityContext {
        file_path: None,
        file_content: None,
        line_content: None,
        match_position: None,
        project_root: None,
    };

    // Resolve what style should be used based on context
    let resolved = ambiguity_resolver.resolve_with_styles(
        &matching_variant,
        replace,
        &ambiguity_context,
        Some(&replacement_possible_styles),
    );

    // Generate the replacement in the resolved style
    let replacement_tokens = parse_to_tokens(replace);
    let styled_replacement = to_style(&replacement_tokens, resolved.style);

    // Replace the variant in the filename with the styled replacement
    Some(filename.replace(&matching_variant, &styled_replacement))
}

#[derive(Debug, Clone)]
pub struct RenameConflict {
    pub sources: Vec<PathBuf>,
    pub target: PathBuf,
    pub kind: ConflictKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    let Ok(temp_dir) = TempDir::new_in(path) else {
        return false; // Assume case-sensitive if we can't test
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
    let base = name.split('.').next().unwrap_or(name).to_uppercase();

    WINDOWS_RESERVED.contains(&base.as_str())
}

/// Check if a filename is a Windows reserved name (only on Windows)
#[cfg(test)]
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
    plan_renames_with_conflicts_and_params(root, mapping, options, "", "")
}

/// Plan renames for files and directories based on variant mapping, with search/replace params
fn plan_renames_with_conflicts_and_params(
    root: &Path,
    mapping: &BTreeMap<String, String>,
    options: &PlanOptions,
    search: &str,
    replace: &str,
) -> Result<RenamePlan> {
    let mut collected_renames = Vec::new();
    let case_insensitive_fs = detect_case_insensitive_fs(root);

    // Create ambiguity resolver for intelligent style selection
    let ambiguity_resolver = AmbiguityResolver::new();

    // Build globsets for include/exclude filtering using shared logic
    let include_globs = build_globset(&options.includes)?;
    let exclude_globs = build_globset(&options.excludes)?;

    // Use shared walker configuration
    let walker = crate::configure_walker(&[root.to_path_buf()], options).build();

    // Collect all potential renames
    for entry in walker {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        // Apply include/exclude filters (use relative path for matching)
        let relative_path = path.strip_prefix(root).unwrap_or(path);

        if let Some(ref includes) = include_globs {
            if !includes.is_match(relative_path) {
                continue;
            }
        }

        if let Some(ref excludes) = exclude_globs {
            if excludes.is_match(relative_path) {
                continue;
            }
        }

        // Get file type from entry metadata
        let Some(file_type) = entry.file_type() else {
            continue;
        };

        // Skip if not renaming files/dirs based on options
        if file_type.is_dir() && !options.rename_dirs {
            continue;
        }
        if file_type.is_file() && !options.rename_files {
            continue;
        }

        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();

            // Use the ambiguity resolver to determine the best replacement
            // If search/replace are provided, use them; otherwise fall back to simple replacement
            if !search.is_empty() && !replace.is_empty() {
                // First, find which variant from the mapping matches this filename
                let matching_variant = mapping
                    .keys()
                    .find(|old_variant| file_name_str.contains(*old_variant));

                if let Some(old_variant) = matching_variant {
                    if let Some(new_variant) = mapping.get(old_variant) {
                        if let Some(mut new_name) = determine_filename_replacement(
                            &file_name_str,
                            search,
                            replace,
                            mapping,
                            &ambiguity_resolver,
                        ) {
                            let mut coercion_applied = None;

                            // Apply coercion if enabled for contextual separator coercion
                            // (e.g., preserving Pascal case inside camel case containers)
                            // Use the specific variant that matched, not the original search/replace
                            if options.coerce_separators == crate::scanner::CoercionMode::Auto {
                                if let Some((coerced, reason)) = crate::coercion::apply_coercion(
                                    &file_name_str,
                                    old_variant,
                                    new_variant,
                                ) {
                                    new_name = coerced;
                                    coercion_applied = Some(reason);
                                }
                            }

                            if new_name != file_name_str {
                                let new_path = path.with_file_name(&new_name);

                                let kind = if file_type.is_dir() {
                                    RenameKind::Dir
                                } else {
                                    RenameKind::File
                                };

                                collected_renames.push(Rename {
                                    path: normalize_path(path),
                                    new_path: normalize_path(&new_path),
                                    kind,
                                    coercion_applied,
                                });
                            }
                        }
                    }
                }
            } else {
                // Fallback to simple replacement for backward compatibility
                // (when search/replace are not provided)
                for (old, new) in mapping {
                    if file_name_str.contains(old) {
                        let mut new_name = file_name_str.replace(old, new);
                        let mut coercion_applied = None;

                        // Apply coercion if enabled
                        if options.coerce_separators == crate::scanner::CoercionMode::Auto {
                            if let Some((coerced, reason)) =
                                crate::coercion::apply_coercion(&file_name_str, old, new)
                            {
                                new_name = coerced;
                                coercion_applied = Some(reason);
                            }
                        }

                        let new_path = path.with_file_name(&new_name);

                        let kind = if file_type.is_dir() {
                            RenameKind::Dir
                        } else {
                            RenameKind::File
                        };

                        collected_renames.push(Rename {
                            path: normalize_path(path),
                            new_path: normalize_path(&new_path),
                            kind,
                            coercion_applied,
                        });
                        break; // Only apply first matching variant
                    }
                }
            }
        }
    }

    // Filter out root directory renames unless explicitly allowed
    if !options.rename_root {
        collected_renames.retain(|rename| {
            // Check if this rename is for the root directory (current working directory)
            if let Ok(current_dir) = std::env::current_dir() {
                rename.path != current_dir
            } else {
                // If we can't get current dir, keep the rename (safe default)
                true
            }
        });
    }

    // Sort renames: directories by depth (deepest first), then files
    collected_renames.sort_by(|a, b| {
        let a_is_dir = matches!(a.kind, RenameKind::Dir);
        let b_is_dir = matches!(b.kind, RenameKind::Dir);
        let a_depth = a.path.components().count();
        let b_depth = b.path.components().count();

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => b_depth.cmp(&a_depth), // Deeper dirs first
            (false, false) => a.path.cmp(&b.path), // Stable sort for files
        }
    });

    // Detect conflicts
    let mut conflicts = Vec::new();
    let mut target_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut requires_staging = false;

    for rename in &collected_renames {
        // Check for Windows reserved names
        if let Some(file_name) = rename.new_path.file_name() {
            if is_windows_reserved(&file_name.to_string_lossy()) {
                conflicts.push(RenameConflict {
                    sources: vec![rename.path.clone()],
                    target: rename.new_path.clone(),
                    kind: ConflictKind::WindowsReserved,
                });
                continue;
            }
        }

        // Check for case-only changes on case-insensitive filesystem
        if case_insensitive_fs {
            let from_lower = rename.path.to_string_lossy().to_lowercase();
            let to_lower = rename.new_path.to_string_lossy().to_lowercase();

            if from_lower != to_lower {
                // Not a case-only change, it's a real rename
            } else if rename.path != rename.new_path {
                // Case-only change detected
                requires_staging = true;
                conflicts.push(RenameConflict {
                    sources: vec![rename.path.clone()],
                    target: rename.new_path.clone(),
                    kind: ConflictKind::CaseInsensitive,
                });
            }
        }

        // Track targets for collision detection
        target_map
            .entry(rename.new_path.clone())
            .or_default()
            .push(rename.path.clone());
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
        .filter(|r| !conflict_targets.contains(&r.new_path))
        .collect();

    Ok(RenamePlan {
        renames: valid_renames,
        conflicts,
        case_insensitive_fs,
        requires_staging,
    })
}

/// Plan renames with search and replace for ambiguity resolution
pub fn plan_renames_with_search(
    root: &Path,
    mapping: &BTreeMap<String, String>,
    options: &PlanOptions,
    search: &str,
    replace: &str,
) -> Result<Vec<Rename>> {
    let plan = plan_renames_with_conflicts_and_params(root, mapping, options, search, replace)?;

    if !plan.conflicts.is_empty() {
        let conflict_count = plan.conflicts.len();
        let conflict_msg = plan
            .conflicts
            .iter()
            .map(|c| format!("{:?}: {:?} -> {}", c.kind, c.sources, c.target.display()))
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

/// Compatibility wrapper for existing API (without search/replace)
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
            .map(|c| format!("{:?}: {:?} -> {}", c.kind, c.sources, c.target.display()))
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

        // Result depends on the actual filesystem
        // On macOS default FS, this is typically true
        // On Linux, this is typically false
        // We just verify it doesn't panic and returns a boolean
        detect_case_insensitive_fs(temp_dir.path());
    }

    #[test]
    fn test_rename_depth_sorting() {
        let renames = vec![
            Rename {
                path: PathBuf::from("/a/b/c/file.txt"),
                new_path: PathBuf::from("/a/b/c/new.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
            Rename {
                path: PathBuf::from("/a/dir"),
                new_path: PathBuf::from("/a/newdir"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            },
            Rename {
                path: PathBuf::from("/a/b/deep_dir"),
                new_path: PathBuf::from("/a/b/new_deep"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            },
            Rename {
                path: PathBuf::from("/file.txt"),
                new_path: PathBuf::from("/new.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
        ];

        let mut sorted = renames;
        sorted.sort_by(|a, b| {
            let a_is_dir = matches!(a.kind, RenameKind::Dir);
            let b_is_dir = matches!(b.kind, RenameKind::Dir);
            let a_depth = a.path.components().count();
            let b_depth = b.path.components().count();

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => b_depth.cmp(&a_depth),
                (false, false) => a.path.cmp(&b.path),
            }
        });

        // Verify directories come first, deepest first
        assert!(matches!(sorted[0].kind, RenameKind::Dir));
        assert_eq!(sorted[0].path, PathBuf::from("/a/b/deep_dir"));
        assert!(matches!(sorted[1].kind, RenameKind::Dir));
        assert_eq!(sorted[1].path, PathBuf::from("/a/dir"));

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
        let collision = plan
            .conflicts
            .iter()
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

        let opts = PlanOptions {
            rename_files: true,
            rename_dirs: false,
            ..Default::default()
        };

        let plan = plan_renames_with_conflicts(temp_dir.path(), &mapping, &opts).unwrap();

        // Should only rename the file, not the directory
        assert_eq!(plan.renames.len(), 1);
        assert!(plan.renames[0].path.to_string_lossy().contains("old_file"));
    }

    #[test]
    fn test_respect_gitignore() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize as a git repository so .gitignore works
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init git repo");

        // Create a .gitignore file
        std::fs::write(
            temp_dir.path().join(".gitignore"),
            "target/\n*.tmp\nbuild/\n",
        )
        .unwrap();

        // Create directories and files
        let target_dir = temp_dir.path().join("target");
        std::fs::create_dir(&target_dir).unwrap();
        std::fs::write(target_dir.join("old_name.txt"), "ignored").unwrap();

        let build_dir = temp_dir.path().join("build");
        std::fs::create_dir(&build_dir).unwrap();
        std::fs::write(build_dir.join("old_name.exe"), "ignored").unwrap();

        // Files that should be ignored by pattern
        std::fs::write(temp_dir.path().join("old_name.tmp"), "ignored").unwrap();

        // Files that should NOT be ignored
        std::fs::write(temp_dir.path().join("old_name.txt"), "not ignored").unwrap();
        std::fs::write(temp_dir.path().join("old_name.rs"), "not ignored").unwrap();

        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::write(src_dir.join("old_name.rs"), "not ignored").unwrap();

        let mut mapping = BTreeMap::new();
        mapping.insert("old_name".to_string(), "new_name".to_string());

        let opts = PlanOptions {
            unrestricted_level: 0, // Default: respect all ignore files
            ..Default::default()
        };

        let plan = plan_renames_with_conflicts(temp_dir.path(), &mapping, &opts).unwrap();

        // Should only include non-ignored files
        let rename_paths: Vec<String> = plan
            .renames
            .iter()
            .map(|r| {
                let path_str = r
                    .path
                    .strip_prefix(temp_dir.path())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                // Normalize to forward slashes for consistent comparison
                if cfg!(windows) {
                    path_str.replace('\\', "/")
                } else {
                    path_str
                }
            })
            .collect();

        // Should NOT rename files in target/ or build/ directories
        assert!(
            !rename_paths.iter().any(|p| p.contains("target/")),
            "Should NOT find target/ files when gitignore is enabled"
        );
        assert!(
            !rename_paths.iter().any(|p| p.contains("build/")),
            "Should NOT find build/ files when gitignore is enabled"
        );
        // Should NOT rename .tmp files
        assert!(
            !rename_paths.iter().any(|p| {
                std::path::Path::new(p)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("tmp"))
            }),
            "Should NOT find .tmp files when gitignore is enabled"
        );

        // SHOULD rename these files
        assert!(rename_paths.iter().any(|p| p == "old_name.txt"));
        assert!(rename_paths.iter().any(|p| p == "old_name.rs"));
        assert!(rename_paths.iter().any(|p| p == "src/old_name.rs"));

        // When gitignore is disabled with -uu, should include all files
        let opts2 = PlanOptions {
            unrestricted_level: 2, // -uu: show all files including hidden
            ..Default::default()
        };
        let plan_no_ignore =
            plan_renames_with_conflicts(temp_dir.path(), &mapping, &opts2).unwrap();

        let all_paths: Vec<String> = plan_no_ignore
            .renames
            .iter()
            .map(|r| {
                let path_str = r
                    .path
                    .strip_prefix(temp_dir.path())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                // Normalize to forward slashes for consistent comparison
                if cfg!(windows) {
                    path_str.replace('\\', "/")
                } else {
                    path_str
                }
            })
            .collect();

        // Should include ignored files when respect_gitignore is false
        assert!(
            all_paths.iter().any(|p| p.contains("target/")),
            "Should find target/ files"
        );
        assert!(
            all_paths.iter().any(|p| p.contains("build/")),
            "Should find build/ files"
        );
        assert!(
            all_paths.iter().any(|p| {
                std::path::Path::new(p)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("tmp"))
            }),
            "Should find .tmp files"
        );
    }
}
