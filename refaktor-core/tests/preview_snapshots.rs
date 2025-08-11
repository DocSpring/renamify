use refaktor_core::{
    preview::{render_plan, PreviewFormat},
    scanner::{MatchHunk, Plan, Rename, RenameKind, Stats},
    Style,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn normalize_paths(s: String) -> String {
    // Normalize path separators for cross-platform compatibility
    s.replace('\\', "/")
}

fn create_sample_plan() -> Plan {
    let mut matches_by_variant = HashMap::new();
    matches_by_variant.insert("user_name".to_string(), 3);
    matches_by_variant.insert("userName".to_string(), 2);
    matches_by_variant.insert("UserName".to_string(), 1);
    
    Plan {
        id: "abc123def456".to_string(),
        created_at: "1234567890".to_string(),
        old: "user_name".to_string(),
        new: "customer_name".to_string(),
        styles: vec![Style::Snake, Style::Camel, Style::Pascal],
        includes: vec!["src/**/*.rs".to_string()],
        excludes: vec!["**/test_*.rs".to_string()],
        matches: vec![
            MatchHunk {
                file: PathBuf::from("src/models/user.rs"),
                line: 15,
                col: 12,
                variant: "user_name".to_string(),
                before: "    let user_name = String::new();".to_string(),
                after: "    let customer_name = String::new();".to_string(),
                start: 11,
                end: 20,
        coercion_applied: None,
            },
            MatchHunk {
                file: PathBuf::from("src/models/user.rs"),
                line: 25,
                col: 8,
                variant: "userName".to_string(),
                before: "    pub userName: String,".to_string(),
                after: "    pub customerName: String,".to_string(),
                start: 7,
                end: 15,
        coercion_applied: None,
            },
            MatchHunk {
                file: PathBuf::from("src/api/handlers.rs"),
                line: 42,
                col: 16,
                variant: "UserName".to_string(),
                before: "struct UserName {".to_string(),
                after: "struct CustomerName {".to_string(),
                start: 7,
                end: 15,
        coercion_applied: None,
            },
            MatchHunk {
                file: PathBuf::from("src/api/handlers.rs"),
                line: 50,
                col: 20,
                variant: "user_name".to_string(),
                before: "    fn get_user_name(&self) -> &str {".to_string(),
                after: "    fn get_customer_name(&self) -> &str {".to_string(),
                start: 11,
                end: 20,
        coercion_applied: None,
            },
        ],
        renames: vec![
            Rename {
                from: PathBuf::from("src/models/user_name.rs"),
                to: PathBuf::from("src/models/customer_name.rs"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
            Rename {
                from: PathBuf::from("tests/user_name_tests"),
                to: PathBuf::from("tests/customer_name_tests"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            },
        ],
        stats: Stats {
            files_scanned: 25,
            total_matches: 6,
            matches_by_variant,
            files_with_matches: 2,
        },
        version: "1.0.0".to_string(),
    }
}

#[test]
fn test_table_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan(&plan, PreviewFormat::Table, Some(false)).unwrap();
    let normalized = normalize_paths(output);
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_table_format_with_color_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan(&plan, PreviewFormat::Table, Some(true)).unwrap();
    // For colored output, we'll snapshot it but CI tests will use non-colored
    insta::assert_snapshot!(output);
}

#[test]
fn test_diff_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan(&plan, PreviewFormat::Diff, Some(false)).unwrap();
    let normalized = normalize_paths(output);
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_diff_format_with_color_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan(&plan, PreviewFormat::Diff, Some(true)).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_json_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan(&plan, PreviewFormat::Json, Some(false)).unwrap();
    // Parse and re-serialize to ensure consistent formatting
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let normalized = serde_json::to_string_pretty(&parsed).unwrap();
    insta::assert_snapshot!(normalize_paths(normalized));
}

#[test]
fn test_empty_plan_table_snapshot() {
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
    
    let output = render_plan(&plan, PreviewFormat::Table, Some(false)).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_empty_plan_diff_snapshot() {
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
    
    let output = render_plan(&plan, PreviewFormat::Diff, Some(false)).unwrap();
    insta::assert_snapshot!(output);
}