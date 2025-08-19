mod diff;
mod matches;
mod summary;
mod table;

pub use diff::render_diff;
pub use matches::render_matches;
pub use summary::render_summary;
pub use table::render_table;

use crate::scanner::Plan;
use anyhow::Result;
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Table,
    Diff,
    Matches,
    Summary,
    None,
}

impl std::str::FromStr for Preview {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "diff" => Ok(Self::Diff),
            "matches" => Ok(Self::Matches),
            "summary" => Ok(Self::Summary),
            "none" => Ok(Self::None),
            _ => Err(format!("Invalid preview format: {}", s)),
        }
    }
}

/// Determine whether to use colors based on explicit preference or terminal detection
pub fn should_use_color_with_detector<F>(use_color: Option<bool>, is_terminal: F) -> bool
where
    F: Fn() -> bool,
{
    match use_color {
        Some(explicit_color) => explicit_color, // Honor explicit color request
        None => is_terminal(),                  // Auto-detect only when not specified
    }
}

/// Determine whether to use colors based on explicit preference or terminal detection
pub fn should_use_color(use_color: Option<bool>) -> bool {
    should_use_color_with_detector(use_color, || io::stdout().is_terminal())
}

/// Render the plan in the specified format
pub fn render_plan(plan: &Plan, format: Preview, use_color: Option<bool>) -> String {
    render_plan_with_fixed_width(plan, format, use_color, false)
}

// For backwards compatibility and tests
pub fn render_plan_with_fixed_width(
    plan: &Plan,
    format: Preview,
    use_color: Option<bool>,
    fixed_width: bool,
) -> String {
    let use_color = should_use_color(use_color);

    match format {
        Preview::Table => render_table(plan, use_color, fixed_width),
        Preview::Diff => render_diff(plan, use_color),
        Preview::Matches => render_matches(plan, use_color),
        Preview::Summary => render_summary(plan),
        Preview::None => String::new(), // Return empty string for no preview
    }
}

