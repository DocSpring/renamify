use crate::{
    scan_repository_multi, write_plan, write_preview, LockFile, PlanOptions, Preview, Style,
};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// High-level plan operation - equivalent to `renamify plan` command
pub fn plan_operation(
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
    plan_out: Option<PathBuf>,
    preview_format: Option<String>, // "table", "diff", "json", "summary", "none"
    use_color: bool,
) -> Result<String> {
    plan_operation_with_dry_run(
        old,
        new,
        paths,
        include,
        exclude,
        true, // respect_gitignore
        unrestricted_level,
        rename_files,
        rename_dirs,
        exclude_styles,
        include_styles,
        only_styles,
        exclude_match,
        plan_out,
        preview_format,
        false, // dry_run
        false, // fixed_table_width - not applicable for non-dry-run
        use_color,
        false,  // no_acronyms - use defaults
        vec![], // include_acronyms
        vec![], // exclude_acronyms
        vec![], // only_acronyms
    )
}

/// High-level plan operation with full options - supports both plan and dry-run commands
#[allow(clippy::too_many_arguments)]
pub fn plan_operation_with_dry_run(
    old: &str,
    new: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    respect_gitignore: bool,
    unrestricted_level: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<Style>,
    include_styles: Vec<Style>,
    only_styles: Vec<Style>,
    exclude_match: Vec<String>,
    plan_out: Option<PathBuf>,
    preview_format: Option<String>, // "table", "diff", "json", "summary", "none"
    dry_run: bool,
    fixed_table_width: bool,
    use_color: bool,
    no_acronyms: bool,
    include_acronyms: Vec<String>,
    exclude_acronyms: Vec<String>,
    only_acronyms: Vec<String>,
) -> Result<String> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Use provided paths or default to current directory
    let search_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Acquire lock (only for non-dry-run operations)
    let renamify_dir = current_dir.join(".renamify");
    let _lock = if dry_run {
        None
    } else {
        Some(
            LockFile::acquire(&renamify_dir)
                .context("Failed to acquire lock for renamify operation")?,
        )
    };

    // Build styles list
    let styles = build_styles_list(exclude_styles, include_styles, only_styles);

    let plan_out_path = plan_out.unwrap_or_else(|| PathBuf::from(".renamify/plan.json"));

    let plan_options = PlanOptions {
        includes: include,
        excludes: exclude,
        respect_gitignore,
        unrestricted_level: unrestricted_level.min(3), // Cap at 3 for safety
        styles,
        rename_files,
        rename_dirs,
        rename_root: false, // Default: do not allow root directory renames in plan
        plan_out: plan_out_path.clone(),
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

    let plan = scan_repository_multi(&resolved_paths, old, new, &plan_options)
        .context("Failed to scan repository")?;

    // Generate preview content
    let preview_content = if let Some(format) = preview_format.as_ref() {
        if format == "none" {
            None
        } else {
            let preview_format = parse_preview_format(format)?;
            let content = if fixed_table_width {
                crate::preview::render_plan_with_fixed_width(
                    &plan,
                    preview_format,
                    Some(use_color),
                    true,
                )
            } else {
                crate::preview::render_plan(&plan, preview_format, Some(use_color))
            };
            // For dry-run operations (like tests), print the preview
            if dry_run {
                println!("{}", content);
            }
            Some(content)
        }
    } else {
        None
    };

    // Write plan unless dry-run
    if !dry_run {
        write_plan(&plan, &plan_out_path).context("Failed to write plan")?;

        // Return both preview content and file path message for non-dry-run
        let file_message = format!("Plan written to: {}", plan_out_path.display());
        if let Some(content) = preview_content {
            return Ok(format!("{}\n\n{}", content, file_message));
        } else if preview_format.as_ref().is_none_or(|f| f != "json") {
            return Ok(file_message);
        }
    }

    // Check for conflicts and warn
    if let Some(conflicts) = check_for_conflicts(&plan) {
        let warning = format!("Warning: {} conflicts detected", conflicts);
        if !dry_run {
            eprintln!("\n{}", warning);
            eprintln!("Use --force-with-conflicts to apply anyway");
        }
        return Ok(warning);
    }

    // Return preview content for dry-run operations, summary for others
    if dry_run && preview_content.is_some() {
        Ok(preview_content.unwrap())
    } else {
        Ok(format!(
            "Generated plan with {} matches and {} renames{}",
            plan.stats.total_matches,
            plan.renames.len(),
            if dry_run {
                ""
            } else {
                &format!(". Saved to {}", plan_out_path.display())
            }
        ))
    }
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

fn parse_preview_format(format: &str) -> Result<Preview> {
    match format.to_lowercase().as_str() {
        "table" => Ok(Preview::Table),
        "diff" => Ok(Preview::Diff),
        "json" => Ok(Preview::Json),
        "summary" => Ok(Preview::Summary),
        "none" => Ok(Preview::None),
        _ => Err(anyhow::anyhow!("Invalid preview format: {}", format)),
    }
}

fn check_for_conflicts(_plan: &crate::scanner::Plan) -> Option<usize> {
    // Check if there are any rename conflicts
    // This is a placeholder - would need to check the actual conflicts
    // from the rename module
    None
}
