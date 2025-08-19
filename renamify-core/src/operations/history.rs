use crate::output::{HistoryItem, HistoryResult};
use crate::History;
use anyhow::Result;
use std::path::Path;

/// History operation - returns structured data
pub fn history_operation(
    limit: Option<usize>,
    working_dir: Option<&Path>,
) -> Result<HistoryResult> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Load history
    let history = History::load(&renamify_dir)?;
    let entries = history.list_entries(limit);

    // Convert to output format
    let items: Vec<HistoryItem> = entries
        .into_iter()
        .map(|e| HistoryItem {
            id: e.id.clone(),
            operation: if e.revert_of.is_some() {
                "undo".to_string()
            } else if e.redo_of.is_some() {
                "redo".to_string()
            } else {
                "apply".to_string()
            },
            timestamp: e.created_at.clone(),
            search: e.search.clone(),
            replace: e.replace.clone(),
            files_changed: e.affected_files.len(),
            replacements: e.affected_files.len() * 2, // Estimate
            renames: e.renames.len(),
            reverted: e.revert_of.is_some(),
        })
        .collect();

    Ok(HistoryResult { entries: items })
}
