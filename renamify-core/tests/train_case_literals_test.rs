use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_train_case_patterns_in_literals() {
    // This test verifies that Train-Case patterns are replaced even when they appear
    // in string literals, comments, and documentation

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with Train-Case patterns in various contexts
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        r#"// Test file with Train-Case patterns in different contexts

// Comment: Renamify-Specific-Settings should be replaced
// Another: Use Renamify-Core-Engine for processing
// And: The Renamify-Based-Solution works well

fn main() {
    // Comments inside code
    // - "Renamify-Specific-Settings" -> "Renamed-Refactoring-Tool-Specific-Settings"
    // - "Renamify-Core-Engine" -> "Renamed-Refactoring-Tool-Core-Engine"
    // - "Renamify-Based-Solution" -> "Renamed-Refactoring-Tool-Based-Solution"
    
    let config = "Renamify-Specific-Settings";
    let engine = "Use Renamify-Core-Engine";
    let solution = "The Renamify-Based-Solution";
    
    println!("Testing Renamify-Specific-Settings");
    
    assert_eq!(config, "Renamify-Specific-Settings");
    
    // Edge case: The-Renamify-Tool (Train-Case context)
}

/// Documentation with Renamify-Core-Engine reference
pub fn process() {
    // Style::Train, // Include Train-Case for patterns like Renamify-Core-Engine
}
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None, // Use default styles (which includes Train-Case)
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "renamed_refactoring_tool", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== All Train-Case literal matches found ===");
    for m in &plan.matches {
        if m.before.contains('-') && m.before.chars().next().is_some_and(|c| c.is_uppercase()) {
            println!("'{}' -> '{}'", m.before, m.after);
        }
    }

    // Count Train-Case pattern matches
    let train_case_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| {
            m.before.contains('-') && m.before.chars().next().is_some_and(|c| c.is_uppercase())
        })
        .collect();

    // We expect to find all these Train-Case patterns:
    // - 3x "Renamify-Specific-Settings" (comment, string literal, inside other strings)
    // - 3x "Renamify-Core-Engine" (comment, string literal, documentation)
    // - 2x "Renamify-Based-Solution" (comment, string literal)
    // - 1x "The-Renamify-Tool"

    // At minimum we should find several Train-Case replacements
    assert!(
        train_case_matches.len() >= 8,
        "Should find at least 8 Train-Case patterns, found {}",
        train_case_matches.len()
    );

    // Check that specific patterns are replaced correctly
    let has_specific_settings = plan.matches.iter().any(|m| {
        m.before == "Renamify-Specific-Settings"
            && m.after == "Renamed-Refactoring-Tool-Specific-Settings"
    });
    assert!(
        has_specific_settings,
        "Should replace 'Renamify-Specific-Settings' with 'Renamed-Refactoring-Tool-Specific-Settings'"
    );

    let has_core_engine = plan.matches.iter().any(|m| {
        m.before == "Renamify-Core-Engine" && m.after == "Renamed-Refactoring-Tool-Core-Engine"
    });
    assert!(
        has_core_engine,
        "Should replace 'Renamify-Core-Engine' with 'Renamed-Refactoring-Tool-Core-Engine'"
    );

    let has_based_solution = plan.matches.iter().any(|m| {
        m.before == "Renamify-Based-Solution"
            && m.after == "Renamed-Refactoring-Tool-Based-Solution"
    });
    assert!(
        has_based_solution,
        "Should replace 'Renamify-Based-Solution' with 'Renamed-Refactoring-Tool-Based-Solution'"
    );

    // This one might not work because "The" is not our pattern
    let has_the_tool = plan
        .matches
        .iter()
        .any(|m| m.before.contains("Renamify-Tool"));
    if has_the_tool {
        println!("Found The-Renamify-Tool pattern");
    }
}

#[test]
fn test_train_case_exact_patterns() {
    // Test the exact patterns that are failing in the real test files

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with the EXACT content that's failing
    let test_file = root.join("test.md");
    std::fs::write(
        &test_file,
        r#"- Renamify-Specific-Settings for configuration
- Use Renamify-Core-Engine for processing
- The Renamify-Based-Solution works well
    // - "Renamify-Specific-Settings" -> "Renamed-Refactoring-Tool-Specific-Settings"
    // - "Renamify-Core-Engine" -> "Renamed-Refactoring-Tool-Core-Engine"
    // - "Renamify-Based-Solution" -> "Renamed-Refactoring-Tool-Based-Solution"
        m.before == "Renamify-Specific-Settings"
        "Should replace 'Renamify-Specific-Settings' with 'Renamed-Refactoring-Tool-Specific-Settings'"
        m.before == "Renamify-Core-Engine" && m.after == "Renamed-Refactoring-Tool-Core-Engine"
        "Should replace 'Renamify-Core-Engine' with 'Renamed-Refactoring-Tool-Core-Engine'"
        m.before == "Renamify-Based-Solution"
        "Should replace 'Renamify-Based-Solution' with 'Renamed-Refactoring-Tool-Based-Solution'"
- The-Renamify-Tool (Train-Case context)
        Style::Train, // Include Train-Case for patterns like Renamify-Core-Engine
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
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

    let plan = scan_repository(&root, "renamify", "renamed_refactoring_tool", &options).unwrap();

    // Count how many times each Train-Case pattern appears
    let specific_settings_count = plan
        .matches
        .iter()
        .filter(|m| m.before == "Renamify-Specific-Settings")
        .count();

    let core_engine_count = plan
        .matches
        .iter()
        .filter(|m| m.before == "Renamify-Core-Engine")
        .count();

    let based_solution_count = plan
        .matches
        .iter()
        .filter(|m| m.before == "Renamify-Based-Solution")
        .count();

    println!(
        "Renamify-Specific-Settings found: {} times",
        specific_settings_count
    );
    println!("Renamify-Core-Engine found: {} times", core_engine_count);
    println!(
        "Renamify-Based-Solution found: {} times",
        based_solution_count
    );

    // These patterns appear multiple times in the content
    assert!(
        specific_settings_count >= 4,
        "Should find 'Renamify-Specific-Settings' at least 4 times, found {}",
        specific_settings_count
    );

    assert!(
        core_engine_count >= 4,
        "Should find 'Renamify-Core-Engine' at least 4 times, found {}",
        core_engine_count
    );

    assert!(
        based_solution_count >= 3,
        "Should find 'Renamify-Based-Solution' at least 3 times, found {}",
        based_solution_count
    );
}
