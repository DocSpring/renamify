use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, PartialEq)]
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
            output.push_str(&format!("Search results for '{}'\n", self.search));
        } else {
            // Plan mode
            output.push_str(&format!(
                "Renamify plan: {} -> {}\n",
                self.search, self.replace
            ));
        }

        output.push_str(&format!(
            "Edits: {} files, {} replacements\n",
            self.files_with_matches, self.total_matches
        ));

        if self.renames > 0 {
            output.push_str(&format!("Renames: {} items\n", self.renames));
        }

        if !self.dry_run {
            output.push_str(&format!("Plan ID: {}\n", self.plan_id));
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

        output.push_str(&format!(
            "✓ Applied {} replacements across {} files\n",
            self.replacements, self.files_changed
        ));

        if self.renames > 0 {
            output.push_str(&format!("✓ Renamed {} items\n", self.renames));
        }

        if self.committed {
            output.push_str("✓ Changes committed to git\n");
        }

        output.push_str(&format!("Undo with: renamify undo {}", self.plan_id));

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
            output.push_str(&format!("✓ Restored {} files\n", self.files_restored));
        }

        if self.renames_reverted > 0 {
            output.push_str(&format!("✓ Reverted {} renames\n", self.renames_reverted));
        }

        output.push_str(&format!("Redo with: renamify redo {}", self.history_id));

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

        output.push_str(&format!(
            "✓ Applied {} replacements across {} files\n",
            self.replacements, self.files_changed
        ));

        if self.renames > 0 {
            output.push_str(&format!("✓ Renamed {} items\n", self.renames));
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
            output.push_str(&format!(
                "Pending plan: {} ({} -> {})\n",
                plan.id, plan.search, plan.replace
            ));
            output.push_str(&format!("Created: {}\n", plan.created_at));
        } else {
            output.push_str("No pending plan\n");
        }

        output.push_str(&format!("History entries: {}\n", self.history_count));

        if let Some(ref last_op) = self.last_operation {
            output.push_str(&format!("Last operation: {}\n", last_op));
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
            output.push_str(&format!(
                "{} [{}] {} -> {} ({} files, {} replacements",
                entry.id,
                entry.operation,
                entry.search,
                entry.replace,
                entry.files_changed,
                entry.replacements
            ));

            if entry.renames > 0 {
                output.push_str(&format!(", {} renames", entry.renames));
            }

            if entry.reverted {
                output.push_str(" [REVERTED]");
            }

            output.push_str(&format!(") {}\n", entry.timestamp));
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
            output.push_str(&format!("✓ Renamed {} items\n", self.renames));
        }

        if self.committed {
            output.push_str("✓ Changes committed to git\n");
        }

        output.push_str(&format!("Undo with: renamify undo {}", self.plan_id));

        output
    }
}
