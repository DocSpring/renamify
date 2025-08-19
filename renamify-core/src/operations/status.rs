use crate::output::{PendingPlan, StatusResult};
use crate::History;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Status operation - returns structured data
pub fn status_operation(working_dir: Option<&Path>) -> Result<StatusResult> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Check for pending plan
    let plan_path = renamify_dir.join("plan.json");
    let pending_plan = if plan_path.exists() {
        let content = fs::read_to_string(&plan_path)?;
        let plan: crate::scanner::Plan = serde_json::from_str(&content)?;
        Some(PendingPlan {
            id: plan.id.clone(),
            search: plan.search.clone(),
            replace: plan.replace.clone(),
            created_at: plan.created_at.clone(),
        })
    } else {
        None
    };

    // Load history to get counts
    let history = History::load(&renamify_dir)?;
    let entries = history.list_entries(None);
    let history_count = entries.len();

    // Get last operation if any
    let last_operation = history
        .last_entry()
        .map(|e| format!("{} ({} -> {})", e.id, e.search, e.replace));

    Ok(StatusResult {
        pending_plan,
        history_count,
        last_operation,
    })
}
