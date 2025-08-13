use crate::{apply_plan, scan_repository_multi, ApplyOptions, LockFile, Plan, PlanOptions, Style};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub fn rename_operation(
    old: &str,
    new: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    unrestricted_level: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<Style>,
    include_styles: Vec<Style>,
    only_styles: Vec<Style>,
    exclude_match: Vec<String>,
    preview_format: Option<String>, // "table", "diff", "json", "none"
    commit: bool,
    large: bool,
    force_with_conflicts: bool,
    rename_root: bool,
    no_rename_root: bool,
    dry_run: bool,
    no_acronyms: bool,
    include_acronyms: Vec<String>,
    exclude_acronyms: Vec<String>,
    only_acronyms: Vec<String>,
    auto_approve: bool,
    use_color: bool,
) -> Result<String> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Use provided paths or default to current directory
    let search_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Acquire lock
    let refaktor_dir = current_dir.join(".refaktor");
    let _lock = LockFile::acquire(&refaktor_dir)
        .context("Failed to acquire lock for refaktor operation")?;

    // Build the list of styles to use based on exclude, include, and only options
    let styles = build_styles_list(exclude_styles, include_styles, only_styles);

    // Generate the plan
    let options = PlanOptions {
        includes: include.clone(),
        excludes: exclude.clone(),
        respect_gitignore: true, // ignored, we use unrestricted instead
        unrestricted_level,
        styles,
        rename_files,
        rename_dirs,
        rename_root: false, // Default: do not allow root directory renames
        plan_out: PathBuf::from(".refaktor/temp_plan.json"), // temporary, will be stored in history
        coerce_separators: crate::scanner::CoercionMode::Auto,
        exclude_match,
        no_acronyms,
        include_acronyms,
        exclude_acronyms,
        only_acronyms,
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
            // Canonicalize to remove . and .. components
            absolute_path.canonicalize().unwrap_or(absolute_path)
        })
        .collect();

    let mut plan = scan_repository_multi(&resolved_paths, old, new, &options)
        .with_context(|| format!("Failed to scan repository for '{old}' -> '{new}'"))?;

    // Separate root directory renames from other renames
    let (root_renames, other_renames) = separate_root_renames(&plan.renames, &resolved_paths);

    // Update plan with filtered renames
    plan.renames = filter_renames_by_root_policy(
        root_renames.clone(),
        other_renames,
        rename_root,
        no_rename_root,
    );

    // Check if there's anything to do after filtering
    if plan.stats.total_matches == 0 && plan.renames.is_empty() {
        if !root_renames.is_empty() && !no_rename_root {
            let snippet = generate_root_rename_snippet(&root_renames)?;
            return Ok(format!("Only root directory rename detected. Use --rename-root to perform it or see suggested snippet:\n{}", snippet));
        } else {
            return Ok("Nothing to do.".to_string());
        }
    }

    // Safety checks
    validate_operation_safety(&plan, auto_approve, large, force_with_conflicts)?;

    // Show preview if requested
    if let Some(format) = preview_format.as_ref() {
        if format != "none" {
            let preview_output = generate_preview_output(&plan, format, use_color)?;
            println!("{preview_output}");
            println!(); // Add spacing before summary
        }
    }

    // Show summary
    show_rename_summary(&plan, &include, &exclude)?;

    // If dry-run, stop here
    if dry_run {
        return Ok("Dry run completed.".to_string());
    }

    // Get confirmation unless auto-approved
    if !auto_approve && !get_user_confirmation()? {
        return Ok("Aborted.".to_string());
    }

    // Apply the changes
    let result = apply_rename_changes(&mut plan, commit, force_with_conflicts)?;

    // Show completion message and handle root renames
    let mut output = result;
    if !root_renames.is_empty() && !rename_root && !no_rename_root {
        let snippet = generate_root_rename_snippet(&root_renames)?;
        output.push_str(&format!(
            "\n\nNext step (root directory rename):\n{}",
            snippet
        ));
    }

    Ok(output)
}