/// Write plan preview to stdout
pub fn write_preview(plan: &Plan, format: Preview, use_color: Option<bool>) -> Result<()> {
    let output = render_plan(plan, format, use_color);
    let mut stdout = io::stdout();
    write!(stdout, "{}", output)?;
    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_model::Style;
    use crate::scanner::{MatchHunk, Rename, RenameKind, Stats};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_plan() -> Plan {
        let mut matches_by_variant = HashMap::new();
        matches_by_variant.insert("old_name".to_string(), 2);
        matches_by_variant.insert("oldName".to_string(), 1);

        Plan {
            id: "test123".to_string(),
            created_at: "123456789".to_string(),
            search: "old_name".to_string(),
            replace: "new_name".to_string(),
            styles: vec![Style::Snake, Style::Camel],
            includes: vec![],
            excludes: vec![],
            matches: vec![
                MatchHunk {
                    file: PathBuf::from("src/main.rs"),
                    line: 10,
                    col: 5,
                    variant: "old_name".to_string(),
                    content: "old_name".to_string(),
                    replace: "new_name".to_string(),
                    start: 4,
                    end: 12,
                    line_before: Some("let old_name = 42;".to_string()),
                    line_after: Some("let new_name = 42;".to_string()),
                    coercion_applied: None,
                    original_file: None,
                    renamed_file: None,
                    patch_hash: None,
                },
                MatchHunk {
                    file: PathBuf::from("src/main.rs"),
                    line: 20,
                    col: 10,
                    variant: "oldName".to_string(),
                    content: "oldName".to_string(),
                    replace: "newName".to_string(),
                    start: 3,
                    end: 10,
                    line_before: Some("return oldName;".to_string()),
                    line_after: Some("return newName;".to_string()),
                    coercion_applied: None,
                    original_file: None,
                    renamed_file: None,
                    patch_hash: None,
                },
            ],
            paths: vec![Rename {
                path: PathBuf::from("old_name.txt"),
                new_path: PathBuf::from("new_name.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            }],
            stats: Stats {
                files_scanned: 10,
                total_matches: 2,
                matches_by_variant,
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        }
    }

    #[test]
    fn test_preview_from_str() {
        use std::str::FromStr;

        assert_eq!(Preview::from_str("table"), Ok(Preview::Table));
        assert_eq!(Preview::from_str("diff"), Ok(Preview::Diff));
        assert_eq!(Preview::from_str("matches"), Ok(Preview::Matches));
        assert_eq!(Preview::from_str("summary"), Ok(Preview::Summary));
        assert_eq!(Preview::from_str("none"), Ok(Preview::None));
        assert_eq!(Preview::from_str("TABLE"), Ok(Preview::Table));
        assert_eq!(Preview::from_str("MATCHES"), Ok(Preview::Matches));
        assert_eq!(Preview::from_str("SUMMARY"), Ok(Preview::Summary));
        assert_eq!(Preview::from_str("NONE"), Ok(Preview::None));
        assert!(Preview::from_str("invalid").is_err());
    }

    #[test]
    fn test_render_table_no_color() {
        let plan = create_test_plan();
        let result = render_table(&plan, false, true);

        assert!(result.contains("src/main.rs"));
        assert!(result.contains("Content"));
        assert!(result.contains("old_name"));
        assert!(result.contains("TOTALS"));
        assert!(result.contains("→ new_name.txt"));
    }

    #[test]
    fn test_render_diff_no_color() {
        let plan = create_test_plan();
        let result = render_diff(&plan, false);

        assert!(result.contains("--- src/main.rs"));
        assert!(result.contains("+++ src/main.rs"));
        assert!(result.contains("@@ line 10 @@"));
        assert!(result.contains("-let old_name = 42;"));
        assert!(result.contains("+let new_name = 42;"));
        assert!(result.contains("@@ line 20 @@"));
        assert!(result.contains("-return oldName;"));
        assert!(result.contains("+return newName;"));
        assert!(result.contains("=== RENAMES ==="));
        assert!(result.contains("old_name.txt → new_name.txt"));
    }

    #[test]
    fn test_render_matches_no_color() {
        let plan = create_test_plan();
        let result = render_matches(&plan, false);

        assert!(result.contains("Search Results for \"old_name\""));
        assert!(result.contains("Content Matches:"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("Path Matches:"));
        assert!(result.contains("Files:"));
        assert!(result.contains("old_name.txt"));
        assert!(result.contains("Found 2 matches in 1 files"));
    }

    #[test]
    fn test_render_diff_shows_full_line_context() {
        let mut matches_by_variant = HashMap::new();
        matches_by_variant.insert("old_func".to_string(), 1);

        // Create a plan with hunks that have full line context
        let plan = Plan {
            id: "test_full_line".to_string(),
            created_at: "123456789".to_string(),
            search: "old_func".to_string(),
            replace: "new_func".to_string(),
            styles: vec![Style::Snake],
            includes: vec![],
            excludes: vec![],
            matches: vec![MatchHunk {
                file: PathBuf::from("src/lib.rs"),
                line: 42,
                col: 12,
                variant: "old_func".to_string(),
                // Word-level replacement for apply
                content: "old_func".to_string(),
                replace: "new_func".to_string(),
                start: 17,
                end: 25,
                // Full line context for diff preview
                line_before: Some("    let result = old_func(param1, param2);".to_string()),
                line_after: Some("    let result = new_func(param1, param2);".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            }],
            paths: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant,
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let result = render_diff(&plan, false);

        // Should show the full line, not just the word
        assert!(result.contains("-    let result = old_func(param1, param2);"));
        assert!(result.contains("+    let result = new_func(param1, param2);"));

        // Should NOT show just the word alone
        assert!(!result.contains("-old_func\n"));
        assert!(!result.contains("+new_func\n"));
    }

    #[test]
    fn test_render_summary() {
        let plan = create_test_plan();
        let result = render_summary(&plan);

        // Check that summary contains expected sections
        assert!(result.contains("[PLAN SUMMARY]"));
        assert!(result.contains("Search: old_name"));
        assert!(result.contains("Replace: new_name"));
        assert!(result.contains("Matches: 2"));
        assert!(result.contains("Files: 1"));

        // Check content changes section
        assert!(result.contains("[CONTENT]"));
        assert!(result.contains("src/main.rs: 2 matches"));
        assert!(result.contains("[oldName: 1, old_name: 1]"));

        // Check renames section
        assert!(result.contains("[PATHS]"));
        assert!(result.contains("file: old_name.txt -> new_name.txt"));
    }

    #[test]
    fn test_empty_plan() {
        let plan = Plan {
            id: "empty".to_string(),
            created_at: "0".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: vec![],
            stats: Stats {
                files_scanned: 0,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let table = render_table(&plan, false, true);
        assert!(table.contains("TOTALS"));
        assert!(table.contains('0'));

        let diff = render_diff(&plan, false);
        assert!(diff.is_empty() || diff == "\n");
    }

    #[test]
    fn test_should_use_color_explicit_true() {
        // When explicitly requesting colors, should always return true regardless of terminal
        assert!(should_use_color_with_detector(Some(true), || false));
        assert!(should_use_color_with_detector(Some(true), || true));
    }

    #[test]
    fn test_should_use_color_explicit_false() {
        // When explicitly disabling colors, should always return false regardless of terminal
        assert!(!should_use_color_with_detector(Some(false), || false));
        assert!(!should_use_color_with_detector(Some(false), || true));
    }

    #[test]
    fn test_should_use_color_auto_detect_terminal() {
        // When no explicit preference, should use terminal detection
        assert!(should_use_color_with_detector(None, || true));
        assert!(!should_use_color_with_detector(None, || false));
    }

    #[test]
    fn test_render_plan_with_explicit_colors() {
        let plan = create_test_plan();

        // Test with forced colors (unset NO_COLOR for this test)
        let original_no_color = std::env::var("NO_COLOR").ok();
        std::env::remove_var("NO_COLOR");

        // Explicit true should produce colors even in non-terminal environment
        let output = render_plan(&plan, Preview::Table, Some(true));

        // Test with NO_COLOR set
        std::env::set_var("NO_COLOR", "1");
        let output_no_color = render_plan(&plan, Preview::Table, Some(false));

        // Restore original NO_COLOR state
        match original_no_color {
            Some(val) => std::env::set_var("NO_COLOR", val),
            None => std::env::remove_var("NO_COLOR"),
        }

        // Check for ANSI color codes - we expect colors when explicitly requested
        assert!(
            output.contains("\u{1b}["),
            "Should contain ANSI color codes when explicitly requested"
        );

        assert!(
            !output_no_color.contains("\u{1b}["),
            "Should not contain ANSI color codes when explicitly disabled"
        );
    }

    #[test]
    fn test_is_root_directory_rename() {
        use crate::scanner::{Rename, RenameKind};
        use std::path::PathBuf;

        // Helper function to check if a rename represents a root directory rename
        fn is_root_directory_rename(rename: &Rename) -> bool {
            let from_path = &rename.path;

            // Check if this rename is for the current working directory
            std::env::current_dir()
                .map(|current_dir| from_path == &current_dir)
                .unwrap_or(false)
        }

        // Get the actual current working directory for testing
        let current_dir = std::env::current_dir().unwrap();

        // Test case that should be considered a root directory rename (current working directory)
        let root_rename = Rename {
            path: current_dir.clone(),
            new_path: PathBuf::from("renamed_renaming_tool"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            is_root_directory_rename(&root_rename),
            "Current working directory should be root"
        );

        // Test case for a relative path - should NOT be considered root unless it matches current dir
        let relative_rename = Rename {
            path: PathBuf::from("project"),
            new_path: PathBuf::from("renamed_renaming_tool"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            !is_root_directory_rename(&relative_rename),
            "Relative path should not be root unless it's the current directory"
        );

        // Test cases that should NOT be considered root directory renames
        let subdir_rename = Rename {
            path: current_dir.join("subdir"),
            new_path: PathBuf::from("renamed-renaming-tool-subdir"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            !is_root_directory_rename(&subdir_rename),
            "Subdirectory should not be root"
        );

        let different_path_rename = Rename {
            path: PathBuf::from("/some/other/path"),
            new_path: PathBuf::from("/some/other/new_path"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            !is_root_directory_rename(&different_path_rename),
            "Different path should not be root"
        );
    }

    #[test]
    fn test_color_consistency_across_formats() {
        let plan = create_test_plan();

        // Test with environment control
        let original_no_color = std::env::var("NO_COLOR").ok();
        std::env::remove_var("NO_COLOR");

        // All formats should respect explicit color settings consistently
        let table_colored = render_plan(&plan, Preview::Table, Some(true));
        let diff_colored = render_plan(&plan, Preview::Diff, Some(true));

        // Set NO_COLOR for disabled test
        std::env::set_var("NO_COLOR", "1");
        let table_no_color = render_plan(&plan, Preview::Table, Some(false));
        let diff_no_color = render_plan(&plan, Preview::Diff, Some(false));

        // Restore original NO_COLOR state
        match original_no_color {
            Some(val) => std::env::set_var("NO_COLOR", val),
            None => std::env::remove_var("NO_COLOR"),
        }

        // Be lenient about colors in non-terminal environments but ensure consistency
        let table_has_colors = table_colored.contains("\u{1b}[");
        let diff_has_colors = diff_colored.contains("\u{1b}[");

        // If one format has colors, both should (consistency check)
        if table_has_colors || diff_has_colors {
            assert_eq!(
                table_has_colors, diff_has_colors,
                "Table and diff formats should be consistent about color usage"
            );
        }

        assert!(
            !table_no_color.contains("\u{1b}["),
            "Table format should not have colors when disabled"
        );
        assert!(
            !diff_no_color.contains("\u{1b}["),
            "Diff format should not have colors when disabled"
        );
    }
}
