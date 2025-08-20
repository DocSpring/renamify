use crate::{apply_plan, output::ApplyResult, scanner::Plan, ApplyOptions};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Apply operation - returns structured data
pub fn apply_operation(
    _plan_path: Option<&Path>,
    plan_id: Option<&str>,
    commit: bool,
    force: bool,
    working_dir: Option<&Path>,
) -> Result<ApplyResult> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Load the plan - check if plan_id looks like a path
    let (plan_path, plan_id) = if let Some(id) = plan_id {
        if id.contains('/')
            || Path::new(id)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            // It's a path
            (Some(PathBuf::from(id)), None)
        } else {
            // It's an ID
            (None, Some(id))
        }
    } else {
        (None, None)
    };

    let (mut plan, used_default_plan_file) =
        load_plan_from_source_with_tracking(plan_path, plan_id, &renamify_dir)?;

    // Save stats before applying
    let files_changed = plan.stats.files_with_matches;
    let replacements = plan.stats.total_matches;
    let renames = plan.paths.len();
    let plan_id = plan.id.clone();

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

    Ok(ApplyResult {
        plan_id,
        files_changed,
        replacements,
        renames,
        committed: commit,
    })
}

fn load_plan_from_source_with_tracking(
    plan_path: Option<PathBuf>,
    plan_id: Option<&str>,
    renamify_dir: &Path,
) -> Result<(Plan, Option<PathBuf>)> {
    match (plan_path, plan_id) {
        (Some(path), None) => {
            // Load from specified path
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read plan file {}", path.display()))?;
            let plan = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse plan file {}", path.display()))?;
            Ok((plan, None))
        },
        (None, Some(id)) => {
            // For plan IDs, we need to look for the plan file in .renamify/plans/
            // The history only stores metadata, not the full plan
            let plan_path = renamify_dir.join("plans").join(format!("{}.json", id));
            if !plan_path.exists() {
                return Err(anyhow!("Plan with ID {} not found", id));
            }

            let content = fs::read_to_string(&plan_path)?;
            let plan = serde_json::from_str(&content)?;
            Ok((plan, None))
        },
        (None, None) => {
            // Load from default plan.json
            let default_plan_path = renamify_dir.join("plan.json");
            if !default_plan_path.exists() {
                return Err(anyhow!(
                    "No plan file found. Create one with 'renamify plan' first."
                ));
            }

            let content = fs::read_to_string(&default_plan_path)?;
            let plan = serde_json::from_str(&content)?;
            Ok((plan, Some(default_plan_path)))
        },
        (Some(_), Some(_)) => Err(anyhow!("Cannot specify both plan path and plan ID")),
    }
}
