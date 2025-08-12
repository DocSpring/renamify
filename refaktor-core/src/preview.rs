use crate::scanner::{MatchHunk, Plan, Rename, RenameKind};
use anyhow::Result;
use comfy_table::{Cell, Color, ColumnConstraint, ContentArrangement, Table, Width};
use nu_ansi_term::{Color as AnsiColor, Style};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

/// Check if a rename represents a root directory rename
/// A root directory rename is when the from path equals the current working directory
fn is_root_directory_rename(rename: &Rename) -> bool {
    let from_path = &rename.from;

    // Check if this rename is for the current working directory
    std::env::current_dir()
        .map(|current_dir| from_path == &current_dir)
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewFormat {
    Table,
    Diff,
    Json,
}

impl PreviewFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(Self::Table),
            "diff" => Some(Self::Diff),
            "json" => Some(Self::Json),
            _ => None,
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
pub fn render_plan(plan: &Plan, format: PreviewFormat, use_color: Option<bool>) -> Result<String> {
    render_plan_with_fixed_width(plan, format, use_color, false)
}

// Mainly for tests without a tty
pub fn render_plan_with_fixed_width(
    plan: &Plan,
    format: PreviewFormat,
    use_color: Option<bool>,
    fixed_width: bool,
) -> Result<String> {
    let use_color = should_use_color(use_color);

    match format {
        PreviewFormat::Table => render_table_with_fixed_width(plan, use_color, fixed_width),
        PreviewFormat::Diff => render_diff(plan, use_color),
        PreviewFormat::Json => render_json(plan),
    }
}

/// Render plan as a table
fn render_table(plan: &Plan, use_color: bool) -> Result<String> {
    render_table_with_fixed_width(plan, use_color, false)
}

/// Render plan as a table with explicit width control
fn render_table_with_fixed_width(
    plan: &Plan,
    use_color: bool,
    fixed_width: bool,
) -> Result<String> {
    let mut table = Table::new();

    // Set content arrangement and constraints based on width parameter
    if fixed_width {
        table.set_content_arrangement(ContentArrangement::Disabled);
        // Set absolute column widths for consistent layout
        table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(75)), // File
            ColumnConstraint::Absolute(Width::Fixed(30)), // Kind
            ColumnConstraint::Absolute(Width::Fixed(15)), // Matches
            ColumnConstraint::Absolute(Width::Fixed(75)), // Variants
        ]);
    } else {
        table.set_content_arrangement(ContentArrangement::Dynamic);
    }

    // Force styling even in non-TTY environments when colors are explicitly requested
    if use_color {
        table.enforce_styling();
    }

    // Set headers
    if use_color {
        table.set_header(vec![
            Cell::new("File").fg(Color::Cyan),
            Cell::new("Kind").fg(Color::Cyan),
            Cell::new("Matches").fg(Color::Cyan),
            Cell::new("Variants").fg(Color::Cyan),
        ]);
    } else {
        table.set_header(vec!["File", "Kind", "Matches", "Variants"]);
    }

    // Group matches by file
    let mut file_stats: HashMap<&Path, (usize, Vec<&str>)> = HashMap::new();
    for hunk in &plan.matches {
        let entry = file_stats.entry(&hunk.file).or_insert((0, Vec::new()));
        entry.0 += 1;
        if !entry.1.contains(&hunk.variant.as_str()) {
            entry.1.push(&hunk.variant);
        }
    }

    // Sort files for deterministic output
    let mut sorted_files: Vec<_> = file_stats.keys().copied().collect();
    sorted_files.sort();

    // Add content rows
    for file in sorted_files {
        let (count, variants) = &file_stats[&file];
        // Make path relative to current directory for cleaner display
        let file_str = match std::env::current_dir()
            .ok()
            .and_then(|cwd| file.strip_prefix(cwd).ok())
        {
            Some(relative_path) => relative_path.display().to_string(),
            None => file.display().to_string(),
        };
        let variants_str = variants.join(", ");

        if use_color {
            table.add_row(vec![
                Cell::new(&file_str),
                Cell::new("Content").fg(Color::Green),
                Cell::new(count.to_string()).fg(Color::Yellow),
                Cell::new(&variants_str),
            ]);
        } else {
            table.add_row(vec![
                &file_str,
                "Content",
                &count.to_string(),
                &variants_str,
            ]);
        }
    }

    // Add rename rows (root directory renames should not be in plans unless explicitly requested)
    for rename in &plan.renames {
        // Make paths relative to current directory for cleaner display
        let from_str = match std::env::current_dir()
            .ok()
            .and_then(|cwd| rename.from.strip_prefix(cwd).ok())
        {
            Some(relative_path) => relative_path.display().to_string(),
            None => rename.from.display().to_string(),
        };
        let to_str = match std::env::current_dir()
            .ok()
            .and_then(|cwd| rename.to.strip_prefix(cwd).ok())
        {
            Some(relative_path) => relative_path.display().to_string(),
            None => rename.to.display().to_string(),
        };
        let kind_str = match rename.kind {
            RenameKind::File => "Rename (File)",
            RenameKind::Dir => "Rename (Dir)",
        };

        if use_color {
            table.add_row(vec![
                Cell::new(&from_str),
                Cell::new(kind_str).fg(Color::Blue),
                Cell::new(""),
                Cell::new(format!("→ {}", to_str)).fg(Color::Magenta),
            ]);
        } else {
            table.add_row(vec![&from_str, kind_str, "", &format!("→ {}", to_str)]);
        }
    }

    // Add footer with totals
    let total_matches = plan.stats.total_matches;
    let total_files = plan.stats.files_with_matches;
    let total_renames = plan.renames.len();

    if use_color {
        table.add_row(vec![
            Cell::new("─────────").fg(Color::DarkGrey),
            Cell::new("─────────").fg(Color::DarkGrey),
            Cell::new("─────────").fg(Color::DarkGrey),
            Cell::new("─────────").fg(Color::DarkGrey),
        ]);
        table.add_row(vec![
            Cell::new("TOTALS").fg(Color::Cyan),
            Cell::new(format!("{} files, {} renames", total_files, total_renames)).fg(Color::White),
            Cell::new(total_matches.to_string()).fg(Color::Yellow),
            Cell::new(format!("{} variants", plan.stats.matches_by_variant.len())).fg(Color::White),
        ]);
    } else {
        table.add_row(vec!["─────────", "─────────", "─────────", "─────────"]);
        table.add_row(vec![
            "TOTALS",
            &format!("{} files, {} renames", total_files, total_renames),
            &total_matches.to_string(),
            &format!("{} variants", plan.stats.matches_by_variant.len()),
        ]);
    }

    Ok(table.to_string())
}

