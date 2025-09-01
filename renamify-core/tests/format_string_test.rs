use renamify_core::operations::plan::plan_operation;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_format_string_with_underscore_preserves_snake_case() {
    // Test that format strings like 'greattool_{}.tmp' correctly become 'awesome_tool_name_{}.tmp'
    // The underscore after greattool indicates snake_case context
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with format strings
    let test_file = temp_path.join("test.rs");
    fs::write(
        &test_file,
        r#"
fn main() {
    let file1 = "greattool_{{}}.tmp";
    let file2 = "greattool_{{}}.log";
    let file3 = format!("greattool_{{}}_backup.txt", id);
    // Also test in string literals within assertions
    assert!(true, "Format string 'greattool_{{}}.tmp' should be replaced");
    // Also test without underscore for comparison
    let file4 = "greattool.txt";
}
"#,
    )
    .unwrap();

    // Scan and replace with all the required parameters
    let (result, _preview) = plan_operation(
        "greattool",
        "awesome_tool_name",
        vec![temp_path.to_path_buf()],
        vec![], // include
        vec![], // exclude
        true,   // respect_gitignore
        0,      // unrestricted_level
        true,   // rename_files
        true,   // rename_dirs
        &[],    // exclude_styles
        &[],    // include_styles
        &[],    // only_styles
        vec![], // exclude_match
        None,   // exclude_matching_lines
        None,   // plan_out
        None,   // preview_format
        true,   // dry_run
        false,  // fixed_table_width
        false,  // use_color
        false,  // no_acronyms
        vec![], // custom_acronyms
        vec![], // atomic_search
        vec![], // atomic_replace
        None,   // cwd
        None,   // atomic_config
    )
    .unwrap();

    // Get the plan from the result
    let plan = result.plan.expect("Should have a plan");

    // Find the matches
    let matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("test.rs"))
        .collect();

    // Should find all instances
    assert!(!matches.is_empty(), "Should find matches for 'greattool'");

    // Check that format strings with underscore become snake_case
    for m in &matches {
        if m.content == "greattool" {
            if let Some(ref line_before) = m.line_before {
                if line_before.contains("greattool_") {
                    assert_eq!(
                        m.replace, "awesome_tool_name",
                        "Format string with double curly brackets should preserve snake_case, got: {}",
                        m.replace
                    );
                    // The full line should have the underscore preserved
                    if let Some(ref line_after) = m.line_after {
                        assert!(
                            line_after.contains("awesome_tool_name_"),
                            "Line should contain 'awesome_tool_name_', got: {}",
                            line_after
                        );
                    }
                    // Make sure it's NOT lowercase without underscores
                    assert_ne!(
                        m.replace, "awesometoolname",
                        "Should NOT be all lowercase without underscores"
                    );
                }
            }
        }
    }
}

#[test]
fn test_ambiguous_with_following_underscore_is_snake_case() {
    // Even more direct test - if we have 'tool_' it's obviously snake_case
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let test_file = temp_path.join("config.toml");
    fs::write(
        &test_file,
        r#"
cache_dir = "mytool_cache"
log_file = "mytool_debug.log"
template = "mytool_{}.tmp"
"#,
    )
    .unwrap();

    let (result, _preview) = plan_operation(
        "mytool",
        "super_awesome_tool",
        vec![temp_path.to_path_buf()],
        vec![],          // include
        vec![],          // exclude
        true,            // respect_gitignore
        0,               // unrestricted_level
        true,            // rename_files
        true,            // rename_dirs
        &[],             // exclude_styles
        &[],             // include_styles
        &[],             // only_styles
        vec![],          // exclude_match
        None,            // exclude_matching_lines
        None,            // plan_out
        None,            // preview_format
        true,            // dry_run
        false,           // fixed_table_width
        false,           // use_color
        false,           // no_acronyms
        vec![],          // custom_acronyms
        vec![],          // atomic_search
        vec![],          // atomic_replace
        Some(temp_path), // cwd - set to temp dir (expects &Path)
        None,            // atomic_config
    )
    .unwrap();

    let plan = result.plan.expect("Should have a plan");
    for m in &plan.matches {
        if m.content == "mytool" {
            // All instances should be replaced with snake_case
            assert_eq!(
                m.replace, "super_awesome_tool",
                "mytool in context '{:?}' should become super_awesome_tool, got: {}",
                m.line_before, m.replace
            );
            // Absolutely should NOT be this
            assert_ne!(
                m.replace, "superawesometool",
                "Should NEVER remove underscores to create 'superawesometool'"
            );
        }
    }
}
