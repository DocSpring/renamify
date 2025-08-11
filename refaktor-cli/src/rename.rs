use anyhow::{anyhow, Context, Result};
use refaktor_core::{apply_plan, scan_repository_multi, ApplyOptions, LockFile, Plan, PlanOptions};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

use crate::{PreviewFormatArg, StyleArg};

pub fn handle_rename(
    old: &str,
    new: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    unrestricted: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<StyleArg>,
    include_styles: Vec<StyleArg>,
    only_styles: Vec<StyleArg>,
    exclude_match: Vec<String>,
    preview: Option<PreviewFormatArg>,
    commit: bool,
    large: bool,
    force_with_conflicts: bool,
    _confirm_collisions: bool, // TODO: implement collision detection
    rename_root: bool,
    no_rename_root: bool,
    dry_run: bool,
    auto_approve: bool,
    use_color: bool,
) -> Result<()> {
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
    let styles = {
        if !only_styles.is_empty() {
            // If --only-styles is specified, use only those styles
            Some(only_styles.into_iter().map(Into::into).collect())
        } else {
            // Start with the default styles
            let default_styles = vec![
                StyleArg::Snake,
                StyleArg::Kebab,
                StyleArg::Camel,
                StyleArg::Pascal,
                StyleArg::ScreamingSnake,
            ];

            // Remove excluded styles from defaults
            let mut active_styles: Vec<StyleArg> = default_styles
                .into_iter()
                .filter(|s| !exclude_styles.contains(s))
                .collect();

            // Add included styles (Title, Train, Dot)
            for style in include_styles {
                if !active_styles.contains(&style) {
                    active_styles.push(style);
                }
            }

            if active_styles.is_empty() {
                None // Use default styles
            } else {
                Some(active_styles.into_iter().map(Into::into).collect())
            }
        }
    };

    // Generate the plan
    let options = PlanOptions {
        includes: include.clone(),
        excludes: exclude.clone(),
        respect_gitignore: true, // ignored, we use unrestricted instead
        unrestricted_level: unrestricted,
        styles,
        rename_files,
        rename_dirs,
        rename_root: false, // Default: do not allow root directory renames
        plan_out: PathBuf::from(".refaktor/temp_plan.json"), // temporary, will be stored in history
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
        exclude_match,
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
        .with_context(|| format!("Failed to scan repository for '{}' -> '{}'", old, new))?;

    // Separate root directory renames from other renames
    // For multi-path, check if a rename matches any of the resolved paths
    let (root_renames, other_renames): (Vec<_>, Vec<_>) =
        plan.renames.into_iter().partition(|rename| {
            resolved_paths.iter().any(|root_path| {
                rename.from.parent().is_none()
                    || rename.from.canonicalize().unwrap_or(rename.from.clone())
                        == root_path.canonicalize().unwrap_or(root_path.clone())
            })
        });

    // Update plan with filtered renames (excluding root by default)
    plan.renames = if rename_root {
        // If --rename-root is specified, include all renames
        root_renames
            .clone()
            .into_iter()
            .chain(other_renames)
            .collect()
    } else if no_rename_root {
        // If --no-rename-root is specified, exclude root renames completely
        other_renames
    } else {
        // Default behavior: exclude root renames but save them for the snippet
        other_renames
    };

    // Check if there's anything to do after filtering
    if plan.stats.total_matches == 0 && plan.renames.is_empty() {
        if !root_renames.is_empty() && !no_rename_root {
            println!("Only root directory rename detected. Use --rename-root to perform it or see suggested snippet below.");
            print_root_rename_snippet(&root_renames)?;
        } else {
            println!("Nothing to do.");
        }
        return Ok(());
    }

    // Safety check: Non-TTY without auto-approve should exit with error
    if !auto_approve && !io::stdout().is_terminal() {
        return Err(anyhow!(
            "Cannot prompt for confirmation in non-interactive mode. Use -y/--yes to auto-approve."
        ));
    }

    // Safety check: Size guard for large changes
    let file_count = plan.stats.files_with_matches;
    let rename_count = plan.renames.len();
    if (file_count > 500 || rename_count > 100) && !large {
        return Err(anyhow!(
            "Large change detected ({} files, {} renames). Use --large to acknowledge.",
            file_count,
            rename_count
        ));
    }

    // Safety check: Conflicts should abort unless forced
    let has_conflicts = false; // TODO: implement conflict detection
    if has_conflicts && !force_with_conflicts {
        return Err(anyhow!(
            "Conflicts detected. Use --force-with-conflicts to override."
        ));
    }

    // Show preview unless explicitly set to none
    // The preview parameter already has the config default applied if not specified
    if let Some(preview_format) = preview {
        if preview_format != PreviewFormatArg::None {
            let preview_output =
                refaktor_core::preview::render_plan(&plan, preview_format.into(), Some(use_color))?;
            println!("{}", preview_output);
            println!(); // Add spacing before summary
        }
    }

    // Show summary
    show_rename_summary(&plan, &include, &exclude)?;

    // If dry-run, stop here without prompting or applying
    if dry_run {
        return Ok(());
    }

    // Get confirmation unless auto-approved
    if !auto_approve {
        print!("Apply? [y/N]: ");
        io::Write::flush(&mut io::stdout()).context("Failed to flush stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Create the refaktor directory if it doesn't exist
    let refaktor_dir = PathBuf::from(".refaktor");
    fs::create_dir_all(&refaktor_dir)?;

    // Save the plan ID for the undo message
    let history_id = plan.id.clone();
    // Don't include plan_id in backup_dir - apply_plan will add it
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
        log_file: Some(
            refaktor_dir
                .join("logs")
                .join(format!("{}.log", history_id)),
        ),
    };

    apply_plan(&plan, &apply_options).context("Failed to apply refactoring plan")?;

    // Show completion message
    println!(
        "✓ Applied {} replacements across {} files",
        plan.stats.total_matches, plan.stats.files_with_matches
    );
    if !plan.renames.is_empty() {
        println!("✓ Renamed {} items", plan.renames.len());
    }
    if commit {
        println!("✓ Changes committed to git");
    }
    println!("Undo with: refaktor undo {}", history_id);

    // If there were root renames that we didn't perform, show the next step snippet
    if !root_renames.is_empty() && !rename_root && !no_rename_root {
        println!();
        println!("== Next Step ==");
        println!("To rename the project directory, run the following command:");
        print_root_rename_snippet(&root_renames)?;
        println!();
        println!("Note: Your current shell's pwd may be stale until you cd to the new directory.");
    }

    Ok(())
}

fn show_rename_summary(plan: &Plan, include: &[String], exclude: &[String]) -> Result<()> {
    println!("Refaktor plan: {} -> {}", plan.old, plan.new);
    println!(
        "Edits: {} files, {} replacements",
        plan.stats.files_with_matches, plan.stats.total_matches
    );
    println!(
        "Renames: {} files, {} dirs",
        plan.renames
            .iter()
            .filter(|r| matches!(r.kind, refaktor_core::scanner::RenameKind::File))
            .count(),
        plan.renames
            .iter()
            .filter(|r| matches!(r.kind, refaktor_core::scanner::RenameKind::Dir))
            .count()
    );
    // println!("Conflicts: 0"); // TODO: implement conflict detection

    if !include.is_empty() || !exclude.is_empty() {
        print!("Includes: ");
        if include.is_empty() {
            print!("**");
        } else {
            print!("{}", include.join(", "));
        }
        if !exclude.is_empty() {
            print!("  Excludes: {}", exclude.join(", "));
        }
        println!();
    }

    Ok(())
}

fn print_root_rename_snippet(root_renames: &[refaktor_core::scanner::Rename]) -> Result<()> {
    if root_renames.is_empty() {
        return Ok(());
    }

    // Assume the first rename is the main root directory rename
    let rename = &root_renames[0];
    let old_name = rename
        .from
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let new_name = rename
        .to
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("new_project");

    println!();

    // Detect platform and show appropriate command
    #[cfg(target_os = "windows")]
    {
        println!("# Windows PowerShell:");
        println!("$parent = Split-Path -Path $PWD");
        println!("$new = '{}'", new_name);
        println!("Rename-Item -LiteralPath $PWD -NewName $new");
        println!("Set-Location (Join-Path $parent $new)");
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("# POSIX shell:");
        println!("cd ..");
        println!("mv \"{}\" \"{}\"", old_name, new_name);
        println!("cd \"{}\"", new_name);
        println!();
        println!("# Robust variant:");
        println!(
            "parent=\"$(dirname \"$PWD\")\"; old=\"$(basename \"$PWD\")\"; new=\"{}\"",
            new_name
        );
        println!("cd \"$parent\" && mv \"$old\" \"$new\" && cd \"$new\"");
    }

    Ok(())
}
