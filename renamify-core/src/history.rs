use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// Represents a single entry in the renaming history
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HistoryEntry {
    /// Unique identifier for this plan/operation
    pub id: String,
    /// Timestamp when the operation was performed
    pub created_at: String,
    /// Original identifier that was replaced
    pub search: String,
    /// New identifier that replaced the old one
    pub replace: String,
    /// Naming styles used for the transformation
    pub styles: Vec<String>,
    /// Include patterns used for file filtering
    pub includes: Vec<String>,
    /// Exclude patterns used for file filtering
    pub excludes: Vec<String>,
    /// Files that were modified (path -> checksum after apply)
    #[ts(type = "Record<string, string>")]
    pub affected_files: HashMap<PathBuf, String>,
    /// Renames that were performed (from -> to)
    #[ts(type = "Array<[string, string]>")]
    pub renames: Vec<(PathBuf, PathBuf)>,
    /// Path to the backup directory for this operation
    #[ts(type = "string")]
    pub backups_path: PathBuf,
    /// If this is a revert operation, the ID of the operation being reverted
    #[ts(optional)]
    pub revert_of: Option<String>,
    /// If this is a redo operation, the ID of the original operation
    #[ts(optional)]
    pub redo_of: Option<String>,
}

/// Manages the renaming history
pub struct History {
    path: PathBuf,
    entries: Vec<HistoryEntry>,
}

impl History {
    /// Load history from the default location
    pub fn load(renamify_dir: &Path) -> Result<Self> {
        let path = renamify_dir.join("history.json");
        Self::load_from_path(&path)
    }

    /// Load history from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let entries = if path.exists() {
            let file = File::open(path)
                .with_context(|| format!("Failed to open history file: {}", path.display()))?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)
                .with_context(|| format!("Failed to parse history file: {}", path.display()))?
        } else {
            Vec::new()
        };

        Ok(Self {
            path: path.to_path_buf(),
            entries,
        })
    }

    /// Save the history to disk
    pub fn save(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .with_context(|| format!("Failed to create history file: {}", self.path.display()))?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.entries)
            .with_context(|| format!("Failed to write history file: {}", self.path.display()))?;

        Ok(())
    }

    /// Add a new entry to the history
    pub fn add_entry(&mut self, entry: HistoryEntry) -> Result<()> {
        // Check for duplicate IDs
        if self.entries.iter().any(|e| e.id == entry.id) {
            return Err(anyhow!("History entry with ID {} already exists", entry.id));
        }

        self.entries.push(entry);
        self.save()?;
        Ok(())
    }

    /// Find an entry by ID
    pub fn find_entry(&self, id: &str) -> Option<&HistoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get the last entry
    pub fn last_entry(&self) -> Option<&HistoryEntry> {
        self.entries.last()
    }

    /// Get all entries, optionally limited to the most recent N
    pub fn list_entries(&self, limit: Option<usize>) -> Vec<&HistoryEntry> {
        let entries: Vec<_> = self.entries.iter().rev().collect();
        if let Some(limit) = limit {
            entries.into_iter().take(limit).collect()
        } else {
            entries
        }
    }

    /// Check if there are any pending conflicts
    pub fn has_pending_conflicts(&self, conflicts_dir: &Path) -> bool {
        if let Some(last) = self.last_entry() {
            let conflict_file = conflicts_dir.join(format!("{}.json", last.id));
            conflict_file.exists()
        } else {
            false
        }
    }

    /// Verify checksums of files from a history entry
    pub fn verify_checksums(entry: &HistoryEntry) -> Result<Vec<PathBuf>> {
        let mut mismatches = Vec::new();

        for (path, expected_checksum) in &entry.affected_files {
            if path.exists() {
                let actual_checksum = crate::apply::calculate_checksum(path)?;
                if actual_checksum != *expected_checksum {
                    mismatches.push(path.clone());
                }
            } else {
                // File doesn't exist - also a mismatch
                mismatches.push(path.clone());
            }
        }

        Ok(mismatches)
    }

    /// Prune history to keep it under a certain size
    pub fn prune(&mut self, max_entries: usize) -> Result<()> {
        if self.entries.len() > max_entries {
            // Keep only the most recent entries
            let to_remove = self.entries.len() - max_entries;
            self.entries.drain(0..to_remove);
            self.save()?;
        }
        Ok(())
    }
}

/// Create a history entry from an applied plan
pub fn create_history_entry<S: ::std::hash::BuildHasher>(
    plan: &crate::Plan,
    affected_files: HashMap<PathBuf, String, S>,
    paths: Vec<(PathBuf, PathBuf)>,
    backups_path: PathBuf,
    revert_of: Option<String>,
    redo_of: Option<String>,
) -> HistoryEntry {
    HistoryEntry {
        id: plan.id.clone(),
        created_at: chrono::Local::now().to_rfc3339(),
        search: plan.search.clone(),
        replace: plan.replace.clone(),
        styles: plan.styles.iter().map(|s| format!("{:?}", s)).collect(),
        includes: plan.includes.clone(),
        excludes: plan.excludes.clone(),
        affected_files: affected_files.into_iter().collect(),
        renames: paths,
        backups_path,
        revert_of,
        redo_of,
    }
}

