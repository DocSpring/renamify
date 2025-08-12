use refaktor_core::{
    coercion::{apply_coercion, detect_style, Style},
    scan_repository,
    scanner::{CoercionMode, PlanOptions, RenameKind},
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_coercion_refaktor_core_to_renamed_refactoring_tool_core() {
    // This is the critical test case from the user's feedback
    let result = apply_coercion("refaktor-core", "refaktor", "renamed_refactoring_tool");

    assert!(result.is_some());
    let (coerced, reason) = result.unwrap();
    assert_eq!(coerced, "renamed-refactoring-tool-core");
    assert!(reason.contains("Kebab"));
}

#[test]
fn test_coercion_various_container_styles() {
    // Test kebab-case container
    let result = apply_coercion("refaktor-lib", "refaktor", "renamed_refactoring_tool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "renamed-refactoring-tool-lib");

    // Test snake_case container
    let result = apply_coercion("refaktor_core", "refaktor", "renamed_refactoring_tool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "renamed_refactoring_tool_core");

    // Test PascalCase container
    let result = apply_coercion("RefaktorCore", "Refaktor", "RenamedRefactoringTool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "RenamedRefactoringToolCore");

    // Test camelCase container
    let result = apply_coercion("refaktorCore", "refaktor", "renamedRefactoringTool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "renamedRefactoringToolCore");

    // Test SCREAMING_SNAKE_CASE container
    let result = apply_coercion("REFAKTOR_CORE", "REFAKTOR", "RENAMED_REFACTORING_TOOL");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "RENAMED_REFACTORING_TOOL_CORE");

    // Test dot.case container (should only work when enabled)
    let result = apply_coercion("refaktor.core", "refaktor", "renamed_refactoring_tool");
    // For now this should be None since dot-case is risky
    assert!(result.is_none());
}

#[test]
fn test_coercion_partial_matches() {
    // Test when old pattern is part of a larger identifier
    let result = apply_coercion("my-refaktor-lib", "refaktor", "renamed_refactoring_tool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "my-renamed-refactoring-tool-lib");

    // Test with multiple occurrences
    let result = apply_coercion("refaktor-to-refaktor", "refaktor", "tool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "tool-to-tool");
}

#[test]
fn test_coercion_no_container_style() {
    // Test when there's no clear container style (should not coerce)
    let result = apply_coercion("refaktor", "refaktor", "renamed_refactoring_tool");
    assert!(result.is_none());

    // Test mixed style containers (should not coerce)
    let result = apply_coercion("refaktor-core_lib", "refaktor", "renamed_refactoring_tool");
    assert!(result.is_none());
}

#[test]
fn test_end_to_end_coercion_with_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files with various naming patterns
    fs::write(
        temp_dir.path().join("refaktor-core.rs"),
        "use refaktor_lib::RefaktorEngine;\nfn refaktor_main() {}",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("refaktor_utils.py"),
        "def refaktor_helper(): pass",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("RefaktorService.java"),
        "class RefaktorService {}",
    )
    .unwrap();

    // Create directories
    fs::create_dir(temp_dir.path().join("refaktor-plugins")).unwrap();
    fs::create_dir(temp_dir.path().join("refaktor_tests")).unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: true,
        rename_dirs: true,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto, // Enable coercion
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    // Check file renames are coerced properly
    let file_renames: Vec<_> = plan
        .renames
        .iter()
        .filter(|r| r.kind == RenameKind::File)
        .collect();

    // refaktor-core.rs should become renamed-refactoring-tool-core.rs (kebab style)
    assert!(
        file_renames
            .iter()
            .any(|r| r.from.file_name().unwrap() == "refaktor-core.rs"
                && r.to.file_name().unwrap() == "renamed-refactoring-tool-core.rs"),
        "kebab-case file should be coerced to kebab-case"
    );

    // refaktor_utils.py should become renamed_refactoring_tool_utils.py (snake style)
    assert!(
        file_renames
            .iter()
            .any(|r| r.from.file_name().unwrap() == "refaktor_utils.py"
                && r.to.file_name().unwrap() == "renamed_refactoring_tool_utils.py"),
        "snake_case file should be coerced to snake_case"
    );

    // RefaktorService.java should become RenamedRefactoringToolService.java (pascal style)
    assert!(
        file_renames
            .iter()
            .any(|r| r.from.file_name().unwrap() == "RefaktorService.java"
                && r.to.file_name().unwrap() == "RenamedRefactoringToolService.java"),
        "PascalCase file should be coerced to PascalCase"
    );

    // Check directory renames are coerced properly
    let dir_renames: Vec<_> = plan
        .renames
        .iter()
        .filter(|r| r.kind == RenameKind::Dir)
        .collect();

    // refaktor-plugins should become renamed-refactoring-tool-plugins
    assert!(
        dir_renames
            .iter()
            .any(|r| r.from.file_name().unwrap() == "refaktor-plugins"
                && r.to.file_name().unwrap() == "renamed-refactoring-tool-plugins"),
        "kebab-case directory should be coerced to kebab-case"
    );

    // refaktor_tests should become renamed_refactoring_tool_tests
    assert!(
        dir_renames
            .iter()
            .any(|r| r.from.file_name().unwrap() == "refaktor_tests"
                && r.to.file_name().unwrap() == "renamed_refactoring_tool_tests"),
        "snake_case directory should be coerced to snake_case"
    );

    // Check that coercion_applied field is set for coerced renames
    let coerced_renames = plan
        .renames
        .iter()
        .filter(|r| r.coercion_applied.is_some())
        .count();
    assert!(
        coerced_renames > 0,
        "Some renames should have coercion applied"
    );
}

#[test]
fn test_coercion_disabled() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("refaktor-core.rs"), "test").unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: true,
        rename_dirs: true,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Off, // Disable coercion
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    // Without coercion, should get renamed_refactoring_tool-core.rs (mixed style)
    let file_renames: Vec<_> = plan
        .renames
        .iter()
        .filter(|r| r.kind == RenameKind::File)
        .collect();

    assert!(
        file_renames.iter().any(
            |r| r.from.file_name().unwrap() == "refaktor-core.rs"
                && r.to.file_name().unwrap() == "renamed_refactoring_tool-core.rs" // Mixed style without coercion
        ),
        "Without coercion should produce mixed style"
    );

    // No coercion_applied should be set
    let coerced_renames = plan
        .renames
        .iter()
        .filter(|r| r.coercion_applied.is_some())
        .count();
    assert_eq!(
        coerced_renames, 0,
        "No coercion should be applied when disabled"
    );
}

#[test]
fn test_coercion_in_content_matches() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file with various identifiers that should be coerced
    fs::write(
        temp_dir.path().join("code.rs"),
        r"
use refaktor_core::RefaktorEngine;
use my_refaktor_lib::utils;  
let refaktor-service = RefaktorService::new();
let config = refaktor.config.load();
",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false, // Focus on content matches only
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    // Check that content matches are coerced based on their container context
    let content_matches = &plan.matches;
    assert!(!content_matches.is_empty());

    // Debug: Print all matches to understand what's happening
    println!("\n=== Debug: All matches found ===");
    for (i, m) in content_matches.iter().enumerate() {
        println!(
            "{}: '{}' -> '{}' (coercion: {:?})",
            i, m.before, m.after, m.coercion_applied
        );
    }

    // Find matches that should have been coerced
    let coerced_matches = content_matches
        .iter()
        .filter(|m| m.coercion_applied.is_some())
        .count();

    // We expect some matches to be coerced based on their context
    // (like refaktor_core should use snake_case for the replacement)
    assert!(
        coerced_matches > 0,
        "Some content matches should have coercion applied"
    );

    // Check specific coercions
    let snake_case_match = content_matches
        .iter()
        .find(|m| m.before.contains("refaktor_core") && m.coercion_applied.is_some());
    if let Some(m) = snake_case_match {
        assert!(
            m.after.contains("renamed_refactoring_tool_core"),
            "snake_case context should produce snake_case replacement"
        );
    }
}

#[test]
fn test_comprehensive_coercion_edge_cases() {
    let temp_dir = TempDir::new().unwrap();

    // Create a file with comprehensive edge cases for coercion
    fs::write(
        temp_dir.path().join("edge_cases.rs"),
        r#"
// Clean contexts that SHOULD get coercion
use refaktor_core::Engine;
let refaktor-utils = RefaktorService::new();
const REFAKTOR_CONFIG = RefaktorKey<T>::new();
let url = "https://github.com/user/refaktor-project";
let path = "src/refaktor/main.rs";
let namespace = refaktor::core::apply();
let env_var = process.env.REFAKTOR_DEBUG;
let css_class = ".refaktor-button:hover";
let db_column = user_refaktor_settings_id;
let config_key = app.refaktor.enabled;
let package = "@scope/refaktor-utils";

// Mixed contexts that might skip coercion but still do replacement
let mixed = refaktor_someCAMEL-case;
let ambiguous = x.refaktor.y;
let complex_generic = HashMap<RefaktorKey<T>, Vec<RefaktorValue>>;

// String literals and comments (should still be replaced)
println!("Please use refaktor for this task");
// The refaktor tool is great
let docs = "refaktor: smart search and replace";

// File extensions and versioning
let binary = "refaktor-v1.2.3-beta.tar.gz";
let regex_pattern = r"refaktor[_-](\w+)";
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    let content_matches = &plan.matches;
    assert!(!content_matches.is_empty(), "Should find many matches");

    // Count matches with different coercion outcomes
    let coerced_matches: Vec<_> = content_matches
        .iter()
        .filter(|m| m.coercion_applied.is_some())
        .collect();

    // let _uncoerced_matches: Vec<_> = content_matches.iter()
    //     .filter(|m| m.coercion_applied.is_none())
    //     .collect();

    // We should have several coerced matches
    assert!(
        coerced_matches.len() >= 5,
        "Should have multiple coerced matches"
    );

    // Check specific coercion patterns - the coercion applies to the replacement style
    let snake_case_matches = content_matches
        .iter()
        .filter(|m| {
            m.coercion_applied
                .as_ref()
                .is_some_and(|c| c.contains("Snake"))
        })
        .count();
    assert!(
        snake_case_matches >= 1,
        "Should have snake_case coercion applied"
    );

    let kebab_case_matches = content_matches
        .iter()
        .filter(|m| {
            m.coercion_applied
                .as_ref()
                .is_some_and(|c| c.contains("Kebab"))
        })
        .count();
    assert!(
        kebab_case_matches >= 1,
        "Should have kebab-case coercion applied"
    );

    // Check that coerced matches use the right separators
    let has_underscores = content_matches
        .iter()
        .any(|m| m.after.contains("renamed_refactoring_tool"));
    assert!(has_underscores, "Should have snake_case replacements");

    let has_hyphens = content_matches
        .iter()
        .any(|m| m.after.contains("renamed-refactoring-tool"));
    assert!(has_hyphens, "Should have kebab-case replacements");
}

#[test]
fn test_path_and_namespace_coercion() {
    let temp_dir = TempDir::new().unwrap();

    // Test various path and namespace patterns
    fs::write(
        temp_dir.path().join("paths.rs"),
        r#"
use refaktor::core::Engine;
use refaktor::utils::helper;
let path1 = "src/refaktor/main.rs";
let path2 = "./refaktor/config.toml";  
let path3 = "/usr/bin/refaktor";
let url = "https://github.com/user/refaktor";
let module = refaktor::scanner::scan();
let nested = refaktor::core::pattern::Match;
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    let content_matches = &plan.matches;

    // Check that we have some coerced matches for path-like contexts
    let coerced_count = content_matches
        .iter()
        .filter(|m| m.coercion_applied.is_some())
        .count();
    assert!(
        coerced_count > 0,
        "Should have coerced some path/namespace contexts"
    );
}

#[test]
fn test_mixed_style_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Test cases where coercion might be skipped due to mixed styles
    fs::write(
        temp_dir.path().join("mixed.rs"),
        r"
// These have mixed styles in the same identifier - coercion might be skipped
let weird1 = refaktor_someCAMEL-case;
let weird2 = refaktor-some_MIXED_Case;
let weird3 = refaktor.some-weird_MIX;

// These are on mixed-style lines but individual contexts should still work
let snake_case_var = refaktor_core; let camelVar = refaktorService;
",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    // All matches should still do replacement, even if coercion is skipped
    assert!(!plan.matches.is_empty());

    // Every match should have the replacement in an appropriate style
    for m in &plan.matches {
        // The replacement should contain the new pattern in some form
        let after_lower = m.after.to_lowercase();
        assert!(
            after_lower.contains("renamedrefactoringtool")
                || after_lower.contains("renamed_refactoring_tool")
                || after_lower.contains("renamed-refactoring-tool"),
            "Expected replacement to contain new pattern, got: {}",
            m.after
        );
    }
}

#[test]
fn test_language_specific_defaults() {
    // Test Rust file defaults (should prefer snake_case for modules)
    let _result = apply_coercion("refaktor.rs", "refaktor", "renamed_refactoring_tool");
    // For now this should be None since we need to implement language-specific logic
    // When implemented, this should prefer snake_case

    // Test JavaScript/TypeScript defaults (should prefer kebab-case)
    let _result = apply_coercion("refaktor.js", "refaktor", "renamed_refactoring_tool");
    // When implemented, should prefer kebab-case

    // Test Python defaults (should prefer snake_case)
    let _result = apply_coercion("refaktor.py", "refaktor", "renamed_refactoring_tool");
    // When implemented, should prefer snake_case

    // Test Java defaults (should prefer PascalCase for classes)
    let _result = apply_coercion("Refaktor.java", "Refaktor", "RenamedRefactoringTool");
    // When implemented, should prefer PascalCase
}

#[test]
fn test_cargo_toml_crate_name_coercion() {
    let temp_dir = TempDir::new().unwrap();

    // Create Cargo.toml with hyphenated crate name
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "refaktor-core"
version = "0.1.0"

[dependencies]
refaktor = { path = "../refaktor" }
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
    };

    let mut plan = scan_repository(
        temp_dir.path(),
        "refaktor",
        "renamed_refactoring_tool",
        &options,
    )
    .unwrap();

    // In Cargo.toml, crate names should use hyphens
    let toml_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.file_name().unwrap() == "Cargo.toml")
        .collect();

    assert!(!toml_matches.is_empty());

    // The "refaktor-core" name should become "renamed-refactoring-tool-core"
    let name_match = toml_matches
        .iter()
        .find(|m| m.before.contains("refaktor-core"));
    if let Some(m) = name_match {
        assert!(
            m.after.contains("renamed-refactoring-tool-core"),
            "Cargo.toml crate names should use hyphen style"
        );
    }
}

#[test]
fn test_mixed_separators_no_coercion() {
    // Test files/identifiers with mixed separators (should not be coerced)
    let result = apply_coercion(
        "refaktor-core_lib.rs",
        "refaktor",
        "renamed_refactoring_tool",
    );
    assert!(
        result.is_none(),
        "Mixed separator containers should not be coerced"
    );

    let result = apply_coercion(
        "refaktor_core-service",
        "refaktor",
        "renamed_refactoring_tool",
    );
    assert!(
        result.is_none(),
        "Mixed separator containers should not be coerced"
    );
}

#[test]
fn test_style_memory_consistency() {
    // This test is for future functionality where we remember style choices
    // and apply them consistently across the same basename

    // When we rename "refaktor.rs" -> "renamed_refactoring_tool.rs" (snake_case)
    // Then other references to "refaktor.rs" should also use snake_case style

    // For now, just test that the basic style detection is consistent
    assert_eq!(detect_style("refaktor_core.rs"), Style::Snake);
    assert_eq!(detect_style("refaktor-core.js"), Style::Kebab);
    assert_eq!(detect_style("RefaktorCore.java"), Style::Pascal);
    assert_eq!(detect_style("refaktorCore.ts"), Style::Camel);
}
