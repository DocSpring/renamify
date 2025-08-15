use crate::{apply_plan, scanner::Plan, ApplyOptions};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// High-level apply operation - equivalent to `renamify apply` command  
pub fn apply_operation(
    plan_path: Option<PathBuf>,
    plan_id: Option<String>,
    commit: bool,
    force: bool,
    working_dir: Option<&Path>,
) -> Result<String> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Load the plan and track if using default plan file
    let (mut plan, used_default_plan_file) =
        load_plan_from_source_with_tracking(plan_path, plan_id, &renamify_dir)?;

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: renamify_dir.join("backups"),
        create_backups: true,
        atomic: true,
        commit,
        force,
        skip_symlinks: false,
        log_file: Some(renamify_dir.join("logs").join(format!("{}.log", plan.id))),
    };

    apply_plan(&mut plan, &apply_options)?;

    // Write success message to stderr
    eprintln!("Plan applied successfully!");

    // Delete the plan.json file after successful apply (only if using default path)
    if let Some(default_plan_path) = used_default_plan_file {
        if let Err(e) = fs::remove_file(&default_plan_path) {
            eprintln!(
                "Warning: Failed to delete plan file {}: {}",
                default_plan_path.display(),
                e
            );
        }
    }

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

    result.push_str(&format!("\nUndo with: renamify undo {}", plan.id));

    Ok(result)
}

fn load_plan_from_source_with_tracking(
    plan_path: Option<PathBuf>,
    plan_id: Option<String>,
    renamify_dir: &Path,
) -> Result<(Plan, Option<PathBuf>)> {
    match (plan_path, plan_id) {
        (Some(path), _) => {
            // External call with explicit path - don't delete afterwards
            let plan_content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read plan from {}", path.display()))?;
            let plan = serde_json::from_str(&plan_content)
                .with_context(|| format!("Failed to parse plan from {}", path.display()))?;
            Ok((plan, None))
        },
        (None, Some(id)) => {
            // Positional argument provided - determine what it is
            if id.ends_with(".json") || Path::new(&id).exists() {
                // It's a file path
                let path = PathBuf::from(id);
                let plan_content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read plan from {}", path.display()))?;
                let plan = serde_json::from_str(&plan_content)
                    .with_context(|| format!("Failed to parse plan from {}", path.display()))?;
                Ok((plan, None))
            } else if id == "latest" {
                // "latest" means use the default plan.json
                let plan_file = renamify_dir.join("plan.json");
                if !plan_file.exists() {
                    return Err(anyhow!("No default plan found at {}", plan_file.display()));
                }
                let plan_content =
                    fs::read_to_string(&plan_file).context("Failed to read default plan")?;
                let plan =
                    serde_json::from_str(&plan_content).context("Failed to parse default plan")?;
                Ok((plan, Some(plan_file)))
            } else {
                // It's a plan ID - load from plans directory
                let plans_dir = renamify_dir.join("plans");
                let plan_file = plans_dir.join(format!("{}.json", id));
                if !plan_file.exists() {
                    return Err(anyhow!("Plan file not found: {}", plan_file.display()));
                }
                let plan_content = fs::read_to_string(&plan_file)
                    .with_context(|| format!("Failed to read plan {}", id))?;
                let plan = serde_json::from_str(&plan_content)
                    .with_context(|| format!("Failed to parse plan {}", id))?;
                Ok((plan, None))
            }
        },
        (None, None) => {
            // Default: load most recent plan - DELETE this file after success
            let plan_file = renamify_dir.join("plan.json");
            if !plan_file.exists() {
                return Err(anyhow!(
                    "No plan specified and no default plan found at {}",
                    plan_file.display()
                ));
            }
            let plan_content =
                fs::read_to_string(&plan_file).context("Failed to read default plan")?;
            let plan =
                serde_json::from_str(&plan_content).context("Failed to parse default plan")?;
            Ok((plan, Some(plan_file)))
        },
    }
}
