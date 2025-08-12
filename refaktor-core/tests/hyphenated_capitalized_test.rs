use refaktor_core::{scan_repository, PlanOptions, Style};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_hyphenated_capitalized_replacement() {
    // Test that "Refaktor-specific" becomes "SmartSearchAndReplace-specific"
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

    let plan = scan_repository(&root, "refaktor", "smart_search_and_replace", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== All matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // Should find and replace:
    // - "Refaktor-specific" -> "SmartSearchAndReplace-specific" (appears once in line 2)
    // - "Refaktor-compatible" -> "SmartSearchAndReplace-compatible"
    // - "Refaktor-engine" -> "SmartSearchAndReplace-engine"
    // - "refaktor-specific" -> "smart-search-and-replace-specific"
    // - "Refaktor-CLI" -> "SmartSearchAndReplace-CLI"

    assert!(
        plan.matches.len() >= 5,
        "Should find at least 5 matches, found {}",
        plan.matches.len()
    );

    // Check specific replacements
    let has_pascal_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "Refaktor-specific" && m.after == "SmartSearchAndReplace-specific");
    assert!(
        has_pascal_specific,
        "Should replace 'Refaktor-specific' with 'SmartSearchAndReplace-specific'"
    );

    let has_pascal_compatible = plan.matches.iter().any(|m| {
        m.before == "Refaktor-compatible" && m.after == "SmartSearchAndReplace-compatible"
    });
    assert!(
        has_pascal_compatible,
        "Should replace 'Refaktor-compatible' with 'SmartSearchAndReplace-compatible'"
    );

    let has_pascal_engine = plan
        .matches
        .iter()
        .any(|m| m.before == "Refaktor-engine" && m.after == "SmartSearchAndReplace-engine");
    assert!(
        has_pascal_engine,
        "Should replace 'Refaktor-engine' with 'SmartSearchAndReplace-engine'"
    );

    let has_kebab_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "refaktor-specific" && m.after == "smart-search-and-replace-specific");
    assert!(
        has_kebab_specific,
        "Should replace 'refaktor-specific' with 'smart-search-and-replace-specific'"
    );

    let has_pascal_cli = plan
        .matches
        .iter()
        .any(|m| m.before == "Refaktor-CLI" && m.after == "SmartSearchAndReplace-CLI");
    assert!(
        has_pascal_cli,
        "Should replace 'Refaktor-CLI' with 'SmartSearchAndReplace-CLI'"
    );
}

#[test]
fn test_train_case_replacement() {
    // Test that Train-Case style enables "Smart-Search-And-Replace-Specific" replacements

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
        styles: Some(vec![
            Style::Snake,
            Style::Kebab,
            Style::Camel,
            Style::Pascal,
            Style::Train, // Enable Train-Case
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "refaktor", "smart_search_and_replace", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== Train-Case matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // With Train-Case enabled, should replace:
    // - "Refaktor-Specific-Settings" -> "Smart-Search-And-Replace-Specific-Settings"
    // - "Refaktor-Core-Engine" -> "Smart-Search-And-Replace-Core-Engine"
    // - "Refaktor-Based-Solution" -> "Smart-Search-And-Replace-Based-Solution"

    let has_train_specific = plan.matches.iter().any(|m| {
        m.before == "Refaktor-Specific-Settings"
            && m.after == "Smart-Search-And-Replace-Specific-Settings"
    });
    assert!(
        has_train_specific,
        "Should replace 'Refaktor-Specific-Settings' with 'Smart-Search-And-Replace-Specific-Settings'"
    );

    let has_train_core = plan.matches.iter().any(|m| {
        m.before == "Refaktor-Core-Engine" && m.after == "Smart-Search-And-Replace-Core-Engine"
    });
    assert!(
        has_train_core,
        "Should replace 'Refaktor-Core-Engine' with 'Smart-Search-And-Replace-Core-Engine'"
    );

    let has_train_based = plan.matches.iter().any(|m| {
        m.before == "Refaktor-Based-Solution"
            && m.after == "Smart-Search-And-Replace-Based-Solution"
    });
    assert!(
        has_train_based,
        "Should replace 'Refaktor-Based-Solution' with 'Smart-Search-And-Replace-Based-Solution'"
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

    let plan = scan_repository(&root, "refaktor", "smart_search_and_replace", &options).unwrap();

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
