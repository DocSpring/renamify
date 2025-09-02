use crate::{
    output::PlanResult, scan_repository_multi, write_plan, LockFile, PlanOptions, Preview, Style,
};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Plan operation - returns structured data
#[allow(clippy::too_many_arguments)]
#[allow(clippy::fn_params_excessive_bools)]
pub fn plan_operation(
    search: &str,
    replace: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    respect_gitignore: bool,
    unrestricted_level: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: &[Style],
    include_styles: &[Style],
    only_styles: &[Style],
    exclude_match: Vec<String>,
    exclude_matching_lines: Option<String>,
    plan_out: Option<PathBuf>,
    preview_format: Option<&String>,
    dry_run: bool,
    fixed_table_width: bool,
    use_color: bool,
    no_acronyms: bool,
    include_acronyms: Vec<String>,
    exclude_acronyms: Vec<String>,
    only_acronyms: Vec<String>,
    working_dir: Option<&std::path::Path>,
    atomic_config: Option<&crate::atomic::AtomicConfig>,
) -> Result<(PlanResult, Option<String>)> {
    let current_dir = working_dir.map_or_else(
        || std::env::current_dir().expect("Failed to get current directory"),
        std::path::Path::to_path_buf,
    );

    // Use provided paths or default to current directory
    let search_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Only acquire lock for non-dry-run operations (actual plan writes)
    let renamify_dir = current_dir.join(".renamify");
    let _lock = if dry_run {
        None // No lock needed for dry-run/search operations
    } else {
        Some(
            LockFile::acquire(&renamify_dir)
                .context("Failed to acquire lock for renamify operation")?,
        )
    };

    // Build styles list
    let styles = build_styles_list(
        exclude_styles.to_vec(),
        include_styles.to_vec(),
        only_styles.to_vec(),
    );

    let plan_out_path = plan_out.unwrap_or_else(|| PathBuf::from(".renamify/plan.json"));

    let plan_options = PlanOptions {
        includes: include,
        excludes: exclude,
        respect_gitignore,
        unrestricted_level: unrestricted_level.min(3),
        styles,
        rename_files,
        rename_dirs,
        rename_root: false,
        plan_out: plan_out_path.clone(),
        coerce_separators: crate::scanner::CoercionMode::Auto,
        exclude_match,
        exclude_matching_lines,
        no_acronyms,
        include_acronyms,
        exclude_acronyms,
        only_acronyms,
        ignore_ambiguous: false, // TODO: Get from args
        atomic_config: atomic_config.cloned(),
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

    let plan = scan_repository_multi(&resolved_paths, search, replace, &plan_options)
        .context("Failed to scan repository")?;

    // Generate preview content
    let preview_content = if let Some(format) = preview_format.as_ref() {
        if *format == "none" {
            None
        } else {
            let preview = parse_preview_format(format)?;
            let use_color = if *format == "json" { false } else { use_color };
            Some(crate::preview::render_plan_with_fixed_width(
                &plan,
                preview,
                Some(use_color),
                fixed_table_width,
            ))
        }
    } else {
        None
    };

    // Write the plan to disk unless dry-run
    if !dry_run {
        // Create the directory if it doesn't exist
        if let Some(parent) = plan_out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        // Write the plan
        write_plan(&plan, &plan_out_path)
            .with_context(|| format!("Failed to write plan to {}", plan_out_path.display()))?;
    }

    // Create structured result (include full plan for JSON output)
    let result = PlanResult {
        plan_id: plan.id.clone(),
        search: search.to_string(),
        replace: replace.to_string(),
        files_with_matches: plan.stats.files_with_matches,
        total_matches: plan.stats.total_matches,
        renames: plan.paths.len(),
        dry_run,
        plan: Some(plan),
    };

    Ok((result, preview_content))
}

#[allow(clippy::needless_pass_by_value)]
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
            None
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
        "matches" => Ok(Preview::Matches),
        "summary" => Ok(Preview::Summary),
        "none" => Ok(Preview::None),
        _ => Err(anyhow::anyhow!("Invalid preview format: {}", format)),
    }
}
