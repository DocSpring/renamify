use refaktor_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_hyphenated_capitalized_replacement() {
    // Test that "Refaktor-specific" becomes "RenamedRefactoringTool-specific"
    // when capitalized word appears in hyphenated context

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with hyphenated capitalized patterns
    let test_file = root.join("README.md");
    std::fs::write(
        &test_file,
        r#"# Documentation

- `.rfignore` - Refaktor-specific ignore patterns (useful for excluding files from refactoring without affecting Git)
- Use Refaktor-compatible tools for better integration
- The Refaktor-engine processes files efficiently
- Try refaktor-specific settings (lowercase should remain kebab)
- Run Refaktor-CLI for command-line usage
"#,
    )
    .unwrap();

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

    let plan = scan_repository(&root, "refaktor", "renamed_refactoring_tool", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== All matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // Should find and replace:
    // - "Refaktor-specific" -> "RenamedRefactoringTool-specific" (appears once in line 2)
    // - "Refaktor-compatible" -> "RenamedRefactoringTool-compatible"
    // - "Refaktor-engine" -> "RenamedRefactoringTool-engine"
    // - "refaktor-specific" -> "renamed-refactoring-tool-specific"
    // - "Refaktor-CLI" -> "RenamedRefactoringTool-CLI"

    assert!(
        plan.matches.len() >= 5,
        "Should find at least 5 matches, found {}",
        plan.matches.len()
    );

    // Check specific replacements
    let has_pascal_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "Refaktor-specific" && m.after == "RenamedRefactoringTool-specific");
    assert!(
        has_pascal_specific,
        "Should replace 'Refaktor-specific' with 'RenamedRefactoringTool-specific'"
    );

    let has_pascal_compatible = plan.matches.iter().any(|m| {
        m.before == "Refaktor-compatible" && m.after == "RenamedRefactoringTool-compatible"
    });
    assert!(
        has_pascal_compatible,
        "Should replace 'Refaktor-compatible' with 'RenamedRefactoringTool-compatible'"
    );

    let has_pascal_engine = plan
        .matches
        .iter()
        .any(|m| m.before == "Refaktor-engine" && m.after == "RenamedRefactoringTool-engine");
    assert!(
        has_pascal_engine,
        "Should replace 'Refaktor-engine' with 'RenamedRefactoringTool-engine'"
    );

    let has_kebab_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "refaktor-specific" && m.after == "renamed-refactoring-tool-specific");
    assert!(
        has_kebab_specific,
        "Should replace 'refaktor-specific' with 'renamed-refactoring-tool-specific'"
    );

    let has_pascal_cli = plan
        .matches
        .iter()
        .any(|m| m.before == "Refaktor-CLI" && m.after == "RenamedRefactoringTool-CLI");
    assert!(
        has_pascal_cli,
        "Should replace 'Refaktor-CLI' with 'RenamedRefactoringTool-CLI'"
    );
}

#[test]
fn test_train_case_replacement() {
    // Test that Train-Case style enables "Renamed-Refactoring-Tool-Specific" replacements

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with Train-Case patterns
    let test_file = root.join("docs.md");
    std::fs::write(
        &test_file,
        r#"# Train-Case Examples

- Refaktor-Specific-Settings for configuration
- Use Refaktor-Core-Engine for processing
- The Refaktor-Based-Solution works well
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use default styles (which now includes Train-Case)
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "refaktor", "renamed_refactoring_tool", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== Train-Case matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // With Train-Case enabled, should replace:
    // - "Refaktor-Specific-Settings" -> "Renamed-Refactoring-Tool-Specific-Settings"
    // - "Refaktor-Core-Engine" -> "Renamed-Refactoring-Tool-Core-Engine"
    // - "Refaktor-Based-Solution" -> "Renamed-Refactoring-Tool-Based-Solution"

    let has_train_specific = plan.matches.iter().any(|m| {
        m.before == "Refaktor-Specific-Settings"
            && m.after == "Renamed-Refactoring-Tool-Specific-Settings"
    });
    assert!(
        has_train_specific,
        "Should replace 'Refaktor-Specific-Settings' with 'Renamed-Refactoring-Tool-Specific-Settings'"
    );

    let has_train_core = plan.matches.iter().any(|m| {
        m.before == "Refaktor-Core-Engine" && m.after == "Renamed-Refactoring-Tool-Core-Engine"
    });
    assert!(
        has_train_core,
        "Should replace 'Refaktor-Core-Engine' with 'Renamed-Refactoring-Tool-Core-Engine'"
    );

    let has_train_based = plan.matches.iter().any(|m| {
        m.before == "Refaktor-Based-Solution"
            && m.after == "Renamed-Refactoring-Tool-Based-Solution"
    });
    assert!(
        has_train_based,
        "Should replace 'Refaktor-Based-Solution' with 'Renamed-Refactoring-Tool-Based-Solution'"
    );
}

#[test]
fn test_mixed_hyphenated_patterns() {
    // Test various edge cases with hyphenated patterns

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with mixed patterns
    let test_file = root.join("mixed.txt");
    std::fs::write(
        &test_file,
        r#"Various patterns:
- REFAKTOR-SPECIFIC (screaming snake in hyphenated context)
- refaktor-Specific (mixed case - unusual)
- Refaktor-specific-Tool (Pascal followed by lowercase in hyphenated)
- refaktor-CLI-version (kebab with acronym)
- The-Refaktor-Tool (Train-Case context)
"#,
    )
    .unwrap();

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

    let plan = scan_repository(&root, "refaktor", "renamed_refactoring_tool", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== Mixed hyphenated matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // Verify various patterns are handled appropriately
    assert!(
        plan.matches.len() >= 5,
        "Should find at least 5 matches in mixed patterns, found {}",
        plan.matches.len()
    );
}
