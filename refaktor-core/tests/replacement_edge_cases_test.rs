use refaktor_core::{apply_plan, scan_repository, ApplyOptions, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_format_string_and_env_var_replacement() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with the problematic patterns
    let test_content = r#"
// Format string patterns that should be replaced
let temp_file = temp_dir.join(format!("refaktor_{}.tmp", std::process::id()));
let log_name = "refaktor_{}.log";

// Environment variable patterns that should be replaced
let env_var = process.env.REFAKTOR_DEBUG;
let config = std::env::var("REFAKTOR_CONFIG").unwrap_or_default();

// Mixed case patterns
let mixed = refaktor_someCAMEL-case;
"#;

    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, test_content).unwrap();

    // Scan and generate plan
    let opts = PlanOptions::default();
    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
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
    let variant_map = refaktor_core::case_model::generate_variant_map(
        "refaktor",
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
        "Environment variable 'REFAKTOR_CONFIG' should be replaced with 'SMART_SEARCH_AND_REPLACE_CONFIG'"
    );

    assert!(
        modified_content.contains("smart_search_and_replace_someCAMEL-case"),
        "Mixed case 'refaktor_someCAMEL-case' should be replaced with 'smart_search_and_replace_someCAMEL-case'"
    );

    // These currently fail but should be fixed:
    assert!(
        modified_content.contains("smart_search_and_replace_{}.tmp"),
        "Format string 'refaktor_{{}}.tmp' should be replaced with 'smart_search_and_replace_{{}}.tmp'"
    );

    assert!(
        modified_content.contains("smart_search_and_replace_{}.log"),
        "Format string 'refaktor_{{}}.log' should be replaced with 'smart_search_and_replace_{{}}.log'"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_DEBUG"),
        "Environment variable 'REFAKTOR_DEBUG' should be replaced with 'SMART_SEARCH_AND_REPLACE_DEBUG'"
    );

    // Verify no instances of 'refaktor' remain
    assert!(
        !modified_content.to_lowercase().contains("refaktor"),
        "No instances of 'refaktor' should remain in the modified content"
    );
}

#[test]
fn test_screaming_snake_case_replacement() {
    let temp_dir = TempDir::new().unwrap();

    // Test file with various SCREAMING_SNAKE_CASE patterns
    let test_content = r#"
const REFAKTOR_VERSION: &str = "1.0.0";
const REFAKTOR_DEBUG: bool = true;
const REFAKTOR_CONFIG_PATH: &str = "/etc/refaktor";
let env = std::env::var("REFAKTOR_ENABLED").unwrap_or_default();
"#;

    let test_file = temp_dir.path().join("constants.rs");
    fs::write(&test_file, test_content).unwrap();

    // Scan and apply
    let opts = PlanOptions::default();
    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
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
        "REFAKTOR_VERSION should become SMART_SEARCH_AND_REPLACE_VERSION"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_DEBUG"),
        "REFAKTOR_DEBUG should become SMART_SEARCH_AND_REPLACE_DEBUG"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_CONFIG_PATH"),
        "REFAKTOR_CONFIG_PATH should become SMART_SEARCH_AND_REPLACE_CONFIG_PATH"
    );

    assert!(
        modified_content.contains("SMART_SEARCH_AND_REPLACE_ENABLED"),
        "REFAKTOR_ENABLED should become SMART_SEARCH_AND_REPLACE_ENABLED"
    );

    // Verify no instances of 'refaktor' remain
    assert!(
        !modified_content.to_lowercase().contains("refaktor"),
        "No instances of 'refaktor' should remain"
    );
}
