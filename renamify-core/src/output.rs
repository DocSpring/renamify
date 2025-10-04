use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write;

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Summary,
    Json,
}

/// Result of a plan operation
#[derive(Debug, Serialize, Deserialize)]
pub struct PlanResult {
    pub plan_id: String,
    pub search: String,
    pub replace: String,
    pub files_with_matches: usize,
    pub total_matches: usize,
    pub renames: usize,
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<crate::scanner::Plan>,
}

/// Result of an apply operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyResult {
    pub plan_id: String,
    pub files_changed: usize,
    pub replacements: usize,
    pub renames: usize,
    pub committed: bool,
}

/// Result of an undo operation
#[derive(Debug, Serialize, Deserialize)]
pub struct UndoResult {
    pub history_id: String,
    pub files_restored: usize,
    pub renames_reverted: usize,
}

/// Result of a redo operation
#[derive(Debug, Serialize, Deserialize)]
pub struct RedoResult {
    pub history_id: String,
    pub files_changed: usize,
    pub replacements: usize,
    pub renames: usize,
}

/// Result of a status operation
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResult {
    pub pending_plan: Option<PendingPlan>,
    pub history_count: usize,
    pub last_operation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingPlan {
    pub id: String,
    pub search: String,
    pub replace: String,
    pub created_at: String,
}

/// Result of a history operation
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryResult {
    pub entries: Vec<HistoryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub operation: String,
    pub timestamp: String,
    pub search: String,
    pub replace: String,
    pub files_changed: usize,
    pub replacements: usize,
    pub renames: usize,
    pub reverted: bool,
}

/// Result of a rename operation (direct apply without plan)
#[derive(Debug, Serialize, Deserialize)]
pub struct RenameResult {
    pub plan_id: String,
    pub search: String,
    pub replace: String,
    pub files_changed: usize,
    pub replacements: usize,
    pub renames: usize,
    pub committed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<crate::scanner::Plan>,
}

/// Result of a version command
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResult {
    pub name: String,
    pub version: String,
}

/// Trait for formatting output in different formats
pub trait OutputFormatter {
    fn format(&self, format: OutputFormat) -> String;
    fn format_json(&self) -> String;
    fn format_summary(&self) -> String;
}

impl OutputFormatter for PlanResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&json!({
            "success": true,
            "operation": if self.replace.is_empty() { "search" } else { "plan" },
            "plan_id": self.plan_id,
            "search": self.search,
            "replace": self.replace,
            "dry_run": self.dry_run,
            "summary": {
                "files_with_matches": self.files_with_matches,
                "total_matches": self.total_matches,
                "renames": self.renames,
            },
            "plan": self.plan,
        }))
        .unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        let mut output = String::new();

        if self.replace.is_empty() {
            // Search mode
            writeln!(output, "Search results for '{}'", self.search).unwrap();
        } else {
            // Plan mode
            writeln!(output, "Renamify plan: {} -> {}", self.search, self.replace).unwrap();
        }

        writeln!(
            output,
            "Edits: {} files, {} replacements",
            self.files_with_matches, self.total_matches
        )
        .unwrap();

        if self.renames > 0 {
            writeln!(output, "Renames: {} items", self.renames).unwrap();
        }

        if !self.dry_run {
            writeln!(output, "Plan ID: {}", self.plan_id).unwrap();
        }

        output
    }
}

impl OutputFormatter for ApplyResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&json!({
            "success": true,
            "operation": "apply",
            "plan_id": self.plan_id,
            "summary": {
                "files_changed": self.files_changed,
                "replacements": self.replacements,
                "renames": self.renames,
            },
            "committed": self.committed,
        }))
        .unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        let mut output = format!("Changes applied successfully. Plan ID: {}\n", self.plan_id);

        writeln!(
            output,
            "✓ Applied {} replacements across {} files",
            self.replacements, self.files_changed
        )
        .unwrap();

        if self.renames > 0 {
            writeln!(output, "✓ Renamed {} items", self.renames).unwrap();
        }

        if self.committed {
            output.push_str("✓ Changes committed to git\n");
        }

        writeln!(output, "Undo with: renamify undo {}", self.plan_id).unwrap();

        output
    }
}

impl OutputFormatter for UndoResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&json!({
            "success": true,
            "operation": "undo",
            "history_id": self.history_id,
            "summary": {
                "files_restored": self.files_restored,
                "renames_reverted": self.renames_reverted,
            }
        }))
        .unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        let mut output = format!("Successfully undid operation {}\n", self.history_id);

        if self.files_restored > 0 {
            writeln!(output, "✓ Restored {} files", self.files_restored).unwrap();
        }

        if self.renames_reverted > 0 {
            writeln!(output, "✓ Reverted {} renames", self.renames_reverted).unwrap();
        }

        write!(output, "Redo with: renamify redo {}", self.history_id).unwrap();

        output
    }
}