fn build_styles_list(
    exclude_styles: Vec<Style>,
    include_styles: Vec<Style>,
    only_styles: Vec<Style>,
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
            if !active_styles.contains(&style) {
                active_styles.push(style);
            }
        }

        if active_styles.is_empty() {
            None // Use default styles
        } else {
            Some(active_styles)
        }
    } else {
        // If --only-styles is specified, use only those styles
        Some(only_styles)
    }
}

fn separate_root_renames(
    renames: &[crate::scanner::Rename],
    resolved_paths: &[PathBuf],
) -> (Vec<crate::scanner::Rename>, Vec<crate::scanner::Rename>) {
    renames.iter().cloned().partition(|rename| {
        resolved_paths.iter().any(|root_path| {
            rename.from.parent().is_none()
                || rename.from.canonicalize().unwrap_or(rename.from.clone())
                    == root_path.canonicalize().unwrap_or(root_path.clone())
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
    let rename_count = plan.renames.len();
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
        "json" => crate::preview::Preview::Json,
        "summary" => crate::preview::Preview::Summary,
        _ => return Err(anyhow!("Invalid preview format: {}", format)),
    };

    Ok(crate::preview::render_plan(
        plan,
        preview_format,
        Some(use_color),
    ))
}

fn show_rename_summary(plan: &Plan, include: &[String], exclude: &[String]) -> Result<()> {
    println!("Refaktor plan: {} -> {}", plan.old, plan.new);
    println!(
        "Edits: {} files, {} replacements",
        plan.stats.files_with_matches, plan.stats.total_matches
    );

    if !plan.renames.is_empty() {
        println!("Renames: {} items", plan.renames.len());
    }

    if !include.is_empty() {
        println!("Include patterns: {}", include.join(", "));
    }

    if !exclude.is_empty() {
        println!("Exclude patterns: {}", exclude.join(", "));
    }

    Ok(())
}

fn get_user_confirmation() -> Result<bool> {
    print!("Apply? [y/N]: ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;
    let input = input.trim().to_lowercase();

    Ok(input == "y" || input == "yes")
}

fn apply_rename_changes(
    plan: &mut Plan,
    commit: bool,
    force_with_conflicts: bool,
) -> Result<String> {
    // Create the refaktor directory if it doesn't exist
    let refaktor_dir = PathBuf::from(".refaktor");
    fs::create_dir_all(&refaktor_dir)?;

    // Save the plan ID for the undo message
    let history_id = plan.id.clone();
    let backup_dir = refaktor_dir.join("backups");

    println!("Applying changes...");

    // Apply the plan
    let apply_options = ApplyOptions {
        create_backups: true,
        backup_dir,
        atomic: true,
        commit,
        force: force_with_conflicts,
        skip_symlinks: false,
        log_file: Some(refaktor_dir.join("logs").join(format!("{history_id}.log"))),
    };

    apply_plan(plan, &apply_options).context("Failed to apply refactoring plan")?;

    // Build completion message
    let mut output = format!(
        "✓ Applied {} replacements across {} files",
        plan.stats.total_matches, plan.stats.files_with_matches
    );

    if !plan.renames.is_empty() {
        output.push_str(&format!("\n✓ Renamed {} items", plan.renames.len()));
    }

    if commit {
        output.push_str("\n✓ Changes committed to git");
    }

    output.push_str(&format!("\nUndo with: refaktor undo {history_id}"));

    Ok(output)
}

fn generate_root_rename_snippet(root_renames: &[crate::scanner::Rename]) -> Result<String> {
    if root_renames.is_empty() {
        return Ok(String::new());
    }

    // Generate a snippet for the root rename
    let rename = &root_renames[0];
    Ok(format!(
        "mv {} {}",
        rename.from.display(),
        rename.to.display()
    ))
}
