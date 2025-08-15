use renamify_core::{apply_plan, scan_repository, ApplyOptions, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_format_string_and_env_var_replacement() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with the problematic patterns
    let test_content = r#"
// Format string patterns that should be replaced
let temp_file = temp_dir.join(format!("renamify_{}.tmp", std::process::id()));
let log_name = "renamify_{}.log";

// Environment variable patterns that should be replaced
let env_var = process.env.RENAMIFY_DEBUG;
let config = std::env::var("RENAMIFY_CONFIG").unwrap_or_default();

// Mixed case patterns
let mixed = renamify_someCAMEL-case;
"#;

    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, test_content).unwrap();

    // Scan and generate plan
    let opts = PlanOptions::default();
    let mut plan = scan_repository(
        temp_dir.path(),
        "renamify",
        "smart_search_and_replace",
        &opts,
    )
    .unwrap();

    // Apply the plan
    let backup_dir = temp_dir.path().join("backups");
    let apply_opts = ApplyOptions {
        create_backups: true,
        backup_dir,
        atomic: true,
        commit: false,
        force: false,
        skip_symlinks: false,
        log_file: None,
    };

    apply_plan(&mut plan, &apply_opts).unwrap();

    // Read the modified content
    let modified_content = fs::read_to_string(&test_file).unwrap();

    println!("Modified content:\n{}", modified_content);

    // Debug: print the plan to see what was matched
    println!("\nMatches found:");
    for m in &plan.matches {
        println!("  {} at {}:{} -> {}", m.before, m.line, m.col, m.after);
    }

    // Debug: print variant mappings
    println!("\nVariant mappings generated:");
    let variant_map = renamify_core::case_model::generate_variant_map(
        "renamify",
        "smart_search_and_replace",
        None,
    );
    for (old, new) in &variant_map {
        println!("  '{}' -> '{}'", old, new);
    }

    // Verify ALL patterns are replaced correctly (these should all pass when bugs are fixed)

    // These work correctly:
    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_CONFIG"),
        "Environment variable 'RENAMIFY_CONFIG' should be replaced with 'SMART_SEARCH_AND_REPLACE_CONFIG'"
    );

    assert!(
        modified_content.contains("smart_search_and_replace_someCAMEL-case"),
        "Mixed case 'renamify_someCAMEL-case' should be replaced with 'smart_search_and_replace_someCAMEL-case'"
    );

    // These currently fail but should be fixed:
    assert!(
        modified_content.contains("smart_search_and_replace_{}.tmp"),
        "Format string 'renamify_{{}}.tmp' should be replaced with 'smart_search_and_replace_{{}}.tmp'"
    );

    assert!(
        modified_content.contains("smart_search_and_replace_{}.log"),
        "Format string 'renamify_{{}}.log' should be replaced with 'smart_search_and_replace_{{}}.log'"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_DEBUG"),
        "Environment variable 'RENAMIFY_DEBUG' should be replaced with 'SMART_SEARCH_AND_REPLACE_DEBUG'"
    );

    // Verify no instances of 'renamify' remain
    assert!(
        !modified_content.to_lowercase().contains("renamify"),
        "No instances of 'renamify' should remain in the modified content"
    );
}

#[test]
fn test_screaming_snake_case_replacement() {
    let temp_dir = TempDir::new().unwrap();

    // Test file with various SCREAMING_SNAKE_CASE patterns
    let test_content = r#"
const RENAMIFY_VERSION: &str = "1.0.0";
const RENAMIFY_DEBUG: bool = true;
const RENAMIFY_CONFIG_PATH: &str = "/etc/renamify";
let env = std::env::var("RENAMIFY_ENABLED").unwrap_or_default();
"#;

    let test_file = temp_dir.path().join("constants.rs");
    fs::write(&test_file, test_content).unwrap();

    // Scan and apply
    let opts = PlanOptions::default();
    let mut plan = scan_repository(
        temp_dir.path(),
        "renamify",
        "smart_search_and_replace",
        &opts,
    )
    .unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let apply_opts = ApplyOptions {
        create_backups: true,
        backup_dir,
        atomic: true,
        commit: false,
        force: false,
        skip_symlinks: false,
        log_file: None,
    };

    apply_plan(&mut plan, &apply_opts).unwrap();

    let modified_content = fs::read_to_string(&test_file).unwrap();

    println!("Modified content:\n{}", modified_content);

    // Verify all SCREAMING_SNAKE_CASE patterns were replaced
    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_VERSION"),
        "RENAMIFY_VERSION should become SMART_SEARCH_AND_REPLACE_VERSION"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_DEBUG"),
        "RENAMIFY_DEBUG should become SMART_SEARCH_AND_REPLACE_DEBUG"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_CONFIG_PATH"),
        "RENAMIFY_CONFIG_PATH should become SMART_SEARCH_AND_REPLACE_CONFIG_PATH"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_ENABLED"),
        "RENAMIFY_ENABLED should become SMART_SEARCH_AND_REPLACE_ENABLED"
    );

    // Verify no instances of 'renamify' remain
    assert!(
        !modified_content.to_lowercase().contains("renamify"),
        "No instances of 'renamify' should remain"
    );
}