impl OutputFormatter for RedoResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&json!({
            "success": true,
            "operation": "redo",
            "history_id": self.history_id,
            "summary": {
                "files_changed": self.files_changed,
                "replacements": self.replacements,
                "renames": self.renames,
            }
        }))
        .unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        let mut output = format!("Successfully redid operation {}\n", self.history_id);

        writeln!(
            output,
            "✓ Applied {} replacements across {} files",
            self.replacements, self.files_changed
        )
        .unwrap();

        if self.renames > 0 {
            writeln!(output, "✓ Renamed {} items", self.renames).unwrap();
        }

        output
    }
}

impl OutputFormatter for StatusResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        let mut output = String::new();

        if let Some(ref plan) = self.pending_plan {
            writeln!(
                output,
                "Pending plan: {} ({} -> {})",
                plan.id, plan.search, plan.replace
            )
            .unwrap();
            writeln!(output, "Created: {}", plan.created_at).unwrap();
        } else {
            output.push_str("No pending plan\n");
        }

        writeln!(output, "History entries: {}", self.history_count).unwrap();

        if let Some(ref last_op) = self.last_operation {
            writeln!(output, "Last operation: {}", last_op).unwrap();
        }

        output
    }
}

impl OutputFormatter for HistoryResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&json!({
            "entries": self.entries
        }))
        .unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        if self.entries.is_empty() {
            return "No history entries found".to_string();
        }

        let mut output = String::new();
        for entry in &self.entries {
            write!(
                output,
                "{} [{}] {} -> {} ({} files, {} replacements",
                entry.id,
                entry.operation,
                entry.search,
                entry.replace,
                entry.files_changed,
                entry.replacements
            )
            .unwrap();

            if entry.renames > 0 {
                write!(output, ", {} renames", entry.renames).unwrap();
            }

            if entry.reverted {
                output.push_str(" [REVERTED]");
            }

            writeln!(output, ") {}", entry.timestamp).unwrap();
        }

        output
    }
}

impl OutputFormatter for RenameResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(&json!({
            "success": true,
            "operation": "rename",
            "plan_id": self.plan_id,
            "search": self.search,
            "replace": self.replace,
            "summary": {
                "files_changed": self.files_changed,
                "replacements": self.replacements,
                "renames": self.renames,
            },
            "committed": self.committed,
            "plan": self.plan,
        }))
        .unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        let mut output = format!(
            "✓ Applied {} replacements across {} files\n",
            self.replacements, self.files_changed
        );

        if self.renames > 0 {
            writeln!(output, "✓ Renamed {} items", self.renames).unwrap();
        }

        if self.committed {
            output.push_str("✓ Changes committed to git\n");
        }

        writeln!(output, "Undo with: renamify undo {}", self.plan_id).unwrap();

        output
    }
}

impl OutputFormatter for VersionResult {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Summary => self.format_summary(),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn format_summary(&self) -> String {
        format!("{} {}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_result_json_format() {
        let result = PlanResult {
            plan_id: "test123".to_string(),
            search: "old_name".to_string(),
            replace: "new_name".to_string(),
            files_with_matches: 5,
            total_matches: 15,
            renames: 3,
            dry_run: false,
            plan: None,
        };

        let json = result.format_json();
        assert!(json.contains("\"operation\":\"plan\""));
        assert!(json.contains("\"plan_id\":\"test123\""));
        assert!(json.contains("\"search\":\"old_name\""));
        assert!(json.contains("\"replace\":\"new_name\""));
        assert!(json.contains("\"files_with_matches\":5"));
        assert!(json.contains("\"total_matches\":15"));
        assert!(json.contains("\"renames\":3"));
        assert!(json.contains("\"dry_run\":false"));
    }

    #[test]
    fn test_plan_result_json_search_mode() {
        let result = PlanResult {
            plan_id: "search123".to_string(),
            search: "find_this".to_string(),
            replace: String::new(), // Empty replace = search mode
            files_with_matches: 2,
            total_matches: 8,
            renames: 0,
            dry_run: true,
            plan: None,
        };

        let json = result.format_json();
        assert!(json.contains("\"operation\":\"search\""));
        assert!(json.contains("\"dry_run\":true"));
    }

