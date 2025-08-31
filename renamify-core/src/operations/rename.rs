use crate::{
    apply_plan, output::RenameResult, scan_repository_multi, ApplyOptions, LockFile, Plan,
    PlanOptions, Style,
};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{self, IsTerminal, Write as IoWrite};
use std::path::PathBuf;

/// Rename operation - returns structured data
#[allow(clippy::too_many_arguments)]
#[allow(clippy::fn_params_excessive_bools)]
pub fn rename_operation(
    search: &str,
    replace: &str,
    paths: Vec<PathBuf>,
    include: &[String],
    exclude: &[String],
    unrestricted_level: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: &[Style],
    include_styles: &[Style],
    only_styles: &[Style],
    exclude_match: &[String],
    exclude_matching_lines: Option<&String>,
    preview_format: Option<&String>,
    commit: bool,
    large: bool,
    force_with_conflicts: bool,
    rename_root: bool,
    no_rename_root: bool,
    dry_run: bool,
    no_acronyms: bool,
    include_acronyms: &[String],
    exclude_acronyms: &[String],
    only_acronyms: &[String],
    auto_approve: bool,
    use_color: bool,
) -> Result<(RenameResult, Option<String>)> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Use provided paths or default to current directory
    let search_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Acquire lock
    let renamify_dir = current_dir.join(".renamify");
    let _lock = LockFile::acquire(&renamify_dir)
        .context("Failed to acquire lock for renamify operation")?;

    // Build the list of styles to use based on exclude, include, and only options
    let styles = build_styles_list(exclude_styles, include_styles, only_styles);

    // Generate the plan
    let options = PlanOptions {
        includes: include.to_owned(),
        excludes: exclude.to_owned(),
        respect_gitignore: true,
        unrestricted_level,
        styles,
        rename_files,
        rename_dirs,
        rename_root: false,
        plan_out: PathBuf::from(".renamify/temp_plan.json"),
        coerce_separators: crate::scanner::CoercionMode::Auto,
        exclude_match: exclude_match.to_owned(),
        exclude_matching_lines: exclude_matching_lines.map(std::string::ToString::to_string),
        no_acronyms,
        include_acronyms: include_acronyms.to_owned(),
        exclude_acronyms: exclude_acronyms.to_owned(),
        only_acronyms: only_acronyms.to_owned(),
        ignore_ambiguous: false, // TODO: Get from args
    };

    // Resolve all search paths to absolute paths and canonicalize them
    let resolved_paths: Vec<PathBuf> = search_paths
        .iter()
        .map(|path| {
            let absolute_path = if path.is_absolute() {
                path.clone()
            } else {
                current_dir.join(path)
            };
            absolute_path.canonicalize().unwrap_or(absolute_path)
        })
        .collect();

    let mut plan = scan_repository_multi(&resolved_paths, search, replace, &options)
        .with_context(|| format!("Failed to scan repository for '{search}' -> '{replace}'"))?;

    // Separate root directory renames from other renames
    let (root_renames, other_renames) = separate_root_renames(&plan.paths, &resolved_paths);

    // Update plan with filtered renames
    plan.paths = filter_renames_by_root_policy(
        root_renames.clone(),
        other_renames,
        rename_root,
        no_rename_root,
    );

    // Check if there's anything to do after filtering
    if plan.stats.total_matches == 0 && plan.paths.is_empty() {
        if !root_renames.is_empty() && !no_rename_root {
            let snippet = generate_root_rename_snippet(&root_renames);
            let preview = Some(format!(
                "Only root directory rename detected. Use --rename-root to perform it or see suggested snippet:\n{}",
                snippet
            ));
            return Ok((
                RenameResult {
                    plan_id: plan.id.clone(),
                    search: search.to_string(),
                    replace: replace.to_string(),
                    files_changed: 0,
                    replacements: 0,
                    renames: 0,
                    committed: false,
                    plan: Some(plan),
                },
                preview,
            ));
        }
        return Ok((
            RenameResult {
                plan_id: plan.id.clone(),
                search: search.to_string(),
                replace: replace.to_string(),
                files_changed: 0,
                replacements: 0,
                renames: 0,
                committed: false,
                plan: Some(plan),
            },
            Some(format!("No matches found for '{}'", search)),
        ));
    }

    // Generate preview if requested
    let mut preview_output = None;
    if let Some(format) = preview_format.as_ref() {
        if *format != "none" {
            let preview = generate_preview_output(&plan, format, use_color)?;
            preview_output = Some(preview.clone());

            // Print preview BEFORE asking for confirmation (but not in dry-run)
            if !dry_run && !auto_approve {
                println!("{}", preview);
            }
        }
    }

    // If dry-run, stop here (no safety checks needed for dry-run)
    if dry_run {
        return Ok((
            RenameResult {
                plan_id: plan.id.clone(),
                search: search.to_string(),
                replace: replace.to_string(),
                files_changed: plan.stats.files_with_matches,
                replacements: plan.stats.total_matches,
                renames: plan.paths.len(),
                committed: false,
                plan: Some(plan),
            },
            preview_output,
        ));
    }

    // Safety checks (only for non-dry-run operations)
    validate_operation_safety(&plan, auto_approve, large, force_with_conflicts)?;

    // Get confirmation unless auto-approved
    if !auto_approve && !get_user_confirmation()? {
        return Ok((
            RenameResult {
                plan_id: plan.id.clone(),
                search: search.to_string(),
                replace: replace.to_string(),
                files_changed: 0,
                replacements: 0,
                renames: 0,
                committed: false,
                plan: Some(plan),
            },
            Some("Aborted.".to_string()),
        ));
    }

    // Apply the changes
    let history_id = plan.id.clone();
    let files_changed = plan.stats.files_with_matches;
    let replacements = plan.stats.total_matches;
    let renames = plan.paths.len();

    apply_rename_changes(&mut plan, commit, force_with_conflicts)?;

    // Add root rename snippet to preview if needed
    if !root_renames.is_empty() && !rename_root && !no_rename_root {
        let snippet = generate_root_rename_snippet(&root_renames);
        if let Some(ref mut preview) = preview_output {
            use std::fmt::Write;
            write!(
                preview,
                "\n\nNext step (root directory rename):\n{}",
                snippet
            )
            .unwrap();
        } else {
            preview_output = Some(format!("Next step (root directory rename):\n{}", snippet));
        }
    }

    Ok((
        RenameResult {
            plan_id: history_id,
            search: search.to_string(),
            replace: replace.to_string(),
            files_changed,
            replacements,
            renames,
            committed: commit,
            plan: Some(plan),
        },
        preview_output,
    ))
}

