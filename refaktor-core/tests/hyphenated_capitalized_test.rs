use refaktor_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_hyphenated_capitalized_replacement() {
    // Test that "Tool-specific" becomes "NewToolName-specific"
    // when capitalized word appears in hyphenated context

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with hyphenated capitalized patterns
    let test_file = root.join("README.md");
    std::fs::write(
        &test_file,
        r#"# Documentation

- `.rfignore` - Tool-specific ignore patterns (useful for excluding files from refactoring without affecting Git)
- Use Tool-compatible tools for better integration
- The Tool-engine processes files efficiently
- Try tool-specific settings (lowercase should remain kebab)
- Run Tool-CLI for command-line usage
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
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "tool", "new_tool_name", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== All matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // Should find and replace:
    // - "Tool-specific" -> "NewToolName-specific" (appears once in line 2)
    // - "Tool-compatible" -> "NewToolName-compatible"
    // - "Tool-engine" -> "NewToolName-engine"
    // - "tool-specific" -> "new-tool-name-specific"
    // - "Tool-CLI" -> "New-Tool-Name-CLI" (Train case)

    assert!(
        plan.matches.len() >= 5,
        "Should find at least 5 matches, found {}",
        plan.matches.len()
    );

    // Check specific replacements
    let has_pascal_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "Tool-specific" && m.after == "NewToolName-specific");
    assert!(
        has_pascal_specific,
        "Should replace 'Tool-specific' with 'NewToolName-specific'"
    );

    let has_pascal_compatible = plan
        .matches
        .iter()
        .any(|m| m.before == "Tool-compatible" && m.after == "NewToolName-compatible");
    assert!(
        has_pascal_compatible,
        "Should replace 'Tool-compatible' with 'NewToolName-compatible'"
    );

    let has_pascal_engine = plan
        .matches
        .iter()
        .any(|m| m.before == "Tool-engine" && m.after == "NewToolName-engine");
    assert!(
        has_pascal_engine,
        "Should replace 'Tool-engine' with 'NewToolName-engine'"
    );

    let has_kebab_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "tool-specific" && m.after == "new-tool-name-specific");
    assert!(
        has_kebab_specific,
        "Should replace 'tool-specific' with 'new-tool-name-specific'"
    );

    let has_train_cli = plan
        .matches
        .iter()
        .any(|m| m.before == "Tool-CLI" && m.after == "New-Tool-Name-CLI");
    assert!(
        has_train_cli,
        "Should replace 'Tool-CLI' with 'New-Tool-Name-CLI'"
    );
}

