use renamify_core::{scan_repository_multi, PlanOptions};
use tempfile::TempDir;

#[test]
fn test_double_underscore_prefix_preserved() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with double underscore patterns
    let test_file = temp_dir.path().join("test.js");
    std::fs::write(
        &test_file,
        r#"// JavaScript double underscore pattern
window.__cgwSentryCspTest = true;
const value = window.__cgwSentryCspTest;

// Other patterns
const __cgwPrivate = 'test';
this.__cgwProperty = value;
"#,
    )
    .unwrap();

    let options = PlanOptions::default();
    let plan =
        scan_repository_multi(&[temp_dir.path().to_path_buf()], "cgw", "rgw", &options).unwrap();

    // Read the expected replacements
    let content = std::fs::read_to_string(&test_file).unwrap();

    // Check that we have matches
    assert!(
        !plan.matches.is_empty(),
        "Should find matches for cgw pattern"
    );

    // Verify each match preserves the __ prefix
    for match_hunk in &plan.matches {
        if let Some(line_after) = &match_hunk.line_after {
            // All replacements should preserve the __ prefix
            if content
                .lines()
                .nth((match_hunk.line - 1) as usize)
                .unwrap()
                .contains("__cgw")
            {
                assert!(
                    line_after.contains("__rgw"),
                    "Double underscore prefix should be preserved. Got: {}",
                    line_after
                );

                // Should NOT convert to snake_case
                assert!(
                    !line_after.contains("rgw_Sentry"),
                    "Should not convert to snake_case with lost prefix. Got: {}",
                    line_after
                );
            }
        }
    }

    // Specific assertions for each pattern
    let has_window_test = plan.matches.iter().any(|m| {
        m.line_after
            .as_ref()
            .is_some_and(|line| line.contains("window.__rgwSentryCspTest"))
    });
    assert!(
        has_window_test,
        "Should preserve window.__rgwSentryCspTest pattern"
    );

    let has_const = plan.matches.iter().any(|m| {
        m.line_after
            .as_ref()
            .is_some_and(|line| line.contains("const __rgwPrivate"))
    });
    assert!(has_const, "Should preserve const __rgwPrivate pattern");

    let has_this = plan.matches.iter().any(|m| {
        m.line_after
            .as_ref()
            .is_some_and(|line| line.contains("this.__rgwProperty"))
    });
    assert!(has_this, "Should preserve this.__rgwProperty pattern");
}