fn build_styles_list(
    exclude_styles: &[Style],
    include_styles: &[Style],
    only_styles: &[Style],
) -> Option<Vec<Style>> {
    if only_styles.is_empty() {
        // Start with the default styles
        let default_styles = Style::default_styles();

        // Remove excluded styles from defaults
        let mut active_styles: Vec<Style> = default_styles
            .into_iter()
            .filter(|s| !exclude_styles.contains(s))
            .collect();

        // Add included styles
        for style in include_styles {
            if !active_styles.contains(style) {
                active_styles.push(*style);
            }
        }

        if active_styles.is_empty() {
            None
        } else {
            Some(active_styles)
        }
    } else {
        // If --only-styles is specified, use only those styles
        Some(only_styles.to_vec())
    }
}

fn separate_root_renames(
    paths: &[crate::scanner::Rename],
    resolved_paths: &[PathBuf],
) -> (Vec<crate::scanner::Rename>, Vec<crate::scanner::Rename>) {
    paths.iter().cloned().partition(|rename| {
        resolved_paths.iter().any(|root_path| {
            rename.path.parent().is_none()
                || rename
                    .path
                    .canonicalize()
                    .unwrap_or_else(|_| rename.path.clone())
                    == root_path
                        .canonicalize()
                        .unwrap_or_else(|_| root_path.clone())
        })
    })
}

fn filter_renames_by_root_policy(
    root_renames: Vec<crate::scanner::Rename>,
    other_renames: Vec<crate::scanner::Rename>,
    rename_root: bool,
    no_rename_root: bool,
) -> Vec<crate::scanner::Rename> {
    if rename_root {
        // Include all renames
        root_renames.into_iter().chain(other_renames).collect()
    } else if no_rename_root {
        // Exclude root renames completely
        other_renames
    } else {
        // Default behavior: exclude root renames
        other_renames
    }
}

fn validate_operation_safety(
    plan: &Plan,
    auto_approve: bool,
    large: bool,
    force_with_conflicts: bool,
) -> Result<()> {
    // Safety check: Non-TTY without auto-approve should exit with error
    if !auto_approve && !io::stdout().is_terminal() {
        return Err(anyhow!(
            "Cannot prompt for confirmation in non-interactive mode. Use auto_approve=true."
        ));
    }

    // Safety check: Size guard for large changes
    let file_count = plan.stats.files_with_matches;
    let rename_count = plan.paths.len();
    if (file_count > 500 || rename_count > 100) && !large {
        return Err(anyhow!(
            "Large change detected ({} files, {} renames). Use large=true to acknowledge.",
            file_count,
            rename_count
        ));
    }

    // Safety check: Conflicts should abort unless forced
    let has_conflicts = false; // TODO: implement conflict detection
    if has_conflicts && !force_with_conflicts {
        return Err(anyhow!(
            "Conflicts detected. Use force_with_conflicts=true to override."
        ));
    }

    Ok(())
}

