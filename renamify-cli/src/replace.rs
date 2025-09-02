use anyhow::{anyhow, Context, Result};
use regex::Regex;
use renamify_core::{apply_plan, create_simple_plan, Plan, PlanOptions, Preview};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::cli::{OutputFormat, PreviewArg};

#[allow(clippy::too_many_arguments)]
pub fn handle_replace(
    pattern: &str,
    replacement: &str,
    paths: Vec<PathBuf>,
    no_regex: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    unrestricted: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_matching_lines: Option<String>,
    preview: Option<PreviewArg>,
    commit: bool,
    large: bool,
    force_with_conflicts: bool,
    dry_run: bool,
    yes: bool,
    use_color: bool,
    output: OutputFormat,
    quiet: bool,
) -> Result<()> {
    // Create plan options for regex/literal replacement
    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines,
        no_acronyms: true, // Disable acronym detection for replace
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        includes: include,
        excludes: exclude,
        respect_gitignore: unrestricted == 0,
        unrestricted_level: unrestricted,
        styles: None, // No case styles for replace
        rename_files,
        rename_dirs,
        rename_root: false,
        plan_out: PathBuf::from(".renamify/plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
        atomic_config: None, // Replace doesn't use atomic mode
    };

    // Create the plan using simple regex/literal replacement
    let plan = if no_regex {
        // Literal string replacement
        create_simple_plan(pattern, replacement, paths, &options, false)?
    } else {
        // Regex replacement - validate the pattern first
        Regex::new(pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;

        // Create plan with regex replacement
        create_simple_plan(pattern, replacement, paths, &options, true)?
    };

    // Check for large changes
    if !large && !yes {
        let total_files = plan
            .matches
            .iter()
            .map(|m| &m.file)
            .collect::<std::collections::HashSet<_>>()
            .len();
        let total_renames = plan.paths.len();

        if total_files > 500 || total_renames > 100 {
            eprintln!(
                "Warning: This will affect {} files and rename {} files/directories.",
                total_files, total_renames
            );
            eprintln!("Use --large to acknowledge large changes or --yes to skip this check.");
            return Err(anyhow!("Large change requires --large flag"));
        }
    }

    // Handle output formats
    match output {
        OutputFormat::Json => {
            if !quiet {
                let json = serde_json::to_string_pretty(&plan)?;
                println!("{}", json);
            }
            return Ok(());
        },
        OutputFormat::Summary if quiet => {
            // Quiet mode - no output
            return Ok(());
        },
        _ => {},
    }

    // Check if there are any changes to apply
    if plan.matches.is_empty() && plan.paths.is_empty() {
        if !quiet {
            println!("No matches found for pattern '{}'", pattern);
        }
        return Ok(());
    }

    // Show preview if not in quiet mode
    if !quiet {
        let preview_format = preview.map(|p| p.into()).unwrap_or(Preview::Summary);

        let output = renamify_core::render_plan(&plan, preview_format, Some(use_color));
        print!("{}", output);
    }

    // If dry-run, we're done
    if dry_run {
        return Ok(());
    }

    // Get confirmation unless --yes flag is provided
    if !yes {
        print!("Apply these changes? [y/N]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    // Ensure .renamify directory exists before applying
    // (it may not exist in fresh repos even after auto-init adds it to .gitignore)
    let renamify_dir = PathBuf::from(".renamify");
    if !renamify_dir.exists() {
        std::fs::create_dir_all(&renamify_dir).context("Failed to create .renamify directory")?;
    }

    // Apply the plan
    let apply_options = renamify_core::ApplyOptions {
        create_backups: true,
        backup_dir: PathBuf::from(".renamify/backups"),
        commit: false,
        force: force_with_conflicts,
        skip_symlinks: false,
        log_file: None,
    };
    apply_plan(&mut plan.clone(), &apply_options)?;

    // Commit if requested
    if commit {
        commit_changes(&plan)?;
    }

    if !quiet {
        println!("✅ Applied successfully! Operation ID: {}", plan.id);
    }

    Ok(())
}

fn commit_changes(plan: &Plan) -> Result<()> {
    use std::process::Command;

    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if !status.status.success() {
        return Err(anyhow!("Not in a git repository"));
    }

    // Add all changed files
    let files_to_add: Vec<String> = plan
        .matches
        .iter()
        .map(|m| m.file.to_string_lossy().to_string())
        .chain(
            plan.paths
                .iter()
                .map(|r| r.new_path.to_string_lossy().to_string()),
        )
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if !files_to_add.is_empty() {
        let status = Command::new("git")
            .arg("add")
            .args(&files_to_add)
            .status()
            .context("Failed to add files to git")?;

        if !status.success() {
            return Err(anyhow!("Failed to add files to git"));
        }
    }

    // Create commit message
    let message = format!("Replace '{}' with '{}'", plan.search, plan.replace);

    let status = Command::new("git")
        .args(["commit", "-m", &message])
        .status()
        .context("Failed to create git commit")?;

    if !status.success() {
        return Err(anyhow!("Failed to create git commit"));
    }

    Ok(())
}
