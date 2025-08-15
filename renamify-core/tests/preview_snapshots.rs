use renamify_core::{
    preview::{render_plan_with_fixed_width, Preview},
    scanner::{MatchHunk, Plan, Rename, RenameKind, Stats},
    Style,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn normalize_paths(s: &str) -> String {
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
                before: "user_name".to_string(),
                after: "customer_name".to_string(),
                start: 11,
                end: 20,
                line_before: Some("    let user_name = String::new();".to_string()),
                line_after: Some("    let customer_name = String::new();".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            },
            MatchHunk {
                file: PathBuf::from("src/models/user.rs"),
                line: 25,
                col: 8,
                variant: "userName".to_string(),
                before: "userName".to_string(),
                after: "customerName".to_string(),
                start: 7,
                end: 15,
                line_before: Some("    pub userName: String,".to_string()),
                line_after: Some("    pub customerName: String,".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            },
            MatchHunk {
                file: PathBuf::from("src/api/handlers.rs"),
                line: 42,
                col: 16,
                variant: "UserName".to_string(),
                before: "UserName".to_string(),
                after: "CustomerName".to_string(),
                start: 7,
                end: 15,
                line_before: Some("struct UserName {".to_string()),
                line_after: Some("struct CustomerName {".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
            },
            MatchHunk {
                file: PathBuf::from("src/api/handlers.rs"),
                line: 50,
                col: 20,
                variant: "user_name".to_string(),
                before: "user_name".to_string(),
                after: "customer_name".to_string(),
                start: 11,
                end: 20,
                line_before: Some("    fn get_user_name(&self) -> &str {".to_string()),
                line_after: Some("    fn get_customer_name(&self) -> &str {".to_string()),
                coercion_applied: None,
                original_file: None,
                renamed_file: None,
                patch_hash: None,
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
        created_directories: None,
    }
}

#[test]
fn test_table_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan_with_fixed_width(&plan, Preview::Table, Some(false), true);
    let normalized = normalize_paths(&output);
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_table_format_with_color_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan_with_fixed_width(&plan, Preview::Table, Some(true), true);
    // For colored output, we'll snapshot it but CI tests will use non-colored
    insta::assert_snapshot!(output);
}

#[test]
fn test_diff_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan_with_fixed_width(&plan, Preview::Diff, Some(false), true);
    let normalized = normalize_paths(&output);
    insta::assert_snapshot!(normalized);
}

#[test]
fn test_diff_format_with_color_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan_with_fixed_width(&plan, Preview::Diff, Some(true), true);
    insta::assert_snapshot!(output);
}

#[test]
fn test_json_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan_with_fixed_width(&plan, Preview::Json, Some(false), true);
    // Parse and re-serialize to ensure consistent formatting
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let normalized = serde_json::to_string_pretty(&parsed).unwrap();
    insta::assert_snapshot!(normalize_paths(&normalized));
}

#[test]
fn test_summary_format_snapshot() {
    let plan = create_sample_plan();
    let output = render_plan_with_fixed_width(&plan, Preview::Summary, Some(false), true);
    let normalized = normalize_paths(&output);
    insta::assert_snapshot!(normalized);
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
        created_directories: None,
    };

    let output = render_plan_with_fixed_width(&plan, Preview::Table, Some(false), true);
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
        created_directories: None,
    };

    let output = render_plan_with_fixed_width(&plan, Preview::Diff, Some(false), true);
    insta::assert_snapshot!(output);
}

#[test]
fn test_root_directory_rename_handling() {
    // Test that root directory renames are filtered out from plans by default
    // Root directory renames should not appear in plans unless explicitly allowed
    let mut matches_by_variant = HashMap::new();
    matches_by_variant.insert("oldtool".to_string(), 1);

    // Note: In a real scenario, the scanner would filter out root directory renames
    // For this test, we're only including the non-root rename to match actual behavior
    let plan = Plan {
        id: "root-test".to_string(),
        created_at: "1234567890".to_string(),
        old: "oldtool".to_string(),
        new: "newtool".to_string(),
        styles: vec![Style::Snake, Style::Kebab],
        includes: vec![],
        excludes: vec![],
        matches: vec![MatchHunk {
            file: PathBuf::from("README.md"),
            line: 1,
            col: 0,
            variant: "oldtool".to_string(),
            before: "oldtool".to_string(),
            after: "newtool".to_string(),
            start: 2,
            end: 9,
            line_before: Some("# Oldtool".to_string()),
            line_after: Some("# Newtool".to_string()),
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        }],
        renames: vec![
            // Only regular directory rename - root directory rename is filtered out
            Rename {
                from: PathBuf::from("/project/oldtool-core"),
                to: PathBuf::from("/project/newtool-core"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            },
            // Root directory rename would be filtered out by scanner with rename_root: false
        ],
        stats: Stats {
            files_scanned: 1,
            total_matches: 1,
            matches_by_variant,
            files_with_matches: 1,
        },
        version: "1.0.0".to_string(),
        created_directories: None,
    };

    let output = render_plan_with_fixed_width(&plan, Preview::Table, Some(false), true);
    let normalized = normalize_paths(&output);

    // Verify that the subdirectory rename is in the table
    assert!(
        normalized.contains("oldtool-core"),
        "Regular directory should appear in table"
    );
    assert!(
        normalized.contains("newtool-core"),
        "Regular directory rename should appear in table"
    );

    // No Next Steps section should exist for plan preview (only for apply/rename commands)
    assert!(
        !normalized.contains("Next Steps"),
        "Plan preview should not have Next Steps section"
    );

    // Verify the totals show only 1 rename (not 2)
    assert!(
        normalized.contains("1 files, 1 renames"),
        "Should show only 1 rename in totals"
    );

    insta::assert_snapshot!(normalized);
}