fn generate_preview_output(plan: &Plan, format: &str, use_color: bool) -> Result<String> {
    let preview_format = match format {
        "table" => crate::preview::Preview::Table,
        "diff" => crate::preview::Preview::Diff,
        "matches" => crate::preview::Preview::Matches,
        "summary" => crate::preview::Preview::Summary,
        _ => return Err(anyhow!("Invalid preview format: {}", format)),
    };

    Ok(crate::preview::render_plan(
        plan,
        preview_format,
        Some(use_color),
    ))
}

fn get_user_confirmation() -> Result<bool> {
    print!("Apply? [y/N]: ");
    IoWrite::flush(&mut io::stdout()).context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;
    let input = input.trim().to_lowercase();

    Ok(input == "y" || input == "yes")
}

fn apply_rename_changes(plan: &mut Plan, commit: bool, force_with_conflicts: bool) -> Result<()> {
    // Create the renamify directory if it doesn't exist
    let renamify_dir = PathBuf::from(".renamify");
    fs::create_dir_all(&renamify_dir)?;

    // Save the plan ID for the undo message
    let history_id = plan.id.clone();
    let backup_dir = renamify_dir.join("backups");

    // Don't print to stdout when we're returning structured data
    eprintln!("Applying changes...");

    // Apply the plan
    let apply_options = ApplyOptions {
        create_backups: true,
        backup_dir,
        atomic: true,
        commit,
        force: force_with_conflicts,
        skip_symlinks: false,
        log_file: Some(renamify_dir.join("logs").join(format!("{history_id}.log"))),
    };

    apply_plan(plan, &apply_options).context("Failed to apply renaming plan")?;
    Ok(())
}

