use crate::history::History;
use anyhow::{anyhow, Result};
use std::path::Path;

/// Type of operation for ID resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Undo,
    Redo,
}

/// Resolve an ID (which might be "latest") to an actual history entry ID
pub fn resolve_id(id: &str, operation: OperationType, renamify_dir: &Path) -> Result<String> {
    if id == "latest" {
        resolve_latest_id(operation, renamify_dir)
    } else {
        // Verify the ID exists
        let history = History::load(renamify_dir)?;
        if history.find_entry(id).is_none() {
            return Err(anyhow!("History entry '{}' not found", id));
        }
        Ok(id.to_string())
    }
}

/// Resolve "latest" to the most recent applicable entry ID
fn resolve_latest_id(operation: OperationType, renamify_dir: &Path) -> Result<String> {
    let history = History::load(renamify_dir)?;
    let entries = history.list_entries(None);

    if entries.is_empty() {
        return Err(anyhow!("No refactoring history found"));
    }

    match operation {
        OperationType::Undo => {
            // Find the most recent entry that is not already a revert
            // (i.e., an entry that can be undone)
            entries
                .iter()
                .find(|entry| entry.revert_of.is_none())
                .map(|entry| entry.id.clone())
                .ok_or_else(|| anyhow!("No refactoring entries found that can be undone"))
        },
        OperationType::Redo => {
            // Find the most recent revert entry
            // (i.e., an entry that represents an undo operation)
            entries
                .iter()
                .find(|entry| entry.revert_of.is_some())
                .map(|entry| {
                    // Return the ID of the original operation that was undone
                    entry.revert_of.as_ref().unwrap().clone()
                })
                .ok_or_else(|| anyhow!("No undone refactoring entries found that can be redone"))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{History, HistoryEntry};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_entry(id: &str, revert_of: Option<String>) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            affected_files: std::collections::HashMap::new(),
            renames: vec![],
            backups_path: std::path::PathBuf::from("/tmp/backups"),
            revert_of,
            redo_of: None,
        }
    }

    fn setup_test_history(entries: Vec<HistoryEntry>) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        // Create an empty history file first
        let history_path = renamify_dir.join("history.json");
        fs::write(&history_path, "[]").unwrap();

        // Load and populate the history
        let mut history = History::load(&renamify_dir).unwrap();
        for entry in entries {
            history.add_entry(entry).unwrap();
        }

        temp_dir
    }

    #[test]
    fn test_resolve_specific_id_exists() {
        let temp_dir = setup_test_history(vec![
            create_test_entry("abc123", None),
            create_test_entry("def456", None),
        ]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // Should resolve to the same ID for both operations
        let result = resolve_id("abc123", OperationType::Undo, &renamify_dir).unwrap();
        assert_eq!(result, "abc123");

        let result = resolve_id("def456", OperationType::Redo, &renamify_dir).unwrap();
        assert_eq!(result, "def456");
    }

    #[test]
    fn test_resolve_specific_id_missing() {
        let temp_dir = setup_test_history(vec![create_test_entry("abc123", None)]);
        let renamify_dir = temp_dir.path().join(".renamify");

        let result = resolve_id("missing", OperationType::Undo, &renamify_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_resolve_latest_for_undo() {
        let temp_dir = setup_test_history(vec![
            create_test_entry("first", None),
            create_test_entry("second", None),
            create_test_entry("revert_second", Some("second".to_string())),
            create_test_entry("third", None),
        ]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // Latest for undo should be the most recent non-revert entry
        let result = resolve_id("latest", OperationType::Undo, &renamify_dir).unwrap();
        assert_eq!(result, "third");
    }

    #[test]
    fn test_resolve_latest_for_redo() {
        let temp_dir = setup_test_history(vec![
            create_test_entry("first", None),
            create_test_entry("second", None),
            create_test_entry("revert_second", Some("second".to_string())),
            create_test_entry("third", None),
            create_test_entry("revert_third", Some("third".to_string())),
        ]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // Latest for redo should return the ID of the original operation that was most recently undone
        let result = resolve_id("latest", OperationType::Redo, &renamify_dir).unwrap();
        assert_eq!(result, "third");
    }

    #[test]
    fn test_resolve_latest_for_undo_only_reverts() {
        // Test where the only entries in history are revert operations
        let temp_dir = setup_test_history(vec![
            create_test_entry("revert1", Some("original1".to_string())),
            create_test_entry("revert2", Some("original2".to_string())),
        ]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // Only revert entries exist, so nothing can be undone
        let result = resolve_id("latest", OperationType::Undo, &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No refactoring entries found that can be undone"));
    }

    #[test]
    fn test_resolve_latest_for_redo_no_reverts() {
        let temp_dir = setup_test_history(vec![
            create_test_entry("first", None),
            create_test_entry("second", None),
        ]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // No revert entries exist, so nothing can be redone
        let result = resolve_id("latest", OperationType::Redo, &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No undone refactoring entries found"));
    }

    #[test]
    fn test_resolve_latest_empty_history() {
        let temp_dir = setup_test_history(vec![]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // Empty history for undo
        let result = resolve_id("latest", OperationType::Undo, &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No refactoring history found"));

        // Empty history for redo
        let result = resolve_id("latest", OperationType::Redo, &renamify_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No refactoring history found"));
    }

    #[test]
    fn test_resolve_latest_complex_history() {
        // Complex scenario with multiple operations and undos
        let temp_dir = setup_test_history(vec![
            create_test_entry("op1", None),
            create_test_entry("op2", None),
            create_test_entry("revert_op2", Some("op2".to_string())),
            create_test_entry("op3", None),
            create_test_entry("revert_op3", Some("op3".to_string())),
            create_test_entry("op4", None),
            create_test_entry("revert_op1", Some("op1".to_string())),
            create_test_entry("op5", None),
        ]);
        let renamify_dir = temp_dir.path().join(".renamify");

        // Latest for undo should be op5 (the most recent non-revert)
        let result = resolve_id("latest", OperationType::Undo, &renamify_dir).unwrap();
        assert_eq!(result, "op5");

        // Latest for redo should be op1 (most recently undone)
        let result = resolve_id("latest", OperationType::Redo, &renamify_dir).unwrap();
        assert_eq!(result, "op1");
    }
}
