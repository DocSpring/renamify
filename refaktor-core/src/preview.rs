use crate::scanner::{MatchHunk, Plan, Rename, RenameKind};
use anyhow::Result;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use nu_ansi_term::{Color as AnsiColor, Style};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Render the plan in the specified format
pub fn render_plan(plan: &Plan, format: PreviewFormat, use_color: Option<bool>) -> Result<String> {
    let use_color = use_color.unwrap_or_else(|| io::stdout().is_terminal());
    
    match format {
        PreviewFormat::Table => render_table(plan, use_color),
        PreviewFormat::Diff => render_diff(plan, use_color),
        PreviewFormat::Json => render_json(plan),
    }
}

/// Render plan as a table
fn render_table(plan: &Plan, use_color: bool) -> Result<String> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    
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
    let mut sorted_files: Vec<_> = file_stats.keys().cloned().collect();
    sorted_files.sort();
    
    // Add content rows
    for file in sorted_files {
        let (count, variants) = &file_stats[&file];
        let file_str = file.display().to_string();
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
    
    // Add rename rows
    for rename in &plan.renames {
        let from_str = rename.from.display().to_string();
        let to_str = rename.to.display().to_string();
        let kind_str = match rename.kind {
            RenameKind::File => "Rename (File)",
            RenameKind::Dir => "Rename (Dir)",
        };
        
        if use_color {
            table.add_row(vec![
                Cell::new(&from_str),
                Cell::new(kind_str).fg(Color::Blue),
                Cell::new(""),
                Cell::new(&format!("→ {}", to_str)).fg(Color::Magenta),
            ]);
        } else {
            table.add_row(vec![
                &from_str,
                kind_str,
                "",
                &format!("→ {}", to_str),
            ]);
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
        table.add_row(vec![
            "─────────",
            "─────────",
            "─────────",
            "─────────",
        ]);
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
    let mut sorted_files: Vec<_> = file_hunks.keys().cloned().collect();
    sorted_files.sort();
    
    // Generate diffs for each file
    for file in sorted_files {
        let hunks = &file_hunks[&file];
        if use_color {
            output.push_str(&format!(
                "{}",
                AnsiColor::Cyan.bold().paint(format!("--- {}\n+++ {}\n", 
                    file.display(), 
                    file.display()))
            ));
        } else {
            output.push_str(&format!("--- {}\n+++ {}\n", file.display(), file.display()));
        }
        
        // Create a unified diff from all hunks in this file
        for hunk in hunks {
            let diff = TextDiff::from_lines(&hunk.before, &hunk.after);
            
            if use_color {
                output.push_str(&format!(
                    "{}",
                    AnsiColor::Blue.paint(format!("@@ line {} @@\n", hunk.line))
                ));
            } else {
                output.push_str(&format!("@@ line {} @@\n", hunk.line));
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
                    coercion_applied: None,
                },
            ],
            renames: vec![
                Rename {
                    from: PathBuf::from("old_name.txt"),
                    to: PathBuf::from("new_name.txt"),
                    kind: RenameKind::File,
                    coercion_applied: None,
                },
            ],
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
        let result = render_table(&plan, false).unwrap();
        
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
        assert!(result.contains("-old_name"));
        assert!(result.contains("+new_name"));
        assert!(result.contains("=== RENAMES ==="));
        assert!(result.contains("old_name.txt → new_name.txt"));
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
        
        let table = render_table(&plan, false).unwrap();
        assert!(table.contains("TOTALS"));
        assert!(table.contains("0"));
        
        let diff = render_diff(&plan, false).unwrap();
        assert!(diff.is_empty() || diff == "\n");
    }
}