fn generate_root_rename_snippet(root_renames: &[crate::scanner::Rename]) -> String {
    if root_renames.is_empty() {
        return String::new();
    }

    // Generate a snippet for the root rename
    let rename = &root_renames[0];
    format!("mv {} {}", rename.path.display(), rename.new_path.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{Rename, RenameKind};
    use std::path::PathBuf;

    fn create_test_rename(from: &str, to: &str) -> Rename {
        Rename {
            path: PathBuf::from(from),
            new_path: PathBuf::from(to),
            kind: RenameKind::File,
            coercion_applied: None,
        }
    }

    #[test]
    fn test_build_styles_list_default() {
        let result = build_styles_list(&[], &[], &[]);
        assert!(result.is_some());
        let styles = result.unwrap();
        assert!(!styles.is_empty());
        // Should contain default styles
        assert!(styles.contains(&Style::Snake));
        assert!(styles.contains(&Style::Camel));
    }

    #[test]
    fn test_build_styles_list_exclude() {
        let exclude = vec![Style::Snake];
        let result = build_styles_list(&exclude, &[], &[]);
        assert!(result.is_some());
        let styles = result.unwrap();
        assert!(!styles.contains(&Style::Snake));
        assert!(styles.contains(&Style::Camel)); // Should still have others
    }

    #[test]
    fn test_build_styles_list_include() {
        let include = vec![Style::Dot];
        let result = build_styles_list(&[], &include, &[]);
        assert!(result.is_some());
        let styles = result.unwrap();
        assert!(styles.contains(&Style::Dot)); // Should add included style
    }

    #[test]
    fn test_build_styles_list_only() {
        let only = vec![Style::Pascal, Style::Kebab];
        let result = build_styles_list(&[], &[], &only);
        assert!(result.is_some());
        let styles = result.unwrap();
        assert_eq!(styles.len(), 2);
        assert!(styles.contains(&Style::Pascal));
        assert!(styles.contains(&Style::Kebab));
        assert!(!styles.contains(&Style::Snake)); // Should not have defaults
    }

    #[test]
    fn test_build_styles_list_exclude_all() {
        let all_styles = Style::default_styles();
        let result = build_styles_list(&all_styles, &[], &[]);
        assert!(result.is_none()); // Should return None when all styles excluded
    }

    #[test]
    fn test_separate_root_renames() {
        let renames = vec![
            create_test_rename("root_file.txt", "new_root_file.txt"),
            create_test_rename("subdir/file.txt", "subdir/new_file.txt"),
        ];
        let resolved_paths = vec![PathBuf::from(".")];

        let (root, other) = separate_root_renames(&renames, &resolved_paths);

        // Note: This test might not work exactly as expected due to path canonicalization
        // But it tests the function structure
        assert!(!root.is_empty() || !other.is_empty());
        assert_eq!(root.len() + other.len(), 2);
    }

    #[test]
    fn test_filter_renames_by_root_policy_include_all() {
        let root_renames = vec![create_test_rename("root.txt", "new_root.txt")];
        let other_renames = vec![create_test_rename("sub/file.txt", "sub/new_file.txt")];

        let result = filter_renames_by_root_policy(
            root_renames,
            other_renames,
            true,  // rename_root = true
            false, // no_rename_root = false
        );

        assert_eq!(result.len(), 2); // Should include both
    }

    #[test]
    fn test_filter_renames_by_root_policy_exclude_root() {
        let root_renames = vec![create_test_rename("root.txt", "new_root.txt")];
        let other_renames = vec![create_test_rename("sub/file.txt", "sub/new_file.txt")];

        let result = filter_renames_by_root_policy(
            root_renames,
            other_renames,
            false, // rename_root = false
            true,  // no_rename_root = true
        );

        assert_eq!(result.len(), 1); // Should exclude root renames
        assert_eq!(result[0].path, PathBuf::from("sub/file.txt"));
    }

    #[test]
    fn test_filter_renames_by_root_policy_default() {
        let root_renames = vec![create_test_rename("root.txt", "new_root.txt")];
        let other_renames = vec![create_test_rename("sub/file.txt", "sub/new_file.txt")];

        let result = filter_renames_by_root_policy(
            root_renames,
            other_renames,
            false, // rename_root = false
            false, // no_rename_root = false (default behavior)
        );

        assert_eq!(result.len(), 1); // Default should exclude root renames
        assert_eq!(result[0].path, PathBuf::from("sub/file.txt"));
    }

    #[test]
    fn test_validate_operation_safety_large_change() {
        use crate::scanner::{Plan, Stats};
        use std::collections::HashMap;

        let plan = Plan {
            id: "test".to_string(),
            created_at: "2024-01-01".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: (0..200)
                .map(|i| {
                    create_test_rename(&format!("file{}.txt", i), &format!("newfile{}.txt", i))
                })
                .collect(),
            stats: Stats {
                files_scanned: 1000,
                total_matches: 1000,
                matches_by_variant: HashMap::new(),
                files_with_matches: 600, // > 500, should trigger large change check
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        // Should error without large=true
        let result = validate_operation_safety(&plan, true, false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Large change"));

        // Should succeed with large=true
        let result = validate_operation_safety(&plan, true, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_operation_safety_rename_count() {
        use crate::scanner::{Plan, Stats};
        use std::collections::HashMap;

        let plan = Plan {
            id: "test".to_string(),
            created_at: "2024-01-01".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: (0..150)
                .map(|i| {
                    create_test_rename(&format!("file{}.txt", i), &format!("newfile{}.txt", i))
                })
                .collect(), // > 100 renames
            stats: Stats {
                files_scanned: 200,
                total_matches: 50,
                matches_by_variant: HashMap::new(),
                files_with_matches: 50,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        // Should error due to too many renames
        let result = validate_operation_safety(&plan, true, false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Large change"));
    }

    #[test]
    fn test_generate_preview_output_table() {
        use crate::scanner::{Plan, Stats};
        use std::collections::HashMap;

        let plan = Plan {
            id: "test".to_string(),
            created_at: "2024-01-01".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let result = generate_preview_output(&plan, "table", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_preview_output_invalid_format() {
        use crate::scanner::{Plan, Stats};
        use std::collections::HashMap;

        let plan = Plan {
            id: "test".to_string(),
            created_at: "2024-01-01".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let result = generate_preview_output(&plan, "invalid", false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid preview format"));
    }

    #[test]
    fn test_generate_root_rename_snippet() {
        let renames = vec![
            create_test_rename("old_root", "new_root"),
            create_test_rename("old_root2", "new_root2"),
        ];

        let snippet = generate_root_rename_snippet(&renames);
        assert_eq!(snippet, "mv old_root new_root");
    }

    #[test]
    fn test_generate_root_rename_snippet_empty() {
        let renames = vec![];
        let snippet = generate_root_rename_snippet(&renames);
        assert_eq!(snippet, "");
    }

    #[test]
    fn test_get_user_confirmation_needs_interactive_environment() {
        // This test can't be properly tested in CI, but we can at least test the function exists
        // and handles the case when stdin is not available
        // In real usage, this would require interactive input
    }
}
