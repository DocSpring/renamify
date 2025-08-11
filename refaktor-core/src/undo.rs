use crate::apply::{apply_plan, calculate_checksum, ApplyOptions};
use crate::history::{create_history_entry, History};
use crate::scanner::{MatchHunk, Plan, Rename, RenameKind, Stats};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Undo a previously applied refactoring
pub fn undo_refactoring(id: &str, refaktor_dir: &Path) -> Result<()> {
    let mut history = History::load(refaktor_dir)?;
    
    // Find the entry to undo
    let entry = history
        .find_entry(id)
        .ok_or_else(|| anyhow!("History entry '{}' not found", id))?
        .clone();
    
    // Check if this was already reverted
    if entry.revert_of.is_some() {
        return Err(anyhow!("Entry '{}' is already a revert operation", id));
    }
    
    // Check if any later entry was already reverted
    let has_later_revert = history
        .list_entries(None)
        .iter()
        .any(|e| e.revert_of.as_ref() == Some(&entry.id));
    
    if has_later_revert {
        return Err(anyhow!("Entry '{}' has already been reverted", id));
    }
    
    eprintln!("Undoing refactoring '{}'...", id);
    
    // Restore files from backups
    let mut restored_files = Vec::new();
    for (path, _checksum) in &entry.affected_files {
        let backup_path = entry.backups_path.join(
            path.strip_prefix("/").unwrap_or(path)
        );
        
        if backup_path.exists() {
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Restore the file
            fs::copy(&backup_path, path)
                .with_context(|| format!("Failed to restore {} from backup", path.display()))?;
            
            restored_files.push(path.clone());
            eprintln!("  Restored: {}", path.display());
        }
    }
    
    // Reverse renames (to -> from)
    let mut reversed_renames = Vec::new();
    for (from, to) in entry.renames.iter().rev() {
        if to.exists() {
            // Handle case-only renames on case-insensitive filesystems
            let case_only = from.to_string_lossy().to_lowercase() == to.to_string_lossy().to_lowercase()
                && from != to;
            
            if case_only && crate::rename::detect_case_insensitive_fs(to.parent().unwrap_or(Path::new("."))) {
                // Two-step rename for case-only changes
                let temp_name = to.with_extension(format!("{}.refaktor.tmp", std::process::id()));
                fs::rename(to, &temp_name)?;
                fs::rename(&temp_name, from)?;
            } else {
                fs::rename(to, from)?;
            }
            
            reversed_renames.push((to.clone(), from.clone()));
            eprintln!("  Renamed: {} -> {}", to.display(), from.display());
        }
    }
    
    // Calculate checksums of restored files
    let mut affected_files = HashMap::new();
    for path in &restored_files {
        if path.exists() && path.is_file() {
            let checksum = calculate_checksum(path)?;
            affected_files.insert(path.clone(), checksum);
        }
    }
    
    // Create a revert history entry
    let revert_entry = crate::history::HistoryEntry {
        id: format!("revert-{}-{}", entry.id, chrono::Local::now().timestamp()),
        created_at: chrono::Local::now().to_rfc3339(),
        old: entry.new.clone(), // Swap old and new
        new: entry.old.clone(),
        styles: entry.styles.clone(),
        includes: entry.includes.clone(),
        excludes: entry.excludes.clone(),
        affected_files,
        renames: reversed_renames,
        backups_path: entry.backups_path.clone(), // Keep same backup path
        revert_of: Some(entry.id.clone()),
        redo_of: None,
    };
    
    history.add_entry(revert_entry)?;
    
    eprintln!("Successfully undid refactoring '{}'", id);
    Ok(())
}

/// Redo a previously undone refactoring
pub fn redo_refactoring(id: &str, refaktor_dir: &Path) -> Result<()> {
    let history = History::load(refaktor_dir)?;
    
    // Find the original entry
    let entry = history
        .find_entry(id)
        .ok_or_else(|| anyhow!("History entry '{}' not found", id))?;
    
    // Check if this entry was reverted
    let entries = history.list_entries(None);
    let revert_entry = entries
        .iter()
        .find(|e| e.revert_of.as_ref() == Some(&entry.id));
    
    if revert_entry.is_none() {
        return Err(anyhow!("Entry '{}' has not been reverted", id));
    }
    
    eprintln!("Redoing refactoring '{}'...", id);
    
    // Create a plan from the history entry
    let plan = history_entry_to_plan(entry)?;
    
    // Apply the plan again
    let options = ApplyOptions {
        backup_dir: refaktor_dir.join("backups"),
        create_backups: true,
        atomic: true,
        ..Default::default()
    };
    
    apply_plan(&plan, &options)?;
    
    eprintln!("Successfully redid refactoring '{}'", id);
    Ok(())
}

/// Convert a history entry back to a plan for re-application
fn history_entry_to_plan(entry: &crate::history::HistoryEntry) -> Result<Plan> {
    // Create a minimal plan from the history entry
    // This won't have all the original match details, but has enough for redo
    let mut plan = Plan {
        id: format!("redo-{}-{}", entry.id, chrono::Local::now().timestamp()),
        created_at: chrono::Local::now().to_rfc3339(),
        old: entry.old.clone(),
        new: entry.new.clone(),
        styles: vec![], // Parse from strings if needed
        includes: entry.includes.clone(),
        excludes: entry.excludes.clone(),
        matches: vec![], // We don't store the actual hunks in history
        renames: vec![],
        stats: Stats {
            files_scanned: 0,
            total_matches: 0,
            matches_by_variant: HashMap::new(),
            files_with_matches: entry.affected_files.len(),
        },
        version: "1.0.0".to_string(),
    };
    
    // Convert rename tuples to Rename structs
    for (from, to) in &entry.renames {
        let kind = if from.is_dir() {
            RenameKind::Dir
        } else {
            RenameKind::File
        };
        
        plan.renames.push(Rename {
            from: from.clone(),
            to: to.clone(),
            kind,
        });
    }
    
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_history_entry_to_plan() {
        let entry = crate::history::HistoryEntry {
            id: "test123".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
            styles: vec!["Snake".to_string()],
            includes: vec!["*.rs".to_string()],
            excludes: vec!["target/**".to_string()],
            affected_files: HashMap::new(),
            renames: vec![
                (PathBuf::from("old.txt"), PathBuf::from("new.txt")),
            ],
            backups_path: PathBuf::from("/tmp/backups"),
            revert_of: None,
            redo_of: None,
        };
        
        let plan = history_entry_to_plan(&entry).unwrap();
        
        assert_eq!(plan.old, "old_name");
        assert_eq!(plan.new, "new_name");
        assert_eq!(plan.includes, vec!["*.rs"]);
        assert_eq!(plan.excludes, vec!["target/**"]);
        assert_eq!(plan.renames.len(), 1);
        assert_eq!(plan.renames[0].from, PathBuf::from("old.txt"));
        assert_eq!(plan.renames[0].to, PathBuf::from("new.txt"));
    }
}