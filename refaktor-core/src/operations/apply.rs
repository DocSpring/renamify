use anyhow::{anyhow, Context, Result};
use crate::{apply_plan, scanner::Plan, ApplyOptions};
use std::fs;
use std::path::{Path, PathBuf};

/// High-level apply operation - equivalent to `refaktor apply` command  
pub fn apply_operation(
    plan_path: Option<PathBuf>,
    plan_id: Option<String>,
    commit: bool,
    force: bool,
    working_dir: Option<&Path>,
) -> Result<String> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let refaktor_dir = current_dir.join(".refaktor");

    // Load the plan
    let plan = load_plan_from_source(plan_path, plan_id, &refaktor_dir)?;

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: refaktor_dir.join("backups"),
        create_backups: true,
        atomic: true,
        commit,
        force,
        skip_symlinks: false,
        log_file: Some(refaktor_dir.join("logs").join(format!("{}.log", plan.id))),
    };

    apply_plan(&plan, &apply_options)?;

    let mut result = format!(
        "✓ Applied {} replacements across {} files",
        plan.stats.total_matches, plan.stats.files_with_matches
    );
    
    if !plan.renames.is_empty() {
        result.push_str(&format!("\n✓ Renamed {} items", plan.renames.len()));
    }
    
    if commit {
        result.push_str("\n✓ Changes committed to git");
    }
    
    result.push_str(&format!("\nUndo with: refaktor undo {}", plan.id));

    Ok(result)
}

fn load_plan_from_source(
    plan_path: Option<PathBuf>,
    plan_id: Option<String>,
    refaktor_dir: &Path,
) -> Result<Plan> {
    match (plan_path, plan_id) {
        (Some(path), None) => {
            // Load from specific file path
            let plan_content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read plan from {}", path.display()))?;
            serde_json::from_str(&plan_content)
                .with_context(|| format!("Failed to parse plan from {}", path.display()))
        },
        (None, Some(id)) => {
            // Load from ID in plans directory
            let plans_dir = refaktor_dir.join("plans");
            let plan_file = plans_dir.join(format!("{}.json", id));
            if !plan_file.exists() {
                return Err(anyhow!("Plan file not found: {}", plan_file.display()));
            }
            let plan_content = fs::read_to_string(&plan_file)
                .with_context(|| format!("Failed to read plan {}", id))?;
            serde_json::from_str(&plan_content)
                .with_context(|| format!("Failed to parse plan {}", id))
        },
        (Some(_), Some(_)) => {
            Err(anyhow!("Cannot specify both plan_path and plan_id"))
        },
        (None, None) => {
            // Default: load most recent plan
            let plan_file = refaktor_dir.join("plan.json");
            if !plan_file.exists() {
                return Err(anyhow!("No plan specified and no default plan found at {}", plan_file.display()));
            }
            let plan_content = fs::read_to_string(&plan_file)
                .context("Failed to read default plan")?;
            serde_json::from_str(&plan_content)
                .context("Failed to parse default plan")
        }
    }
}