use crate::scanner::{MatchHunk, Plan, RenameKind};
use nu_ansi_term::{Color as AnsiColor, Style};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// Highlight specific ranges in a line with background color
fn highlight_line_with_hunks(
    line: &str,
    hunks: &[&MatchHunk],
    is_delete: bool,
    use_color: bool,
) -> String {
    if !use_color || hunks.is_empty() {
        return line.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;

    // Sort hunks by column position
    let mut sorted_hunks = hunks.to_vec();
    sorted_hunks.sort_by_key(|h| h.byte_offset);

    // Base style for the whole line (Claude Code custom colors)
    let base_style = if is_delete {
        // Deleted line background: #852134
        Style::new()
            .on(AnsiColor::Rgb(0x85, 0x21, 0x34))
            .fg(AnsiColor::Rgb(0xFF, 0xFF, 0xFF))
    } else {
        // Added line background: #005e24
        Style::new()
            .on(AnsiColor::Rgb(0x00, 0x5E, 0x24))
            .fg(AnsiColor::Rgb(0xFF, 0xFF, 0xFF))
    };

    for hunk in sorted_hunks {
        let col = hunk.byte_offset as usize;
        let term = if is_delete {
            &hunk.content
        } else {
            &hunk.replace
        };

        // Add the part before the match with base style
        if col > last_end && col <= line.len() {
            result.push_str(&base_style.paint(&line[last_end..col]).to_string());
        }

        // Add the highlighted match with brighter background (word-level highlight)
        let end = (col + term.len()).min(line.len());
        if col < line.len() {
            let highlight_style = if is_delete {
                // Deleted match highlight: #c0526a
                Style::new()
                    .on(AnsiColor::Rgb(0xC0, 0x52, 0x6A))
                    .fg(AnsiColor::Rgb(0xFF, 0xFF, 0xFF))
            } else {
                // Added match highlight: #00a958
                Style::new()
                    .on(AnsiColor::Rgb(0x00, 0xA9, 0x58))
                    .fg(AnsiColor::Rgb(0xFF, 0xFF, 0xFF))
            };
            result.push_str(&highlight_style.paint(&line[col..end]).to_string());
            last_end = end;
        }
    }

    // Add any remaining part with base style
    if last_end < line.len() {
        result.push_str(&base_style.paint(&line[last_end..]).to_string());
    }

    result
}

/// Render plan as unified diffs
pub fn render_diff(plan: &Plan, use_color: bool) -> String {
    let mut output = String::new();
    let _is_search = plan.replace.is_empty();

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

        // Make path relative to current directory for cleaner display
        let relative_path = match std::env::current_dir()
            .ok()
            .and_then(|cwd| file.strip_prefix(cwd).ok())
        {
            Some(rel) => rel,
            None => file,
        };

        // Use forward slashes for consistent cross-platform output
        let file_str = if cfg!(windows) {
            relative_path.to_string_lossy().replace('\\', "/")
        } else {
            relative_path.to_string_lossy().to_string()
        };

        if use_color {
            write!(
                output,
                "{}",
                Style::new()
                    .fg(AnsiColor::White)
                    .bold()
                    .paint(format!("--- {}\n+++ {}\n", file_str, file_str))
            )
            .unwrap();
        } else {
            write!(output, "--- {}\n+++ {}\n", file_str, file_str).unwrap();
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
                first_hunk.content.clone()
            };

            // For the after text, if we have multiple hunks we need to apply all changes
            let after_text = if line_hunk_group.len() == 1 {
                // Single hunk - use line_after if available
                if let Some(ref line_after) = first_hunk.line_after {
                    line_after.clone()
                } else {
                    // Fallback: just the replacement word
                    first_hunk.replace.clone()
                }
            } else {
                // Multiple hunks on same line - apply all replacements
                let mut after_line = before_text.clone();

                // Sort hunks by column position (reverse order to avoid position shifts)
                let mut sorted_hunks = line_hunk_group.clone();
                sorted_hunks.sort_by(|a, b| b.byte_offset.cmp(&a.byte_offset));

                // Apply replacements from right to left to maintain positions
                for hunk in sorted_hunks {
                    let col = hunk.byte_offset as usize;
                    if col < after_line.len() && after_line[col..].starts_with(&hunk.content) {
                        let end = col + hunk.content.len();
                        after_line.replace_range(col..end, &hunk.replace);
                    }
                }

                after_line
            };

            let diff = TextDiff::from_lines(&before_text, &after_text);

            if use_color {
                write!(
                    output,
                    "{}",
                    AnsiColor::Blue.paint(format!("@@ line {} @@\n", line_num))
                )
                .unwrap();
            } else {
                writeln!(output, "@@ line {} @@", line_num).unwrap();
            }

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };

                let change_text = change.to_string();
                let change_text = change_text.trim_end_matches('\n');

                if use_color {
                    // Apply highlighting to the actual changed parts
                    let highlighted = match change.tag() {
                        ChangeTag::Delete => {
                            let highlighted_line = highlight_line_with_hunks(
                                change_text,
                                line_hunk_group,
                                true,
                                use_color,
                            );
                            format!("{}{}\n", AnsiColor::Red.paint(sign), highlighted_line)
                        },
                        ChangeTag::Insert => {
                            let highlighted_line = highlight_line_with_hunks(
                                change_text,
                                line_hunk_group,
                                false,
                                use_color,
                            );
                            format!("{}{}\n", AnsiColor::Green.paint(sign), highlighted_line)
                        },
                        ChangeTag::Equal => format!("{}{}\n", sign, change_text),
                    };
                    output.push_str(&highlighted);
                } else {
                    writeln!(output, "{}{}", sign, change_text).unwrap();
                }
            }
            output.push('\n');
        }
    }

    // Add rename section
    if !plan.paths.is_empty() {
        if use_color {
            write!(
                output,
                "\n{}\n",
                AnsiColor::Cyan.bold().paint("=== RENAMES ===")
            )
            .unwrap();
        } else {
            output.push_str("\n=== RENAMES ===\n");
        }

        for rename in &plan.paths {
            let kind = match rename.kind {
                RenameKind::File => "file",
                RenameKind::Dir => "dir",
            };

            if use_color {
                writeln!(
                    output,
                    "{} {} {} {}",
                    AnsiColor::Yellow.paint(kind),
                    AnsiColor::Red.paint(rename.path.display().to_string()),
                    AnsiColor::White.paint("→"),
                    AnsiColor::Green.paint(rename.new_path.display().to_string())
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "{} {} → {}",
                    kind,
                    rename.path.display(),
                    rename.new_path.display()
                )
                .unwrap();
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::MatchHunk;
    use std::path::PathBuf;

    #[test]
    fn test_highlight_line_with_hunks() {
        // Test the off-by-one bug in highlighting
        // The line: require "active_support/core_ext/hash"
        // Should highlight "core_ext" but was highlighting "ore_ext/"

        let line = r#"require "active_support/core_ext/hash""#;

        // Create a hunk that matches "core_ext" at position 24
        let hunk = MatchHunk {
            file: PathBuf::from("test.rb"),
            line: 1,
            byte_offset: 24, // Position of 'c' in 'core_ext' (0-based)
            char_offset: 24, // Same as byte_offset for ASCII text
            variant: "core_ext".to_string(),
            content: "core_ext".to_string(),
            replace: "ruby_extras".to_string(),
            start: 24,
            end: 32,
            line_before: Some(line.to_string()),
            line_after: Some(r#"require "active_support/ruby_extras/hash""#.to_string()),
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        };

        let hunks = vec![&hunk];

        // Test delete line highlighting (red background)
        let highlighted = highlight_line_with_hunks(line, &hunks, true, true);

        // The ANSI codes should highlight exactly "core_ext", not "ore_ext/"
        // Base style for deleted line: bg=#852134, fg=#FFFFFF
        // Highlight style for match: bg=#c0526a, fg=#FFFFFF

        // Check that the output contains the correct structure:
        // 1. Base style from start to col 24: "require \"active_support/"
        // 2. Highlight style from col 24 to 32: "core_ext"
        // 3. Base style from col 32 to end: "/hash\""

        // The expected output with correct ANSI codes
        // Note: nu_ansi_term combines bg and fg into a single escape sequence
        let expected = "\u{1b}[48;2;133;33;52;38;2;255;255;255mrequire \"active_support/\u{1b}[0m\u{1b}[48;2;192;82;106;38;2;255;255;255mcore_ext\u{1b}[0m\u{1b}[48;2;133;33;52;38;2;255;255;255m/hash\"\u{1b}[0m";

        assert_eq!(
            highlighted, expected,
            "Highlighting should cover exactly 'core_ext', not 'ore_ext/'"
        );
    }
}
