use refaktor_core::{apply_plan, scan_repository, ApplyOptions, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_train_case_patterns_are_replaced_in_apply() {
    // This test verifies that Train-Case patterns like "Refaktor-Core-Engine"
    // are actually replaced when applying the plan

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with Train-Case patterns
    let test_file = root.join("test.md");
    let original_content = r#"# Documentation

## Configuration
- Refaktor-Specific-Settings for configuration
- Use Refaktor-Core-Engine for processing
- The Refaktor-Based-Solution works well

## Comments
    // - "Refaktor-Specific-Settings" -> "Renamed-Refactoring-Tool-Specific-Settings"
    // - "Refaktor-Core-Engine" -> "Renamed-Refactoring-Tool-Core-Engine"
    // - "Refaktor-Based-Solution" -> "Renamed-Refactoring-Tool-Based-Solution"
"#;

    std::fs::write(&test_file, original_content).unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    // Create the plan
    let mut plan = scan_repository(&root, "refaktor", "renamed_refactoring_tool", &options).unwrap();

    // Debug: Print matches found
    println!("\n=== Matches found in plan ===");
    for m in &plan.matches {
        if m.before.contains("-") {
            println!("'{}' -> '{}'", m.before, m.after);
        }
    }

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: root.join(".backups"),
        create_backups: true,
        atomic: true,
        commit: false,
        force: false,
        skip_symlinks: false,
        log_file: None,
    };

    apply_plan(&mut plan, &apply_options).unwrap();

    println!("\n=== Apply completed ===");

    // Read the modified content
    let modified_content = std::fs::read_to_string(&test_file).unwrap();

    println!("\n=== Modified content ===");
    println!("{}", modified_content);

    // Check that Train-Case patterns were actually replaced
    assert!(
        !modified_content.contains("Refaktor-Core-Engine"),
        "File should not contain 'Refaktor-Core-Engine' after replacement"
    );

    assert!(
        modified_content.contains("Renamed-Refactoring-Tool-Core-Engine"),
        "File should contain 'Renamed-Refactoring-Tool-Core-Engine' after replacement"
    );

    assert!(
        !modified_content.contains("Refaktor-Specific-Settings"),
        "File should not contain 'Refaktor-Specific-Settings' after replacement"
    );

    assert!(
        modified_content.contains("Renamed-Refactoring-Tool-Specific-Settings"),
        "File should contain 'Renamed-Refactoring-Tool-Specific-Settings' after replacement"
    );

    assert!(
        !modified_content.contains("Refaktor-Based-Solution"),
        "File should not contain 'Refaktor-Based-Solution' after replacement"
    );

    assert!(
        modified_content.contains("Renamed-Refactoring-Tool-Based-Solution"),
        "File should contain 'Renamed-Refactoring-Tool-Based-Solution' after replacement"
    );
}
