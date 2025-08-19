use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
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

        write!(output, "Undo with: renamify undo {}", self.plan_id).unwrap();

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
        serde_json::to_string(&self.entries).unwrap_or_default()
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

        write!(output, "Undo with: renamify undo {}", self.plan_id).unwrap();

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
