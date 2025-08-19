use crate::scanner::{Plan, RenameKind};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// Render plan as AI-friendly summary format
pub fn render_summary(plan: &Plan) -> String {
    let mut output = String::new();

    // Header with basic info
    let is_search = plan.replace.is_empty();
    if is_search {
        writeln!(output, "[SEARCH RESULTS]").unwrap();
        writeln!(output, "Search: {}", plan.search).unwrap();
    } else {
        writeln!(output, "[PLAN SUMMARY]").unwrap();
        writeln!(output, "Search: {}", plan.search).unwrap();
        writeln!(output, "Replace: {}", plan.replace).unwrap();
    }
    writeln!(output, "Matches: {}", plan.stats.total_matches).unwrap();
    writeln!(output, "Files: {}", plan.stats.files_with_matches).unwrap();
    writeln!(output, "Paths: {}", plan.paths.len()).unwrap();
    writeln!(output).unwrap();

    // Content changes grouped by file
    if !plan.matches.is_empty() {
        writeln!(output, "[CONTENT]").unwrap();

        // Group matches by file
        let mut file_stats: HashMap<&Path, HashMap<String, usize>> = HashMap::new();
        for hunk in &plan.matches {
            *file_stats
                .entry(&hunk.file)
                .or_default()
                .entry(hunk.variant.clone())
                .or_insert(0) += 1;
        }

        // Sort files for deterministic output
        let mut sorted_files: Vec<_> = file_stats.keys().copied().collect();
        sorted_files.sort();

        for file in sorted_files {
            let variant_counts = &file_stats[&file];
            let total_matches: usize = variant_counts.values().sum();

            // Make path relative for cleaner display
            let file_str = match std::env::current_dir()
                .ok()
                .and_then(|cwd| file.strip_prefix(cwd).ok())
            {
                Some(relative_path) => relative_path.display().to_string(),
                None => file.display().to_string(),
            };

            write!(output, "{}: {} matches", file_str, total_matches).unwrap();

            // List variants with counts
            let mut variants: Vec<_> = variant_counts.iter().collect();
            variants.sort_by_key(|(v, _)| v.as_str());

            write!(output, " [").unwrap();
            for (i, (variant, count)) in variants.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ").unwrap();
                }
                write!(output, "{}: {}", variant, count).unwrap();
            }
            writeln!(output, "]").unwrap();
        }
    }

    // File and directory renames
    if !plan.paths.is_empty() {
        writeln!(output).unwrap();
        writeln!(output, "[PATHS]").unwrap();
        for rename in &plan.paths {
            let kind = match rename.kind {
                RenameKind::File => "file",
                RenameKind::Dir => "dir",
            };

            // Make paths relative for cleaner display
            let from_str = match std::env::current_dir()
                .ok()
                .and_then(|cwd| rename.path.strip_prefix(cwd).ok())
            {
                Some(relative_path) => relative_path.display().to_string(),
                None => rename.path.display().to_string(),
            };

            if is_search {
                // For search, just show the matching file/dir
                writeln!(output, "{}: {}", kind, from_str).unwrap();
            } else {
                // For plan, show the rename
                let to_str = match std::env::current_dir()
                    .ok()
                    .and_then(|cwd| rename.new_path.strip_prefix(cwd).ok())
                {
                    Some(relative_path) => relative_path.display().to_string(),
                    None => rename.new_path.display().to_string(),
                };
                writeln!(output, "{}: {} -> {}", kind, from_str, to_str).unwrap();
            }
        }
    }

    output
}
