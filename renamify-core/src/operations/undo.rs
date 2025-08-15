use crate::id_resolver::{resolve_id, OperationType};
use crate::{redo_refactoring, undo_refactoring};
use anyhow::Result;
use std::path::Path;

/// High-level undo operation - equivalent to `renamify undo` command
pub fn undo_operation(id: &str, working_dir: Option<&Path>) -> Result<String> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Resolve the ID (handles "latest" and validates the ID exists)
    let actual_id = resolve_id(id, OperationType::Undo, &renamify_dir)?;

    undo_refactoring(&actual_id, &renamify_dir)?;

    Ok(format!("Successfully undid refactoring '{}'", actual_id))
}

/// High-level redo operation - equivalent to `renamify redo` command
pub fn redo_operation(id: &str, working_dir: Option<&Path>) -> Result<String> {
    let current_dir = working_dir.unwrap_or_else(|| Path::new("."));
    let renamify_dir = current_dir.join(".renamify");

    // Resolve the ID (handles "latest" and validates the ID exists)
    let actual_id = resolve_id(id, OperationType::Redo, &renamify_dir)?;

    redo_refactoring(&actual_id, &renamify_dir)?;

    Ok(format!("Successfully redid refactoring '{}'", actual_id))
}