/// Format history entries for display
pub fn format_history(entries: &[&HistoryEntry], json: bool) -> Result<String> {
    if json {
        Ok(serde_json::to_string_pretty(entries)?)
    } else {
        use comfy_table::{Cell, Color, Table};

        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("ID").fg(Color::Cyan),
            Cell::new("Date").fg(Color::Cyan),
            Cell::new("Rename").fg(Color::Cyan),
            Cell::new("Files").fg(Color::Cyan),
            Cell::new("Renames").fg(Color::Cyan),
            Cell::new("Type").fg(Color::Cyan),
        ]);

        for entry in entries {
            let date = entry
                .created_at
                .split('T')
                .next()
                .unwrap_or(&entry.created_at);
            let rename = format!("{} → {}", entry.search, entry.replace);
            let files = entry.affected_files.len();
            let renames = entry.renames.len();

            let entry_type = if entry.revert_of.is_some() {
                "revert"
            } else if entry.redo_of.is_some() {
                "redo"
            } else {
                "apply"
            };

            table.add_row(vec![
                &entry.id[..8.min(entry.id.len())], // Show first 8 chars of ID
                date,
                &rename,
                &files.to_string(),
                &renames.to_string(),
                entry_type,
            ]);
        }

        Ok(table.to_string())
    }
}

/// Get the status of the working directory
pub fn get_status(renamify_dir: &Path) -> Result<StatusInfo> {
    let history = History::load(renamify_dir)?;
    let conflicts_dir = renamify_dir.join("conflicts");

    let last_plan = history.last_entry().map(|e| e.id.clone());
    let has_conflicts = history.has_pending_conflicts(&conflicts_dir);

    // Check if working tree is clean according to our records
    let working_tree_clean = if let Some(last) = history.last_entry() {
        History::verify_checksums(last)?.is_empty()
    } else {
        true // No history means clean
    };

    Ok(StatusInfo {
        last_plan,
        has_conflicts,
        working_tree_clean,
        total_entries: history.entries.len(),
    })
}

/// Status information
#[derive(Debug, Serialize)]
pub struct StatusInfo {
    pub last_plan: Option<String>,
    pub has_conflicts: bool,
    pub working_tree_clean: bool,
    pub total_entries: usize,
}

impl StatusInfo {
    /// Format status for display
    pub fn format(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        if let Some(ref plan_id) = self.last_plan {
            use std::fmt::Write;
            writeln!(output, "Last applied plan: {}", plan_id).unwrap();
        } else {
            output.push_str("No plans applied yet\n");
        }

        if self.has_conflicts {
            output.push_str("⚠️  Pending conflicts detected\n");
        }

        if self.working_tree_clean {
            output.push_str("✓ Working tree is clean\n");
        } else {
            output.push_str("⚠️  Working tree has been modified since last apply\n");
        }

        writeln!(output, "Total history entries: {}", self.total_entries).unwrap();

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            search: "old_name".to_string(),
            replace: "new_name".to_string(),
            styles: vec!["Snake".to_string()],
            includes: vec![],
            excludes: vec![],
            affected_files: HashMap::new(),
            renames: vec![],
            backups_path: PathBuf::from("/tmp/backups"),
            revert_of: None,
            redo_of: None,
        }
    }

    #[test]
    fn test_history_add_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        // Create and save history
        let mut history = History::load_from_path(&history_path).unwrap();
        let entry = create_test_entry("test123");
        history.add_entry(entry).unwrap();

        // Load and verify
        let loaded = History::load_from_path(&history_path).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].id, "test123");
    }

    #[test]
    fn test_find_entry() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let mut history = History::load_from_path(&history_path).unwrap();
        history.add_entry(create_test_entry("test1")).unwrap();
        history.add_entry(create_test_entry("test2")).unwrap();

        assert!(history.find_entry("test1").is_some());
        assert!(history.find_entry("test2").is_some());
        assert!(history.find_entry("test3").is_none());
    }

    #[test]
    fn test_duplicate_id_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let mut history = History::load_from_path(&history_path).unwrap();
        history.add_entry(create_test_entry("test1")).unwrap();

        // Try to add duplicate
        let result = history.add_entry(create_test_entry("test1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_entries_with_limit() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let mut history = History::load_from_path(&history_path).unwrap();
        for i in 0..5 {
            history
                .add_entry(create_test_entry(&format!("test{}", i)))
                .unwrap();
        }

        let all = history.list_entries(None);
        assert_eq!(all.len(), 5);

        let limited = history.list_entries(Some(3));
        assert_eq!(limited.len(), 3);
        assert_eq!(limited[0].id, "test4"); // Most recent first
    }

    #[test]
    fn test_prune() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let mut history = History::load_from_path(&history_path).unwrap();
        for i in 0..10 {
            history
                .add_entry(create_test_entry(&format!("test{}", i)))
                .unwrap();
        }

        history.prune(5).unwrap();
        assert_eq!(history.entries.len(), 5);
        assert_eq!(history.entries[0].id, "test5"); // Oldest kept
    }

    #[test]
    fn test_status_format() {
        let status = StatusInfo {
            last_plan: Some("abc123".to_string()),
            has_conflicts: false,
            working_tree_clean: true,
            total_entries: 5,
        };

        let formatted = status.format();
        assert!(formatted.contains("Last applied plan: abc123"));
        assert!(formatted.contains("Working tree is clean"));
        assert!(formatted.contains("Total history entries: 5"));
    }
}
