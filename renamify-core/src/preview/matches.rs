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