    #[test]
    fn test_plan_result_summary_format() {
        let result = PlanResult {
            plan_id: "test123".to_string(),
            search: "old_name".to_string(),
            replace: "new_name".to_string(),
            files_with_matches: 5,
            total_matches: 15,
            renames: 3,
            dry_run: false,
            plan: None,
        };

        let summary = result.format_summary();
        assert!(summary.contains("old_name"));
        assert!(summary.contains("new_name"));
        assert!(summary.contains('5'));
        assert!(summary.contains("15"));
        assert!(summary.contains('3'));
    }

    #[test]
    fn test_plan_result_summary_search_mode() {
        let result = PlanResult {
            plan_id: "search123".to_string(),
            search: "find_this".to_string(),
            replace: String::new(),
            files_with_matches: 2,
            total_matches: 8,
            renames: 0,
            dry_run: true,
            plan: None,
        };

        let summary = result.format_summary();
        assert!(summary.contains("Search results for 'find_this'"));
        assert!(summary.contains('2'));
        assert!(summary.contains('8'));
    }

    #[test]
    fn test_apply_result_json_format() {
        let result = ApplyResult {
            plan_id: "apply123".to_string(),
            files_changed: 10,
            replacements: 25,
            renames: 5,
            committed: true,
        };

        let json = result.format_json();
        assert!(json.contains("\"plan_id\":\"apply123\""));
        assert!(json.contains("\"files_changed\":10"));
        assert!(json.contains("\"replacements\":25"));
        assert!(json.contains("\"renames\":5"));
        assert!(json.contains("\"committed\":true"));
    }

    #[test]
    fn test_apply_result_summary_format() {
        let result = ApplyResult {
            plan_id: "apply123".to_string(),
            files_changed: 10,
            replacements: 25,
            renames: 5,
            committed: false,
        };

        let summary = result.format_summary();
        assert!(summary.contains("Applied"));
        assert!(summary.contains("10"));
        assert!(summary.contains("25"));
        assert!(summary.contains('5'));
        assert!(!summary.contains("committed")); // Should not mention git commit when false
    }

    #[test]
    fn test_apply_result_summary_with_commit() {
        let result = ApplyResult {
            plan_id: "apply123".to_string(),
            files_changed: 10,
            replacements: 25,
            renames: 5,
            committed: true,
        };

        let summary = result.format_summary();
        assert!(summary.contains("committed") || summary.contains("git"));
    }

    #[test]
    fn test_undo_result_json_format() {
        let result = UndoResult {
            history_id: "undo456".to_string(),
            files_restored: 8,
            renames_reverted: 3,
        };

        let json = result.format_json();
        assert!(json.contains("\"history_id\":\"undo456\""));
        assert!(json.contains("\"files_restored\":8"));
        assert!(json.contains("\"renames_reverted\":3"));
    }

    #[test]
    fn test_undo_result_summary_format() {
        let result = UndoResult {
            history_id: "undo456".to_string(),
            files_restored: 8,
            renames_reverted: 3,
        };

        let summary = result.format_summary();
        assert!(summary.contains("Reverted"));
        assert!(summary.contains('8'));
        assert!(summary.contains('3'));
    }

    #[test]
    fn test_redo_result_json_format() {
        let result = RedoResult {
            history_id: "redo789".to_string(),
            files_changed: 12,
            replacements: 30,
            renames: 4,
        };

        let json = result.format_json();
        assert!(json.contains("\"history_id\":\"redo789\""));
        assert!(json.contains("\"files_changed\":12"));
        assert!(json.contains("\"replacements\":30"));
        assert!(json.contains("\"renames\":4"));
    }

    #[test]
    fn test_redo_result_summary_format() {
        let result = RedoResult {
            history_id: "redo789".to_string(),
            files_changed: 12,
            replacements: 30,
            renames: 4,
        };

        let summary = result.format_summary();
        assert!(summary.contains("Successfully redid"));
        assert!(summary.contains("12"));
        assert!(summary.contains("30"));
        assert!(summary.contains('4'));
    }