/// Render plan as unified diffs
fn render_diff(plan: &Plan, use_color: bool) -> Result<String> {
    let mut output = String::new();

    // Group hunks by file
    let mut file_hunks: HashMap<&Path, Vec<&MatchHunk>> = HashMap::new();
    for hunk in &plan.matches {
        file_hunks.entry(&hunk.file).or_default().push(hunk);
    }

    // Sort files for deterministic output
    let mut sorted_files: Vec<_> = file_hunks.keys().copied().collect();
    sorted_files.sort();

    // Generate diffs for each file
    for file in sorted_files {
        let hunks = &file_hunks[&file];
        if use_color {
            output.push_str(&format!(
                "{}",
                AnsiColor::Cyan.bold().paint(format!(
                    "--- {}\n+++ {}\n",
                    file.display(),
                    file.display()
                ))
            ));
        } else {
            output.push_str(&format!("--- {}\n+++ {}\n", file.display(), file.display()));
        }

        // Group hunks by line number to merge multiple changes on the same line
        let mut line_hunks: HashMap<u64, Vec<&MatchHunk>> = HashMap::new();
        for hunk in hunks {
            line_hunks.entry(hunk.line).or_default().push(hunk);
        }

        // Sort lines for deterministic output
        let mut sorted_lines: Vec<_> = line_hunks.keys().copied().collect();
        sorted_lines.sort_unstable();

        // Create a unified diff from all hunks in this file
        for line_num in sorted_lines {
            let line_hunk_group = &line_hunks[&line_num];

            // For multiple hunks on the same line, we need to show the cumulative effect
            // Start with the original line and apply all replacements to get the final result
            let first_hunk = line_hunk_group[0];

            // Get the original line - use line_before if available, otherwise construct from hunk
            let before_text = if let Some(ref line_before) = first_hunk.line_before {
                line_before.clone()
            } else {
                // Fallback: just use the word itself if no line context
                first_hunk.before.clone()
            };

            // For the after text, if we have multiple hunks we need to apply all changes
            let after_text = if line_hunk_group.len() == 1 {
                // Single hunk - use line_after if available
                if let Some(ref line_after) = first_hunk.line_after {
                    line_after.clone()
                } else {
                    // Fallback: just the replacement word
                    first_hunk.after.clone()
                }
            } else {
                // Multiple hunks on same line - apply all replacements
                let mut after_line = before_text.clone();

                // Sort hunks by column position (reverse order to avoid position shifts)
                let mut sorted_hunks = line_hunk_group.clone();
                sorted_hunks.sort_by(|a, b| b.col.cmp(&a.col));

                // Apply replacements from right to left to maintain positions
                for hunk in sorted_hunks {
                    let col = hunk.col as usize;
                    if col < after_line.len() && after_line[col..].starts_with(&hunk.before) {
                        let end = col + hunk.before.len();
                        after_line.replace_range(col..end, &hunk.after);
                    }
                }

                after_line
            };

            let diff = TextDiff::from_lines(&before_text, &after_text);

            if use_color {
                output.push_str(&format!(
                    "{}",
                    AnsiColor::Blue.paint(format!("@@ line {} @@\n", line_num))
                ));
            } else {
                output.push_str(&format!("@@ line {} @@\n", line_num));
            }

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };

                let line = format!("{}{}", sign, change);

                if use_color {
                    let styled_line = match change.tag() {
                        ChangeTag::Delete => AnsiColor::Red.paint(line).to_string(),
                        ChangeTag::Insert => AnsiColor::Green.paint(line).to_string(),
                        ChangeTag::Equal => line,
                    };
                    output.push_str(&styled_line);
                } else {
                    output.push_str(&line);
                }
            }
            output.push('\n');
        }
    }

    // Add rename section
    if !plan.renames.is_empty() {
        if use_color {
            output.push_str(&format!(
                "\n{}\n",
                AnsiColor::Cyan.bold().paint("=== RENAMES ===")
            ));
        } else {
            output.push_str("\n=== RENAMES ===\n");
        }

        for rename in &plan.renames {
            let kind = match rename.kind {
                RenameKind::File => "file",
                RenameKind::Dir => "dir",
            };

            if use_color {
                output.push_str(&format!(
                    "{} {} {} {}\n",
                    AnsiColor::Yellow.paint(kind),
                    AnsiColor::Red.paint(rename.from.display().to_string()),
                    AnsiColor::White.paint("→"),
                    AnsiColor::Green.paint(rename.to.display().to_string())
                ));
            } else {
                output.push_str(&format!(
                    "{} {} → {}\n",
                    kind,
                    rename.from.display(),
                    rename.to.display()
                ));
            }
        }
    }

    Ok(output)
}

