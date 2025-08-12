use crate::{apply_plan, scan_repository_multi, undo_refactoring, ApplyOptions, PlanOptions};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// High-level API for rename operations - equivalent to `refaktor rename` command
pub fn rename(
    old: &str,
    new: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    unrestricted_level: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<String>,
    include_styles: Vec<String>, 
    only_styles: Vec<String>,
    exclude_match: Vec<String>,
    commit: bool,
    force_with_conflicts: bool,
    auto_approve: bool,
) -> Result<String> {
    // Convert style args - for now just use None to get defaults
    let styles = if only_styles.is_empty() && include_styles.is_empty() && exclude_styles.is_empty() {
        None
    } else {
        // TODO: implement proper style conversion
        None
    };

    // Use current directory if no paths provided
    let search_paths = if paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        paths
    };

    // Generate plan
    let plan_options = PlanOptions {
        includes: include,
        excludes: exclude,
        respect_gitignore: true,
        unrestricted_level,
        styles,
        rename_files,
        rename_dirs,
        rename_root: false,
        plan_out: PathBuf::from(".refaktor/temp_plan.json"),
        coerce_separators: crate::scanner::CoercionMode::Auto,
        exclude_match,
    };

    let plan = scan_repository_multi(&search_paths, old, new, &plan_options)?;

    // Check if there's anything to do
    if plan.stats.total_matches == 0 && plan.renames.is_empty() {
        return Ok("Nothing to do.".to_string());
    }

    // Apply the plan
    let refaktor_dir = std::env::current_dir()?.join(".refaktor");
    let apply_options = ApplyOptions {
        backup_dir: refaktor_dir.join("backups"),
        create_backups: true,
        atomic: true,
        commit,
        force: force_with_conflicts,
        skip_symlinks: true,
        log_file: None,
    };

    apply_plan(&plan, &apply_options)?;

    Ok(format!("Successfully applied refactoring '{}' -> '{}'", old, new))
}

/// High-level API for undo operations - equivalent to `refaktor undo` command
pub fn undo(id: &str) -> Result<String> {
    let refaktor_dir = std::env::current_dir()?.join(".refaktor");
    
    // Handle "latest" shortcut
    let actual_id = if id == "latest" {
        let history = crate::history::History::load(&refaktor_dir)?;
        let entries = history.list_entries(None);
        if entries.is_empty() {
            return Err(anyhow::anyhow!("No refactoring history found"));
        }
        entries[0].id.clone()
    } else {
        id.to_string()
    };

    undo_refactoring(&actual_id, &refaktor_dir)?;
    
    Ok(format!("Successfully undid refactoring '{}'", actual_id))
}

/// High-level API for plan operations - equivalent to `refaktor plan` command
pub fn plan(
    old: &str,
    new: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    unrestricted_level: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<String>,
    include_styles: Vec<String>,
    only_styles: Vec<String>, 
    exclude_match: Vec<String>,
    plan_out: Option<PathBuf>,
) -> Result<String> {
    // Convert style args - for now just use None to get defaults
    let styles = if only_styles.is_empty() && include_styles.is_empty() && exclude_styles.is_empty() {
        None
    } else {
        // TODO: implement proper style conversion
        None
    };

    // Use current directory if no paths provided
    let search_paths = if paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        paths
    };

    let plan_options = PlanOptions {
        includes: include,
        excludes: exclude,
        respect_gitignore: true,
        unrestricted_level,
        styles,
        rename_files,
        rename_dirs,
        rename_root: false,
        plan_out: plan_out.unwrap_or_else(|| PathBuf::from(".refaktor/plan.json")),
        coerce_separators: crate::scanner::CoercionMode::Auto,
        exclude_match,
    };

    let plan = scan_repository_multi(&search_paths, old, new, &plan_options)?;
    
    Ok(format!("Generated plan with {} matches and {} renames", 
               plan.stats.total_matches, plan.renames.len()))
}

/// High-level API for apply operations - equivalent to `refaktor apply` command  
pub fn apply(
    plan_path: Option<PathBuf>,
    plan_id: Option<String>,
    commit: bool,
    force: bool,
) -> Result<String> {
    // TODO: implement proper plan loading and application
    // For now this is a placeholder
    Err(anyhow::anyhow!("Apply API not yet implemented"))
}