    #[test]
    fn test_status_result_json_format() {
        let pending = Some(PendingPlan {
            id: "pending123".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let result = StatusResult {
            pending_plan: pending,
            history_count: 5,
            last_operation: Some("apply".to_string()),
        };

        let json = result.format_json();
        assert!(json.contains("\"pending_plan\""));
        assert!(json.contains("\"id\":\"pending123\""));
        assert!(json.contains("\"history_count\":5"));
        assert!(json.contains("\"last_operation\":\"apply\""));
    }

    #[test]
    fn test_status_result_json_no_pending() {
        let result = StatusResult {
            pending_plan: None,
            history_count: 0,
            last_operation: None,
        };

        let json = result.format_json();
        assert!(json.contains("\"pending_plan\":null"));
        assert!(json.contains("\"history_count\":0"));
        assert!(json.contains("\"last_operation\":null"));
    }

    #[test]
    fn test_status_result_summary_with_pending() {
        let pending = Some(PendingPlan {
            id: "pending123".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let result = StatusResult {
            pending_plan: pending,
            history_count: 5,
            last_operation: Some("apply".to_string()),
        };

        let summary = result.format_summary();
        assert!(summary.contains("pending"));
        assert!(summary.contains("old"));
        assert!(summary.contains("new"));
        assert!(summary.contains('5'));
    }

    #[test]
    fn test_status_result_summary_no_pending() {
        let result = StatusResult {
            pending_plan: None,
            history_count: 3,
            last_operation: Some("undo".to_string()),
        };

        let summary = result.format_summary();
        assert!(summary.contains("No pending"));
        assert!(summary.contains('3'));
    }

    #[test]
    fn test_history_result_json_format() {
        let items = vec![
            HistoryItem {
                id: "hist1".to_string(),
                operation: "apply".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                search: "old".to_string(),
                replace: "new".to_string(),
                files_changed: 5,
                replacements: 10,
                renames: 2,
                reverted: false,
            },
            HistoryItem {
                id: "hist2".to_string(),
                operation: "undo".to_string(),
                timestamp: "2024-01-01T01:00:00Z".to_string(),
                search: "old".to_string(),
                replace: "new".to_string(),
                files_changed: 5,
                replacements: 10,
                renames: 2,
                reverted: true,
            },
        ];

        let result = HistoryResult { entries: items };
        let json = result.format_json();
        assert!(json.contains("\"entries\""));
        assert!(json.contains("\"id\":\"hist1\""));
        assert!(json.contains("\"operation\":\"apply\""));
        assert!(json.contains("\"reverted\":false"));
        assert!(json.contains("\"reverted\":true"));
    }

    #[test]
    fn test_history_result_summary_format() {
        let items = vec![HistoryItem {
            id: "hist1".to_string(),
            operation: "apply".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            files_changed: 5,
            replacements: 10,
            renames: 2,
            reverted: false,
        }];

        let result = HistoryResult { entries: items };
        let summary = result.format_summary();
        assert!(summary.contains("hist1"));
        assert!(summary.contains("apply"));
        assert!(summary.contains("old"));
        assert!(summary.contains("new"));
    }

    #[test]
    fn test_history_result_summary_empty() {
        let result = HistoryResult { entries: vec![] };
        let summary = result.format_summary();
        assert!(summary.contains("No history") || summary.contains("empty"));
    }

    #[test]
    fn test_rename_result_json_format() {
        let result = RenameResult {
            plan_id: "rename123".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            files_changed: 7,
            replacements: 14,
            renames: 3,
            committed: true,
            plan: None,
        };

        let json = result.format_json();
        assert!(json.contains("\"operation\":\"rename\""));
        assert!(json.contains("\"plan_id\":\"rename123\""));
        assert!(json.contains("\"files_changed\":7"));
        assert!(json.contains("\"committed\":true"));
    }

    #[test]
    fn test_rename_result_summary_format() {
        let result = RenameResult {
            plan_id: "rename123".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            files_changed: 7,
            replacements: 14,
            renames: 3,
            committed: false,
            plan: None,
        };

        let summary = result.format_summary();
        assert!(summary.contains("14"));
        assert!(summary.contains('7'));
        assert!(summary.contains('3'));
    }

    #[test]
    fn test_version_result_json_format() {
        let result = VersionResult {
            name: "renamify".to_string(),
            version: "1.0.0".to_string(),
        };

        let json = result.format_json();
        assert!(json.contains("\"name\":\"renamify\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }

    #[test]
    fn test_version_result_summary_format() {
        let result = VersionResult {
            name: "renamify".to_string(),
            version: "1.0.0".to_string(),
        };

        let summary = result.format_summary();
        assert_eq!(summary, "renamify 1.0.0");
    }

    #[test]
    fn test_output_format_trait() {
        let result = VersionResult {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
        };

        // Test that format() calls the right method
        assert_eq!(result.format(OutputFormat::Summary), "test 0.1.0");
        assert!(result
            .format(OutputFormat::Json)
            .contains("\"name\":\"test\""));
    }
}