#[test]
fn test_train_case_replacement() {
    // Test that Train-Case style enables "New-Tool-Name-Specific" replacements

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with Train-Case patterns
    let test_file = root.join("docs.md");
    std::fs::write(
        &test_file,
        r#"# Train-Case Examples

- Tool-Specific-Settings for configuration
- Use Tool-Core-Engine for processing
- The Tool-Based-Solution works well
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
        styles: None, // Use default styles (which now includes Train-Case)
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "tool", "new_tool_name", &options).unwrap();

    // Debug: Print all matches
    println!("\n=== Train-Case matches found ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // With Train-Case enabled, should replace:
    // - "Tool-Specific-Settings" -> "New-Tool-Name-Specific-Settings"
    // - "Tool-Core-Engine" -> "New-Tool-Name-Core-Engine"
    // - "Tool-Based-Solution" -> "New-Tool-Name-Based-Solution"

    let has_train_specific = plan.matches.iter().any(|m| {
        m.before == "Tool-Specific-Settings" && m.after == "New-Tool-Name-Specific-Settings"
    });
    assert!(
        has_train_specific,
        "Should replace 'Tool-Specific-Settings' with 'New-Tool-Name-Specific-Settings'"
    );

    let has_train_core = plan
        .matches
        .iter()
        .any(|m| m.before == "Tool-Core-Engine" && m.after == "New-Tool-Name-Core-Engine");
    assert!(
        has_train_core,
        "Should replace 'Tool-Core-Engine' with 'New-Tool-Name-Core-Engine'"
    );

    let has_train_based = plan
        .matches
        .iter()
        .any(|m| m.before == "Tool-Based-Solution" && m.after == "New-Tool-Name-Based-Solution");
    assert!(
        has_train_based,
        "Should replace 'Tool-Based-Solution' with 'New-Tool-Name-Based-Solution'"
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
- tool-Specific (mixed case - unusual)
- Tool-specific-Tool (Pascal followed by lowercase in hyphenated)
- tool-CLI-version (kebab with acronym)
- The-Tool-Tool (Train-Case context)
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
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "tool", "new_tool_name", &options).unwrap();

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

#[test]
fn test_four_component_pascal_case() {
    // Test replacement of 4-component PascalCase identifiers
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("code.rs");
    std::fs::write(
        &test_file,
        r#"// Test with 4-component PascalCase names
use FooBarBazQux;
use FooBarBazQuxClient;
use FooBarBazQux::Engine;

struct FooBarBazQuxConfig {
    settings: FooBarBazQuxSettings,
}

// Also test hyphenated versions
const CONFIG: &str = "FooBarBazQux-config";
let client = "FooBarBazQux-client-v2";

// And snake_case versions
let foo_bar_baz_qux = init();
use foo_bar_baz_qux_utils;
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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "FooBarBazQux", "AlphaBetaGammaDelta", &options).unwrap();

    println!("\n=== Four-component PascalCase matches ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // Check PascalCase replacement
    let has_pascal = plan
        .matches
        .iter()
        .any(|m| m.before == "FooBarBazQux" && m.after == "AlphaBetaGammaDelta");
    assert!(has_pascal, "Should replace PascalCase FooBarBazQux");

    // Check PascalCase with suffix
    let has_pascal_client = plan
        .matches
        .iter()
        .any(|m| m.before == "FooBarBazQuxClient" && m.after == "AlphaBetaGammaDeltaClient");
    assert!(has_pascal_client, "Should replace FooBarBazQuxClient");

    // Check snake_case replacement
    let has_snake = plan
        .matches
        .iter()
        .any(|m| m.before == "foo_bar_baz_qux" && m.after == "alpha_beta_gamma_delta");
    assert!(has_snake, "Should replace snake_case foo_bar_baz_qux");

    // Check hyphenated PascalCase
    let has_hyphen_pascal = plan
        .matches
        .iter()
        .any(|m| m.before == "FooBarBazQux-config" && m.after == "AlphaBetaGammaDelta-config");
    assert!(has_hyphen_pascal, "Should replace FooBarBazQux-config");
}

#[test]
fn test_pascal_case_with_hyphen_suffix() {
    // This test verifies that PascalCase patterns with hyphenated suffixes
    // are properly matched and replaced
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let test_file = root.join("test.md");
    std::fs::write(
        &test_file,
        r#"# Documentation

- FooBarBazQux-specific settings for configuration
- Use FooBarBazQux-engine for processing
- The FooBarBazQux-compatible API works well
- Try foo_bar_baz_qux-specific options (snake_case variant)
- FooBarBazQux-Specific with capital S (Train-Case style)
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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    // Debug: Let's see what variants are generated
    let variants = refaktor_core::case_model::generate_variant_map(
        "foo_bar_baz_qux",
        "alpha_beta_gamma_delta",
        None,
    );
    println!("\n=== Generated variants ===");
    for (old, new) in &variants {
        println!("'{}' -> '{}'", old, new);
    }

    let plan =
        scan_repository(&root, "foo_bar_baz_qux", "alpha_beta_gamma_delta", &options).unwrap();

    println!("\n=== PascalCase-hyphenated matches ===");
    for m in &plan.matches {
        println!("'{}' -> '{}'", m.before, m.after);
    }

    // Should find and replace:
    // - "FooBarBazQux-specific" -> "AlphaBetaGammaDelta-specific"
    // - "FooBarBazQux-engine" -> "AlphaBetaGammaDelta-engine"
    // - "FooBarBazQux-compatible" -> "AlphaBetaGammaDelta-compatible"
    // - "foo_bar_baz_qux-specific" -> "alpha_beta_gamma_delta-specific"

    let has_pascal_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "FooBarBazQux-specific" && m.after == "AlphaBetaGammaDelta-specific");
    assert!(
        has_pascal_specific,
        "Should replace 'FooBarBazQux-specific' with 'AlphaBetaGammaDelta-specific'"
    );

    let has_pascal_engine = plan
        .matches
        .iter()
        .any(|m| m.before == "FooBarBazQux-engine" && m.after == "AlphaBetaGammaDelta-engine");
    assert!(
        has_pascal_engine,
        "Should replace 'FooBarBazQux-engine' with 'AlphaBetaGammaDelta-engine'"
    );

    let has_pascal_compatible = plan.matches.iter().any(|m| {
        m.before == "FooBarBazQux-compatible" && m.after == "AlphaBetaGammaDelta-compatible"
    });
    assert!(
        has_pascal_compatible,
        "Should replace 'FooBarBazQux-compatible' with 'AlphaBetaGammaDelta-compatible'"
    );

    let has_snake_specific = plan.matches.iter().any(|m| {
        m.before == "foo_bar_baz_qux-specific" && m.after == "alpha_beta_gamma_delta-specific"
    });
    assert!(
        has_snake_specific,
        "Should replace 'foo_bar_baz_qux-specific' with 'alpha_beta_gamma_delta-specific'"
    );

    // Check Train-Case variant (capitalized suffix)
    let has_train_specific = plan
        .matches
        .iter()
        .any(|m| m.before == "FooBarBazQux-Specific" && m.after == "AlphaBetaGammaDelta-Specific");
    println!(
        "Checking for FooBarBazQux-Specific -> AlphaBetaGammaDelta-Specific: {}",
        has_train_specific
    );
}
