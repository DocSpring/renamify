use renamify_core::{apply_plan, scan_repository, ApplyOptions, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_mixed_case_prefix_replacement() {
    // Test that when replacing lowercase identifiers that appear after uppercase prefixes,
    // the replacement maintains the correct case (lowercase) rather than being coerced to uppercase

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Test content with CARGO_BIN_EXE_ prefix followed by lowercase identifier
    let content = r#"
fn main() {
    // This tests the CARGO_BIN_EXE_ pattern where the suffix should remain lowercase
    let path = env!("CARGO_BIN_EXE_foobar");
    let another = env!("CARGO_BIN_EXE_foobar");

    // Also test with other mixed patterns
    const PREFIX_foobar: &str = "test";
    let DEBUG_foobar_enabled = true;

    // Regular lowercase should still work
    let foobar_config = "config";
    let simple_foobar = "test";
}
"#;

    fs::write(&test_file, content).unwrap();

    // Search for "foobar" and replace with "baz_qux"
    let mut plan = scan_repository(
        temp_dir.path(),
        "foobar",
        "baz_qux",
        &PlanOptions::default(),
    )
    .unwrap();

    // Apply the plan
    apply_plan(&mut plan, &ApplyOptions::default()).unwrap();

    // Read the modified content
    let result = fs::read_to_string(&test_file).unwrap();

    // The key assertion: CARGO_BIN_EXE_foobar should become CARGO_BIN_EXE_baz_qux
    // NOT CARGO_BIN_EXE_BAZ_QUX
    assert!(
        result.contains(r#"env!("CARGO_BIN_EXE_baz_qux")"#),
        "Expected CARGO_BIN_EXE_baz_qux (lowercase), but got: {}",
        result
    );

    // Other mixed patterns should also preserve the lowercase
    assert!(
        result.contains("PREFIX_baz_qux"),
        "Expected PREFIX_baz_qux (lowercase suffix)"
    );
    assert!(
        result.contains("DEBUG_baz_qux_enabled"),
        "Expected DEBUG_baz_qux_enabled (lowercase in middle)"
    );

    // Regular lowercase replacements
    assert!(result.contains("baz_qux_config"));
    assert!(result.contains("simple_baz_qux"));
}

#[test]
fn test_screaming_snake_with_lowercase_suffix() {
    // More specific test for the SCREAMING_SNAKE_lowercase pattern
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("build.rs");

    let content = r#"
// Build script with environment variables
const CARGO_PKG_NAME_oldtool: &str = "oldtool";
const CARGO_BIN_EXE_oldtool: &str = "/path/to/oldtool";
const RUST_VERSION_oldtool: &str = "1.70.0";

// Should not change fully uppercase
const CARGO_OLDTOOL_VERSION: &str = "1.0.0";
"#;

    fs::write(&test_file, content).unwrap();

    // Replace "oldtool" with "new_tool"
    let mut plan = scan_repository(
        temp_dir.path(),
        "oldtool",
        "new_tool",
        &PlanOptions::default(),
    )
    .unwrap();

    apply_plan(&mut plan, &ApplyOptions::default()).unwrap();

    let result = fs::read_to_string(&test_file).unwrap();

    // These should have lowercase replacements
    assert!(
        result.contains("CARGO_PKG_NAME_new_tool"),
        "Expected lowercase new_tool after CARGO_PKG_NAME_"
    );
    assert!(
        result.contains("CARGO_BIN_EXE_new_tool"),
        "Expected lowercase new_tool after CARGO_BIN_EXE_"
    );
    assert!(
        result.contains("RUST_VERSION_new_tool"),
        "Expected lowercase new_tool after RUST_VERSION_"
    );

    // This should be fully uppercase since the original was fully uppercase
    assert!(
        result.contains("CARGO_NEW_TOOL_VERSION"),
        "Expected uppercase NEW_TOOL when replacing within fully uppercase context"
    );
}
