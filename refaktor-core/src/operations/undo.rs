use crate::{undo_refactoring, History};
use anyhow::{anyhow, Result};
use std::path::Path;

/// High-level undo operation - equivalent to `refaktor undo` command
pub fn undo_operation(id: &str, working_dir: Option<&Path>) -> Result<String> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let refaktor_dir = current_dir.join(".refaktor");

    // Handle "latest" shortcut
    let actual_id = if id == "latest" {
        let history = History::load(&refaktor_dir)?;
        let entries = history.list_entries(None);
        if entries.is_empty() {
            return Err(anyhow!("No refactoring history found"));
        }
        // Find the first entry that is not already a revert
        let latest = entries
            .iter()
            .find(|entry| entry.revert_of.is_none())
            .ok_or_else(|| anyhow!("No refactoring entries found that can be undone"))?;
        latest.id.clone()
    } else {
        id.to_string()
    };

    undo_refactoring(&actual_id, &refaktor_dir)?;

    Ok(format!("Successfully undid refactoring '{}'", actual_id))
}

/// High-level redo operation - equivalent to `refaktor redo` command
pub fn redo_operation(id: &str, working_dir: Option<&Path>) -> Result<String> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let refaktor_dir = current_dir.join(".refaktor");

    crate::redo_refactoring(id, &refaktor_dir)?;

    Ok(format!("Successfully redid refactoring '{}'", id))
}
