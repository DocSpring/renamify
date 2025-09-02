use renamify_core::{apply_plan, scan_repository, ApplyOptions, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_train_case_patterns_are_replaced_in_apply() {
    // This test verifies that Train-Case patterns like "Renamify-Core-Engine"
    // are actually replaced when applying the plan

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with Train-Case patterns
    let test_file = root.join("test.md");
    let original_content = r#"# Documentation

## Configuration
- Renamify-Specific-Settings for configuration
- Use Renamify-Core-Engine for processing
- The Renamify-Based-Solution works well

## Comments
    // - "Renamify-Specific-Settings" -> "Renamed-Renaming-Tool-Specific-Settings"
    // - "Renamify-Core-Engine" -> "Renamed-Renaming-Tool-Core-Engine"
    // - "Renamify-Based-Solution" -> "Renamed-Renaming-Tool-Based-Solution"
"#;

    std::fs::write(&test_file, original_content).unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    // Create the plan
    let mut plan = scan_repository(&root, "renamify", "renamed_renaming_tool", &options).unwrap();

    // Debug: Print matches found
    println!("\n=== Matches found in plan ===");
    for m in &plan.matches {
        if m.content.contains("-") {
            println!("'{}' -> '{}'", m.content, m.replace);
        }
    }

    // Apply the plan
    let apply_options = ApplyOptions {
        backup_dir: root.join(".backups"),
        create_backups: true,
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
        !modified_content.contains("Renamify-Core-Engine"),
        "File should not contain 'Renamify-Core-Engine' after replacement"
    );

    assert!(
        modified_content.contains("Renamed-Renaming-Tool-Core-Engine"),
        "File should contain 'Renamed-Renaming-Tool-Core-Engine' after replacement"
    );

    assert!(
        !modified_content.contains("Renamify-Specific-Settings"),
        "File should not contain 'Renamify-Specific-Settings' after replacement"
    );

    assert!(
        modified_content.contains("Renamed-Renaming-Tool-Specific-Settings"),
        "File should contain 'Renamed-Renaming-Tool-Specific-Settings' after replacement"
    );

    assert!(
        !modified_content.contains("Renamify-Based-Solution"),
        "File should not contain 'Renamify-Based-Solution' after replacement"
    );

    assert!(
        modified_content.contains("Renamed-Renaming-Tool-Based-Solution"),
        "File should contain 'Renamed-Renaming-Tool-Based-Solution' after replacement"
    );
}
