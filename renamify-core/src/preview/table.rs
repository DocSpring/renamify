use crate::scanner::{Plan, RenameKind};
use comfy_table::{Cell, Color, ColumnConstraint, ContentArrangement, Table, Width};
use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::path::Path;

/// Render plan as a table with optional fixed column widths
pub fn render_table(plan: &Plan, use_color: bool, fixed_table_width: bool) -> String {
    let mut table = Table::new();

    // Set content arrangement and constraints based on fixed width parameter
    if fixed_table_width {
        table.set_content_arrangement(ContentArrangement::Disabled);
        // Set absolute column widths for consistent layout
        table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(75)), // File
            ColumnConstraint::Absolute(Width::Fixed(30)), // Kind
            ColumnConstraint::Absolute(Width::Fixed(15)), // Matches
            ColumnConstraint::Absolute(Width::Fixed(75)), // Variants
        ]);
    } else {
        // Use TTY detection fallback when no fixed width specified
        if io::stdout().is_terminal() {
            table.set_content_arrangement(ContentArrangement::Dynamic);
        } else {
            table.set_content_arrangement(ContentArrangement::Disabled);
            table.set_constraints(vec![
                ColumnConstraint::Absolute(Width::Fixed(75)), // File
                ColumnConstraint::Absolute(Width::Fixed(30)), // Kind
                ColumnConstraint::Absolute(Width::Fixed(15)), // Matches
                ColumnConstraint::Absolute(Width::Fixed(75)), // Variants
            ]);
        }
    }

    // Force styling even in non-TTY environments when colors are explicitly requested
    if use_color {
        table.enforce_styling();
    }

    // Set headers
    let is_search = plan.replace.is_empty();
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
    let mut file_stats: HashMap<&Path, (usize, HashMap<String, usize>)> = HashMap::new();
    for hunk in &plan.matches {
        let entry = file_stats
            .entry(&hunk.file)
            .or_insert_with(|| (0, HashMap::new()));
        entry.0 += 1;
        *entry.1.entry(hunk.variant.clone()).or_insert(0) += 1;
    }

    // Sort files for deterministic output
    let mut sorted_files: Vec<_> = file_stats.keys().copied().collect();
    sorted_files.sort();

    // Add content rows
    for file in sorted_files {
        let (count, variant_counts) = &file_stats[&file];
        // Make path relative to current directory for cleaner display
        let file_str = match std::env::current_dir()
            .ok()
            .and_then(|cwd| file.strip_prefix(cwd).ok())
        {
            Some(relative_path) => relative_path.display().to_string(),
            None => file.display().to_string(),
        };
        // Show variants with their per-file counts in parentheses
        let mut variants_with_counts: Vec<String> = variant_counts
            .iter()
            .map(|(variant, count)| format!("{} ({})", variant, count))
            .collect();
        // Sort for deterministic output
        variants_with_counts.sort();
        let variants_str = variants_with_counts.join(", ");

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
    for rename in &plan.paths {
        // Make paths relative to current directory for cleaner display
        let from_str = match std::env::current_dir()
            .ok()
            .and_then(|cwd| rename.path.strip_prefix(cwd).ok())
        {
            Some(relative_path) => relative_path.display().to_string(),
            None => rename.path.display().to_string(),
        };

        let kind_str = match rename.kind {
            RenameKind::File => "File",
            RenameKind::Dir => "Dir",
        };

        if is_search {
            // For search, just show the path without rename arrow
            if use_color {
                table.add_row(vec![
                    Cell::new(&from_str),
                    Cell::new(kind_str).fg(Color::Blue),
                    Cell::new(""),
                    Cell::new(""),
                ]);
            } else {
                table.add_row(vec![&from_str, kind_str, "", ""]);
            }
        } else {
            // For plan, show the rename with arrow
            let to_str = match std::env::current_dir()
                .ok()
                .and_then(|cwd| rename.new_path.strip_prefix(cwd).ok())
            {
                Some(relative_path) => relative_path.display().to_string(),
                None => rename.new_path.display().to_string(),
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
    }

    // Add footer with totals
    let total_matches = plan.stats.total_matches;
    let total_files = plan.stats.files_with_matches;
    let total_paths = plan.paths.len();

    if use_color {
        table.add_row(vec![
            Cell::new("─────────").fg(Color::DarkGrey),
            Cell::new("─────────").fg(Color::DarkGrey),
            Cell::new("─────────").fg(Color::DarkGrey),
            Cell::new("─────────").fg(Color::DarkGrey),
        ]);
        table.add_row(vec![
            Cell::new("TOTALS").fg(Color::Cyan),
            Cell::new(format!("{} files, {} paths", total_files, total_paths)).fg(Color::White),
            Cell::new(total_matches.to_string()).fg(Color::Yellow),
            Cell::new(format!("{} variants", plan.stats.matches_by_variant.len())).fg(Color::White),
        ]);
    } else {
        table.add_row(vec!["─────────", "─────────", "─────────", "─────────"]);
        table.add_row(vec![
            "TOTALS",
            &format!("{} files, {} paths", total_files, total_paths),
            &total_matches.to_string(),
            &format!("{} variants", plan.stats.matches_by_variant.len()),
        ]);
    }

    table.to_string()
}