/// Render plan as JSON
fn render_json(plan: &Plan) -> Result<String> {
    Ok(serde_json::to_string_pretty(plan)?)
}

/// Write plan preview to stdout
pub fn write_preview(plan: &Plan, format: PreviewFormat, use_color: Option<bool>) -> Result<()> {
    let output = render_plan(plan, format, use_color)?;
    let mut stdout = io::stdout();
    write!(stdout, "{}", output)?;
    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_model::Style;
    use crate::scanner::Stats;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_plan() -> Plan {
        let mut matches_by_variant = HashMap::new();
        matches_by_variant.insert("old_name".to_string(), 2);
        matches_by_variant.insert("oldName".to_string(), 1);

        Plan {
            id: "test123".to_string(),
            created_at: "123456789".to_string(),
            old: "old_name".to_string(),
            new: "new_name".to_string(),
            styles: vec![Style::Snake, Style::Camel],
            includes: vec![],
            excludes: vec![],
            matches: vec![
                MatchHunk {
                    file: PathBuf::from("src/main.rs"),
                    line: 10,
                    col: 5,
                    variant: "old_name".to_string(),
                    before: "old_name".to_string(),
                    after: "new_name".to_string(),
                    start: 4,
                    end: 12,
                    line_before: Some("let old_name = 42;".to_string()),
                    line_after: Some("let new_name = 42;".to_string()),
                    coercion_applied: None,
                },
                MatchHunk {
                    file: PathBuf::from("src/main.rs"),
                    line: 20,
                    col: 10,
                    variant: "oldName".to_string(),
                    before: "oldName".to_string(),
                    after: "newName".to_string(),
                    start: 3,
                    end: 10,
                    line_before: Some("return oldName;".to_string()),
                    line_after: Some("return newName;".to_string()),
                    coercion_applied: None,
                },
            ],
            renames: vec![Rename {
                from: PathBuf::from("old_name.txt"),
                to: PathBuf::from("new_name.txt"),
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
        }
    }

    #[test]
    fn test_preview_format_from_str() {
        assert_eq!(PreviewFormat::from_str("table"), Some(PreviewFormat::Table));
        assert_eq!(PreviewFormat::from_str("diff"), Some(PreviewFormat::Diff));
        assert_eq!(PreviewFormat::from_str("json"), Some(PreviewFormat::Json));
        assert_eq!(PreviewFormat::from_str("TABLE"), Some(PreviewFormat::Table));
        assert_eq!(PreviewFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_render_table_no_color() {
        let plan = create_test_plan();
        let result = render_table_with_fixed_width(&plan, false, true).unwrap();

        assert!(result.contains("src/main.rs"));
        assert!(result.contains("Content"));
        assert!(result.contains("old_name"));
        assert!(result.contains("TOTALS"));
        assert!(result.contains("→ new_name.txt"));
    }

    #[test]
    fn test_render_diff_no_color() {
        let plan = create_test_plan();
        let result = render_diff(&plan, false).unwrap();

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
    fn test_render_diff_shows_full_line_context() {
        let mut matches_by_variant = HashMap::new();
        matches_by_variant.insert("old_func".to_string(), 1);

        // Create a plan with hunks that have full line context
        let plan = Plan {
            id: "test_full_line".to_string(),
            created_at: "123456789".to_string(),
            old: "old_func".to_string(),
            new: "new_func".to_string(),
            styles: vec![Style::Snake],
            includes: vec![],
            excludes: vec![],
            matches: vec![MatchHunk {
                file: PathBuf::from("src/lib.rs"),
                line: 42,
                col: 12,
                variant: "old_func".to_string(),
                // Word-level replacement for apply
                before: "old_func".to_string(),
                after: "new_func".to_string(),
                start: 17,
                end: 25,
                // Full line context for diff preview
                line_before: Some("    let result = old_func(param1, param2);".to_string()),
                line_after: Some("    let result = new_func(param1, param2);".to_string()),
                coercion_applied: None,
            }],
            renames: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 1,
                matches_by_variant,
                files_with_matches: 1,
            },
            version: "1.0.0".to_string(),
        };

        let result = render_diff(&plan, false).unwrap();

        // Should show the full line, not just the word
        assert!(result.contains("-    let result = old_func(param1, param2);"));
        assert!(result.contains("+    let result = new_func(param1, param2);"));

        // Should NOT show just the word alone
        assert!(!result.contains("-old_func\n"));
        assert!(!result.contains("+new_func\n"));
    }

    #[test]
    fn test_render_json() {
        let plan = create_test_plan();
        let result = render_json(&plan).unwrap();
        let parsed: Plan = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed.id, plan.id);
        assert_eq!(parsed.matches.len(), plan.matches.len());
        assert_eq!(parsed.renames.len(), plan.renames.len());
    }

    #[test]
    fn test_empty_plan() {
        let plan = Plan {
            id: "empty".to_string(),
            created_at: "0".to_string(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            renames: vec![],
            stats: Stats {
                files_scanned: 0,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
        };

        let table = render_table_with_fixed_width(&plan, false, true).unwrap();
        assert!(table.contains("TOTALS"));
        assert!(table.contains("0"));

        let diff = render_diff(&plan, false).unwrap();
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
        let output = render_plan(&plan, PreviewFormat::Table, Some(true)).unwrap();

        // Test with NO_COLOR set
        std::env::set_var("NO_COLOR", "1");
        let output_no_color = render_plan(&plan, PreviewFormat::Table, Some(false)).unwrap();

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
        use std::path::PathBuf;

        // Get the actual current working directory for testing
        let current_dir = std::env::current_dir().unwrap();

        // Test case that should be considered a root directory rename (current working directory)
        let root_rename = Rename {
            from: current_dir.clone(),
            to: PathBuf::from("renamed_refactoring_tool"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            is_root_directory_rename(&root_rename),
            "Current working directory should be root"
        );

        // Test case for a relative path - should NOT be considered root unless it matches current dir
        let relative_rename = Rename {
            from: PathBuf::from("project"),
            to: PathBuf::from("renamed_refactoring_tool"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            !is_root_directory_rename(&relative_rename),
            "Relative path should not be root unless it's the current directory"
        );

        // Test cases that should NOT be considered root directory renames
        let subdir_rename = Rename {
            from: current_dir.join("subdir"),
            to: PathBuf::from("renamed-refactoring-tool-subdir"),
            kind: RenameKind::Dir,
            coercion_applied: None,
        };
        assert!(
            !is_root_directory_rename(&subdir_rename),
            "Subdirectory should not be root"
        );

        let different_path_rename = Rename {
            from: PathBuf::from("/some/other/path"),
            to: PathBuf::from("/some/other/new_path"),
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
        let table_colored = render_plan(&plan, PreviewFormat::Table, Some(true)).unwrap();
        let diff_colored = render_plan(&plan, PreviewFormat::Diff, Some(true)).unwrap();

        // Set NO_COLOR for disabled test
        std::env::set_var("NO_COLOR", "1");
        let table_no_color = render_plan(&plan, PreviewFormat::Table, Some(false)).unwrap();
        let diff_no_color = render_plan(&plan, PreviewFormat::Diff, Some(false)).unwrap();

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

        // JSON format should never have colors regardless of setting
        let json_colored = render_plan(&plan, PreviewFormat::Json, Some(true)).unwrap();
        let json_no_color = render_plan(&plan, PreviewFormat::Json, Some(false)).unwrap();

        assert!(
            !json_colored.contains("\u{1b}["),
            "JSON format should never have colors"
        );
        assert!(
            !json_no_color.contains("\u{1b}["),
            "JSON format should never have colors"
        );
        assert_eq!(
            json_colored, json_no_color,
            "JSON output should be identical regardless of color setting"
        );
    }
}
