use crate::scanner::{Plan, RenameKind};
use nu_ansi_term::{Color as AnsiColor, Style};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// Render search results as a focused matches view
#[allow(clippy::too_many_lines)]
pub fn render_matches(plan: &Plan, use_color: bool) -> String {
    let mut output = String::new();

    // Header
    if use_color {
        writeln!(
            output,
            "{}",
            AnsiColor::Cyan
                .bold()
                .paint(format!("🔍 Search Results for \"{}\"", plan.search))
        )
        .unwrap();
    } else {
        writeln!(output, "Search Results for \"{}\"", plan.search).unwrap();
    }
    writeln!(output).unwrap();

    // Show content matches grouped by file
    if !plan.matches.is_empty() {
        if use_color {
            writeln!(
                output,
                "{}",
                AnsiColor::Yellow.bold().paint("Content Matches:")
            )
            .unwrap();
        } else {
            writeln!(output, "Content Matches:").unwrap();
        }

        // Group matches by file
        let mut file_matches: HashMap<&Path, Vec<_>> = HashMap::new();
        for hunk in &plan.matches {
            file_matches.entry(&hunk.file).or_default().push(hunk);
        }

        // Sort files for deterministic output
        let mut sorted_files: Vec<_> = file_matches.keys().copied().collect();
        sorted_files.sort();

        for file in sorted_files {
            let hunks = &file_matches[&file];

            // Make path relative for cleaner display
            let file_str = match std::env::current_dir()
                .ok()
                .and_then(|cwd| file.strip_prefix(cwd).ok())
            {
                Some(relative_path) => relative_path.display().to_string(),
                None => file.display().to_string(),
            };

            if use_color {
                writeln!(output, "\n  {}", AnsiColor::Green.paint(&file_str)).unwrap();
            } else {
                writeln!(output, "\n  {}", file_str).unwrap();
            }

            // Sort hunks by line number
            let mut sorted_hunks = hunks.clone();
            sorted_hunks.sort_by_key(|h| (h.line, h.col));

            // Show up to first 5 matches per file with context
            let display_count = sorted_hunks.len().min(5);
            for hunk in sorted_hunks.iter().take(display_count) {
                if let Some(ref line_before) = hunk.line_before {
                    // Highlight the match in the line
                    let col = hunk.col as usize;
                    let end = col + hunk.content.len();

                    if use_color {
                        write!(output, "    {}:{}: ", hunk.line, hunk.col).unwrap();

                        // Print the line with the match highlighted with green background
                        if col > 0 && col <= line_before.len() {
                            write!(output, "{}", &line_before[..col]).unwrap();
                        }

                        if col < line_before.len() {
                            let actual_end = end.min(line_before.len());
                            write!(
                                output,
                                "{}",
                                Style::new()
                                    .on(AnsiColor::Rgb(0x00, 0xA9, 0x58))  // Same green as diff highlights
                                    .fg(AnsiColor::Rgb(0xFF, 0xFF, 0xFF))
                                    .paint(&line_before[col..actual_end])
                            )
                            .unwrap();

                            if actual_end < line_before.len() {
                                write!(output, "{}", &line_before[actual_end..]).unwrap();
                            }
                        }
                        writeln!(output).unwrap();
                    } else {
                        writeln!(
                            output,
                            "    {}:{}: {}",
                            hunk.line,
                            hunk.col,
                            line_before.trim()
                        )
                        .unwrap();
                    }
                } else {
                    // Fallback without line context
                    if use_color {
                        writeln!(
                            output,
                            "    {}:{}: {} ({})",
                            hunk.line,
                            hunk.col,
                            Style::new()
                                .on(AnsiColor::Yellow)
                                .fg(AnsiColor::Black)
                                .bold()
                                .paint(&hunk.content),
                            hunk.variant
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            output,
                            "    {}:{}: {} ({})",
                            hunk.line, hunk.col, hunk.content, hunk.variant
                        )
                        .unwrap();
                    }
                }
            }

            if hunks.len() > display_count {
                let remaining = hunks.len() - display_count;
                if use_color {
                    writeln!(
                        output,
                        "    {}",
                        AnsiColor::DarkGray.paint(format!("... and {} more matches", remaining))
                    )
                    .unwrap();
                } else {
                    writeln!(output, "    ... and {} more matches", remaining).unwrap();
                }
            }
        }
    }

    // Show file/directory matches
    if !plan.paths.is_empty() {
        writeln!(output).unwrap();
        if use_color {
            writeln!(
                output,
                "{}",
                AnsiColor::Yellow.bold().paint("Path Matches:")
            )
            .unwrap();
        } else {
            writeln!(output, "Path Matches:").unwrap();
        }

        // Separate files and directories
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for rename in &plan.paths {
            match rename.kind {
                RenameKind::File => files.push(rename),
                RenameKind::Dir => dirs.push(rename),
            }
        }

        // Show directories first
        if !dirs.is_empty() {
            if use_color {
                writeln!(output, "\n  {}", AnsiColor::Blue.paint("Directories:")).unwrap();
            } else {
                writeln!(output, "\n  Directories:").unwrap();
            }

            for rename in dirs {
                let path_str = match std::env::current_dir()
                    .ok()
                    .and_then(|cwd| rename.path.strip_prefix(cwd).ok())
                {
                    Some(relative_path) => relative_path.display().to_string(),
                    None => rename.path.display().to_string(),
                };

                if use_color {
                    writeln!(output, "    {}", AnsiColor::Green.paint(&path_str)).unwrap();
                } else {
                    writeln!(output, "    {}", path_str).unwrap();
                }
            }
        }

        // Show files
        if !files.is_empty() {
            if use_color {
                writeln!(output, "\n  {}", AnsiColor::Blue.paint("Files:")).unwrap();
            } else {
                writeln!(output, "\n  Files:").unwrap();
            }

            for rename in files {
                let path_str = match std::env::current_dir()
                    .ok()
                    .and_then(|cwd| rename.path.strip_prefix(cwd).ok())
                {
                    Some(relative_path) => relative_path.display().to_string(),
                    None => rename.path.display().to_string(),
                };

                if use_color {
                    writeln!(output, "    📄 {}", AnsiColor::Green.paint(&path_str)).unwrap();
                } else {
                    writeln!(output, "    {}", path_str).unwrap();
                }
            }
        }
    }

    // Summary
    writeln!(output).unwrap();
    if use_color {
        writeln!(output, "{}", AnsiColor::Cyan.paint("─".repeat(60))).unwrap();
    } else {
        writeln!(output, "{}", "─".repeat(60)).unwrap();
    }

    let summary = format!(
        "Found {} matches in {} files, {} paths to rename",
        plan.stats.total_matches,
        plan.stats.files_with_matches,
        plan.paths.len()
    );

    if use_color {
        writeln!(output, "{}", Style::new().bold().paint(&summary)).unwrap();
    } else {
        writeln!(output, "{}", summary).unwrap();
    }

    // Show variant breakdown
    if !plan.stats.matches_by_variant.is_empty() {
        let mut variants: Vec<_> = plan.stats.matches_by_variant.iter().collect();
        variants.sort_by_key(|(k, _)| k.as_str());

        write!(output, "Variants: ").unwrap();
        for (i, (variant, count)) in variants.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").unwrap();
            }
            if use_color {
                write!(
                    output,
                    "{} ({})",
                    AnsiColor::Yellow.paint(variant.as_str()),
                    count
                )
                .unwrap();
            } else {
                write!(output, "{} ({})", variant, count).unwrap();
            }
        }
        writeln!(output).unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{MatchHunk, Rename, Stats};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_plan() -> Plan {
        let mut stats = Stats {
            files_scanned: 5,
            total_matches: 3,
            matches_by_variant: HashMap::new(),
            files_with_matches: 2,
        };
        stats.matches_by_variant.insert("old_name".to_string(), 2);
        stats.matches_by_variant.insert("OldName".to_string(), 1);

        Plan {
            id: "test123".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            search: "old_name".to_string(),
            replace: "new_name".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![
                MatchHunk {
                    file: PathBuf::from("/project/src/lib.rs"),
                    line: 10,
                    col: 5,
                    variant: "old_name".to_string(),
                    content: "old_name".to_string(),
                    replace: "new_name".to_string(),
                    start: 0,
                    end: 8,
                    line_before: Some("    let old_name = 42;".to_string()),
                    line_after: Some("    let new_name = 42;".to_string()),
                    coercion_applied: None,
                    original_file: None,
                    renamed_file: None,
                    patch_hash: None,
                },
                MatchHunk {
                    file: PathBuf::from("/project/src/main.rs"),
                    line: 25,
                    col: 12,
                    variant: "OldName".to_string(),
                    content: "OldName".to_string(),
                    replace: "NewName".to_string(),
                    start: 0,
                    end: 7,
                    line_before: Some("    struct OldName {".to_string()),
                    line_after: Some("    struct NewName {".to_string()),
                    coercion_applied: None,
                    original_file: None,
                    renamed_file: None,
                    patch_hash: None,
                },
                // A match without line context
                MatchHunk {
                    file: PathBuf::from("/project/src/utils.rs"),
                    line: 1,
                    col: 0,
                    variant: "old_name".to_string(),
                    content: "old_name".to_string(),
                    replace: "new_name".to_string(),
                    start: 0,
                    end: 8,
                    line_before: None,
                    line_after: None,
                    coercion_applied: None,
                    original_file: None,
                    renamed_file: None,
                    patch_hash: None,
                },
            ],
            paths: vec![
                Rename {
                    path: PathBuf::from("/project/old_name.txt"),
                    new_path: PathBuf::from("/project/new_name.txt"),
                    kind: RenameKind::File,
                    coercion_applied: None,
                },
                Rename {
                    path: PathBuf::from("/project/old_name_dir"),
                    new_path: PathBuf::from("/project/new_name_dir"),
                    kind: RenameKind::Dir,
                    coercion_applied: None,
                },
            ],
            stats,
            version: "1.0.0".to_string(),
            created_directories: None,
        }
    }

    #[test]
    fn test_render_matches_with_color() {
        let plan = create_test_plan();
        let output = render_matches(&plan, true);

        // Should contain header
        assert!(output.contains("Search Results for \"old_name\""));

        // Should contain content matches section
        assert!(output.contains("Content Matches:"));

        // Should contain path matches section
        assert!(output.contains("Path Matches:"));

        // Should contain directories and files sections
        assert!(output.contains("Directories:"));
        assert!(output.contains("Files:"));

        // Should contain summary
        assert!(output.contains("Found 3 matches in 2 files, 2 paths to rename"));

        // Should contain variants
        assert!(output.contains("Variants:"));
        // With color codes, the variant names have ANSI escape sequences
        assert!(output.contains("old_name") && output.contains("(2)"));
        assert!(output.contains("OldName") && output.contains("(1)"));

        // Should contain color codes (ANSI escape sequences)
        assert!(output.contains("\u{1b}["));
    }

    #[test]
    fn test_render_matches_without_color() {
        let plan = create_test_plan();
        let output = render_matches(&plan, false);

        // Should contain all the same content sections
        assert!(output.contains("Search Results for \"old_name\""));
        assert!(output.contains("Content Matches:"));
        assert!(output.contains("Path Matches:"));
        assert!(output.contains("Directories:"));
        assert!(output.contains("Files:"));
        assert!(output.contains("Found 3 matches in 2 files, 2 paths to rename"));
        assert!(output.contains("Variants:"));

        // Should NOT contain color codes
        assert!(!output.contains("\u{1b}["));
    }

    #[test]
    fn test_render_matches_empty_plan() {
        let plan = Plan {
            id: "empty".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            search: "nothing".to_string(),
            replace: "something".to_string(),
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

        let output = render_matches(&plan, false);

        // Should still have header and summary
        assert!(output.contains("Search Results for \"nothing\""));
        assert!(output.contains("Found 0 matches in 0 files, 0 paths to rename"));

        // Should not have content or path sections
        assert!(!output.contains("Content Matches:"));
        assert!(!output.contains("Path Matches:"));
        assert!(!output.contains("Variants:"));
    }

    #[test]
    fn test_render_matches_many_matches_truncation() {
        let mut plan = create_test_plan();

        // Add more matches to the same file to test truncation
        let extra_matches: Vec<MatchHunk> = (0..10)
            .map(|i| MatchHunk {
                file: PathBuf::from("/project/src/lib.rs"),
                line: 50 + i,
                col: 5,
                variant: "old_name".to_string(),
                content: "old_name".to_string(),
                replace: "new_name".to_string(),
                start: 0,
                end: 8,
                line_before: Some(format!("    let old_name_{} = {};", i, i)),
                line_after: Some(format!("    let new_name_{} = {};", i, i)),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            })
            .collect();

        plan.matches.extend(extra_matches);

        let output = render_matches(&plan, false);

        // Should show truncation message
        assert!(output.contains("... and"));
        assert!(output.contains("more matches"));
    }

    #[test]
    fn test_render_matches_line_highlighting_edge_cases() {
        let mut plan = create_test_plan();

        // Test match at end of line
        plan.matches.push(MatchHunk {
            file: PathBuf::from("/project/edge.rs"),
            line: 1,
            col: 15,
            variant: "old_name".to_string(),
            content: "old_name".to_string(),
            replace: "new_name".to_string(),
            start: 0,
            end: 8,
            line_before: Some("    let x = old_name".to_string()),
            line_after: Some("    let x = new_name".to_string()),
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        });

        // Test match at beginning of line
        plan.matches.push(MatchHunk {
            file: PathBuf::from("/project/edge.rs"),
            line: 2,
            col: 0,
            variant: "old_name".to_string(),
            content: "old_name".to_string(),
            replace: "new_name".to_string(),
            start: 0,
            end: 8,
            line_before: Some("old_name = 42".to_string()),
            line_after: Some("new_name = 42".to_string()),
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        });

        let output = render_matches(&plan, true);
        assert!(output.contains("/project/edge.rs"));
    }

    #[test]
    fn test_render_matches_only_files() {
        let plan = Plan {
            id: "files_only".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: vec![Rename {
                path: PathBuf::from("/project/old.txt"),
                new_path: PathBuf::from("/project/new.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            }],
            stats: Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let output = render_matches(&plan, false);
        assert!(output.contains("Files:"));
        assert!(!output.contains("Directories:"));
        assert!(!output.contains("Content Matches:"));
    }

    #[test]
    fn test_render_matches_only_directories() {
        let plan = Plan {
            id: "dirs_only".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: vec![Rename {
                path: PathBuf::from("/project/old_dir"),
                new_path: PathBuf::from("/project/new_dir"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            }],
            stats: Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        let output = render_matches(&plan, false);
        assert!(output.contains("Directories:"));
        assert!(!output.contains("Files:"));
        assert!(!output.contains("Content Matches:"));
    }
}
