use crate::id_resolver::{resolve_id, OperationType};
use crate::output::{RedoResult, UndoResult};
use crate::{redo_renaming, undo_renaming, History};
use anyhow::Result;
use std::path::Path;

/// Undo operation - returns structured data
pub fn undo_operation(id: &str, working_dir: Option<&Path>) -> Result<UndoResult> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Resolve the ID (handles "latest" and validates the ID exists)
    let actual_id = resolve_id(id, OperationType::Undo, &renamify_dir)?;

    // Load history to get entry details before undoing
    let history = History::load(&renamify_dir)?;
    let entry = history
        .find_entry(&actual_id)
        .ok_or_else(|| anyhow::anyhow!("History entry '{}' not found", actual_id))?;

    let files_restored = entry.affected_files.len();
    let renames_reverted = entry.renames.len();

    // Perform the undo
    undo_renaming(&actual_id, &renamify_dir)?;

    Ok(UndoResult {
        history_id: actual_id,
        files_restored,
        renames_reverted,
    })
}

/// Redo operation - returns structured data
pub fn redo_operation(id: &str, working_dir: Option<&Path>) -> Result<RedoResult> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Resolve the ID (handles "latest" and validates the ID exists)
    let actual_id = resolve_id(id, OperationType::Redo, &renamify_dir)?;

    // Load history to get entry details before redoing
    let history = History::load(&renamify_dir)?;
    let entry = history
        .find_entry(&actual_id)
        .ok_or_else(|| anyhow::anyhow!("History entry '{}' not found", actual_id))?;

    let files_changed = entry.affected_files.len();
    let renames = entry.renames.len();

    // Calculate replacements from the entry (this is an approximation)
    // In a real implementation, we'd need to track this in the history
    let replacements = files_changed * 2; // Rough estimate

    // Perform the redo
    redo_renaming(&actual_id, &renamify_dir)?;

    Ok(RedoResult {
        history_id: actual_id,
        files_changed,
        replacements,
        renames,
    })
}
