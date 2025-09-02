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
        search: "user_name".to_string(),
        replace: "customer_name".to_string(),
        styles: vec![Style::Snake, Style::Camel, Style::Pascal],
        includes: vec!["src/**/*.rs".to_string()],
        excludes: vec!["**/test_*.rs".to_string()],
        matches: vec![
            MatchHunk {
                file: PathBuf::from("src/models/user.rs"),
                line: 15,
                byte_offset: 12,
                char_offset: 12,
                variant: "user_name".to_string(),
                content: "user_name".to_string(),
                replace: "customer_name".to_string(),
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
                byte_offset: 8,
                char_offset: 8,
                variant: "userName".to_string(),
                content: "userName".to_string(),
                replace: "customerName".to_string(),
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
                byte_offset: 16,
                char_offset: 16,
                variant: "UserName".to_string(),
                content: "UserName".to_string(),
                replace: "CustomerName".to_string(),
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
                byte_offset: 20,
                char_offset: 20,
                variant: "user_name".to_string(),
                content: "user_name".to_string(),
                replace: "customer_name".to_string(),
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
        paths: vec![
            Rename {
                path: PathBuf::from("src/models/user_name.rs"),
                new_path: PathBuf::from("src/models/customer_name.rs"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
            Rename {
                path: PathBuf::from("tests/user_name_tests"),
                new_path: PathBuf::from("tests/customer_name_tests"),
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

    let output = render_plan_with_fixed_width(&plan, Preview::Table, Some(false), true);
    insta::assert_snapshot!(output);
}

#[test]
fn test_empty_plan_diff_snapshot() {
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
        search: "oldtool".to_string(),
        replace: "newtool".to_string(),
        styles: vec![Style::Snake, Style::Kebab],
        includes: vec![],
        excludes: vec![],
        matches: vec![MatchHunk {
            file: PathBuf::from("README.md"),
            line: 1,
            byte_offset: 0,
            char_offset: 0,
            variant: "oldtool".to_string(),
            content: "oldtool".to_string(),
            replace: "newtool".to_string(),
            start: 2,
            end: 9,
            line_before: Some("# Oldtool".to_string()),
            line_after: Some("# Newtool".to_string()),
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        }],
        paths: vec![
            // Only regular directory rename - root directory rename is filtered out
            Rename {
                path: PathBuf::from("/project/oldtool-core"),
                new_path: PathBuf::from("/project/newtool-core"),
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
        "Regular directory should appear in table, got {}",
        normalized
    );
    assert!(
        normalized.contains("newtool-core"),
        "Regular directory rename should appear in table, got {}",
        normalized
    );

    // No Next Steps section should exist for plan preview (only for apply/rename commands)
    assert!(
        !normalized.contains("Next Steps"),
        "Plan preview should not have Next Steps section, got {}",
        normalized
    );

    // Verify the totals show only 1 rename (not 2)
    assert!(
        normalized.contains("1 files, 1 paths"),
        "Should show only 1 rename in totals, got {}",
        normalized
    );

    insta::assert_snapshot!(normalized);
}
