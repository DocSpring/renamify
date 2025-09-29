use renamify_core::{
    coercion::{apply_coercion, detect_style, Style},
    scan_repository,
    scanner::{CoercionMode, PlanOptions, RenameKind},
    Style as VariantStyle,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_coercion_oldtool_core_to_newtool_core() {
    // This is the critical test case from the user's feedback
    let result = apply_coercion("oldtool-core", "oldtool", "newtool");

    assert!(result.is_some());
    let (coerced, reason) = result.unwrap();
    assert_eq!(coerced, "newtool-core");
    assert!(reason.contains("Kebab"));
}

#[test]
fn test_coercion_various_container_styles() {
    // Test kebab-case container
    let result = apply_coercion("oldtool-lib", "oldtool", "newtool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "newtool-lib");

    // Test snake_case container
    let result = apply_coercion("oldtool_core", "oldtool", "newtool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "newtool_core");

    // Test PascalCase container
    let result = apply_coercion("OldtoolCore", "Oldtool", "Newtool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "NewtoolCore");

    // Test camelCase container
    let result = apply_coercion("oldtoolCore", "oldtool", "newtool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "newtoolCore");

    // Test SCREAMING_SNAKE_CASE container
    let result = apply_coercion("OLDTOOL_CORE", "OLDTOOL", "NEWTOOL");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "NEWTOOL_CORE");

    // Test dot.case container (should only work when enabled)
    let result = apply_coercion("oldtool.core", "oldtool", "newtool");
    // For now this should be None since dot-case is risky
    assert!(result.is_none());
}

#[test]
fn test_coercion_partial_matches() {
    // Test when old pattern is part of a larger identifier
    let result = apply_coercion("my-oldtool-lib", "oldtool", "newtool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "my-newtool-lib");

    // Test with multiple occurrences
    let result = apply_coercion("oldtool-to-oldtool", "oldtool", "tool");
    assert!(result.is_some());
    let (coerced, _) = result.unwrap();
    assert_eq!(coerced, "tool-to-tool");
}

#[test]
fn test_coercion_no_container_style() {
    // Test when there's no clear container style (should not coerce)
    let result = apply_coercion("oldtool", "oldtool", "newtool");
    assert!(result.is_none());

    // Test mixed style containers (should not coerce)
    let result = apply_coercion("oldtool-core_lib", "oldtool", "newtool");
    assert!(result.is_none());
}

#[test]
fn test_pascal_segment_inside_camel_container() {
    // Regression: ensure DeployRequests segment keeps Pascal casing inside camel container
    let result = apply_coercion(
        "getAdminDeployRequestsParams.ts",
        "DeployRequests",
        "DeployApprovalRequests",
    );
    assert!(result.is_some());
    let (coerced, reason) = result.unwrap();
    assert_eq!(coerced, "getAdminDeployApprovalRequestsParams.ts");
    assert_eq!(reason, "coerced to Pascal style");

    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("getAdminDeployRequestsParams.ts"),
        "export type Test = string;",
    )
    .unwrap();

    let mut options = PlanOptions {
        plan_out: temp_dir.path().join("plan.json"),
        respect_gitignore: false,
        ..Default::default()
    };
    options.coerce_separators = CoercionMode::Auto;

    let plan = scan_repository(
        temp_dir.path(),
        "deploy_requests",
        "deploy_approval_requests",
        &options,
    )
    .unwrap();

    let rename = plan
        .paths
        .iter()
        .find(|r| {
            r.path
                .file_name()
                .map(|f| f == "getAdminDeployRequestsParams.ts")
                .unwrap_or(false)
        })
        .expect("expected rename for DeployRequests params file");

    assert_eq!(
        rename.new_path.file_name().unwrap().to_str().unwrap(),
        "getAdminDeployApprovalRequestsParams.ts"
    );
    assert_eq!(
        rename.coercion_applied.as_deref(),
        Some("coerced to Pascal style")
    );
}

#[test]
fn test_pascal_singular_content_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("api.ts");

    fs::write(
        &file_path,
        "export const listDeployRequests = (): Promise<DeployRequestList> => unwrap(gateway.getAdminDeployRequests(params));\nexport const approveDeployRequest = (): Promise<DeployRequest> => unwrap(gateway.postAdminDeployRequestsIdApprove(id, payload ?? {}));\n",
    )
    .unwrap();

    let mut options = PlanOptions {
        plan_out: temp_dir.path().join("plan.json"),
        respect_gitignore: false,
        ..Default::default()
    };
    options.coerce_separators = CoercionMode::Auto;

    let plan = scan_repository(
        temp_dir.path(),
        "deploy_requests",
        "deploy_approval_requests",
        &options,
    )
    .unwrap();

    assert!(
        plan.matches.iter().any(|m| {
            m.file.ends_with("api.ts")
                && m.line_after
                    .as_deref()
                    .is_some_and(|line| line.contains("Promise<DeployApprovalRequestList>"))
        }),
        "expected Promise<DeployApprovalRequestList> replacement"
    );

    assert!(
        plan.matches.iter().any(|m| {
            m.file.ends_with("api.ts")
                && m.line_after
                    .as_deref()
                    .is_some_and(|line| line.contains("Promise<DeployApprovalRequest>"))
        }),
        "expected Promise<DeployApprovalRequest> replacement"
    );
}

#[test]
fn test_title_case_replacement_preserves_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("docs.md");

    fs::write(&file_path, "**Deploy Requests**\n").unwrap();

    let mut options = PlanOptions {
        plan_out: temp_dir.path().join("plan.json"),
        respect_gitignore: false,
        ..Default::default()
    };
    options.coerce_separators = CoercionMode::Auto;

    let mut styles = VariantStyle::default_styles();
    if !styles.contains(&VariantStyle::Title) {
        styles.push(VariantStyle::Title);
    }

    options.styles = Some(styles);

    let plan = scan_repository(
        temp_dir.path(),
        "deploy_requests",
        "deploy_approval_requests",
        &options,
    )
    .unwrap();

    let title_match = plan
        .matches
        .iter()
        .find(|m| m.file.ends_with("docs.md"))
        .expect("expected title case replacement");

    assert_eq!(title_match.variant, "Deploy Requests");
    assert_eq!(title_match.replace, "Deploy Approval Requests");
    assert!(title_match
        .line_after
        .as_deref()
        .is_some_and(|line| line.contains("**Deploy Approval Requests**")));
    assert!(title_match.coercion_applied.is_none());
}

#[test]
fn test_end_to_end_coercion_with_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files with various naming patterns
    fs::write(
        temp_dir.path().join("oldtool-core.rs"),
        "use oldtool_lib::OldtoolEngine;\nfn oldtool_main() {}",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("oldtool_utils.py"),
        "def oldtool_helper(): pass",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("OldtoolService.java"),
        "class OldtoolService {}",
    )
    .unwrap();

    // Create directories
    fs::create_dir(temp_dir.path().join("oldtool-plugins")).unwrap();
    fs::create_dir(temp_dir.path().join("oldtool_tests")).unwrap();

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
        styles: None,
        rename_files: true,
        rename_dirs: true,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto, // Enable coercion
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

    // Check file renames are coerced properly
    let file_renames: Vec<_> = plan
        .paths
        .iter()
        .filter(|r| r.kind == RenameKind::File)
        .collect();

    // oldtool-core.rs should become newtool-core.rs (kebab style)
    assert!(
        file_renames
            .iter()
            .any(|r| r.path.file_name().unwrap() == "oldtool-core.rs"
                && r.new_path.file_name().unwrap() == "newtool-core.rs"),
        "kebab-case file should be coerced to kebab-case"
    );

    // oldtool_utils.py should become newtool_utils.py (snake style)
    assert!(
        file_renames
            .iter()
            .any(|r| r.path.file_name().unwrap() == "oldtool_utils.py"
                && r.new_path.file_name().unwrap() == "newtool_utils.py"),
        "snake_case file should be coerced to snake_case"
    );

    // OldtoolService.java should become NewtoolService.java (pascal style)
    assert!(
        file_renames
            .iter()
            .any(|r| r.path.file_name().unwrap() == "OldtoolService.java"
                && r.new_path.file_name().unwrap() == "NewtoolService.java"),
        "PascalCase file should be coerced to PascalCase"
    );

    // Check directory renames are coerced properly
    let dir_renames: Vec<_> = plan
        .paths
        .iter()
        .filter(|r| r.kind == RenameKind::Dir)
        .collect();

    // oldtool-plugins should become newtool-plugins
    assert!(
        dir_renames
            .iter()
            .any(|r| r.path.file_name().unwrap() == "oldtool-plugins"
                && r.new_path.file_name().unwrap() == "newtool-plugins"),
        "kebab-case directory should be coerced to kebab-case"
    );

    // oldtool_tests should become newtool_tests
    assert!(
        dir_renames
            .iter()
            .any(|r| r.path.file_name().unwrap() == "oldtool_tests"
                && r.new_path.file_name().unwrap() == "newtool_tests"),
        "snake_case directory should be coerced to snake_case"
    );

    // Check that coercion_applied field is set for coerced renames
    let coerced_renames = plan
        .paths
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

    fs::write(temp_dir.path().join("oldtool-core.rs"), "test").unwrap();

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
        styles: None,
        rename_files: true,
        rename_dirs: true,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Off, // Disable coercion
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

    // Without coercion, should get newtool-core.rs (mixed style)
    let file_renames: Vec<_> = plan
        .paths
        .iter()
        .filter(|r| r.kind == RenameKind::File)
        .collect();

    assert!(
        file_renames.iter().any(
            |r| r.path.file_name().unwrap() == "oldtool-core.rs"
                && r.new_path.file_name().unwrap() == "newtool-core.rs" // Mixed style without coercion
        ),
        "Without coercion should produce mixed style"
    );

    // No coercion_applied should be set
    let coerced_renames = plan
        .paths
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
use oldtool_core::OldtoolEngine;
use my_oldtool_lib::utils;
let oldtool-service = OldtoolService::new();
let config = oldtool.config.load();
",
    )
    .unwrap();

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
        styles: None,
        rename_files: false, // Focus on content matches only
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

    // Check that content matches are coerced based on their container context
    let content_matches = &plan.matches;
    assert!(!content_matches.is_empty());

    // Debug: Print all matches to understand what's happening
    println!("\n=== Debug: All matches found ===");
    for (i, m) in content_matches.iter().enumerate() {
        println!(
            "{}: '{}' -> '{}' (coercion: {:?})",
            i, m.content, m.replace, m.coercion_applied
        );
    }

    // Find matches that should have been coerced
    let coerced_matches = content_matches
        .iter()
        .filter(|m| m.coercion_applied.is_some())
        .count();

    // We expect some matches to be coerced based on their context
    // (like oldtool_core should use snake_case for the replacement)
    assert!(
        coerced_matches > 0,
        "Some content matches should have coercion applied"
    );

    // Check specific coercions
    let snake_case_match = content_matches
        .iter()
        .find(|m| m.content.contains("oldtool_core") && m.coercion_applied.is_some());
    if let Some(m) = snake_case_match {
        assert!(
            m.replace.contains("newtool_core"),
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
use oldtool_core::Engine;
let oldtool-utils = OldtoolService::new();
const OLDTOOL_CONFIG = OldtoolKey<T>::new();
let url = "https://github.com/user/oldtool-project";
let path = "src/oldtool/main.rs";
let namespace = oldtool::core::apply();
let env_var = process.env.OLDTOOL_DEBUG;
let css_class = ".oldtool-button:hover";
let db_column = user_oldtool_settings_id;
let config_key = app.oldtool.enabled;
let package = "@scope/oldtool-utils";

// Mixed contexts that might skip coercion but still do replacement
let mixed = oldtool_someCAMEL-case;
let ambiguous = x.oldtool.y;
let complex_generic = HashMap<OldtoolKey<T>, Vec<OldtoolValue>>;

// String literals and comments (should still be replaced)
println!("Please use oldtool for this task");
// The oldtool tool is great
let docs = "oldtool: smart search and replace";

// File extensions and versioning
let binary = "oldtool-v1.2.3-beta.tar.gz";
let regex_pattern = r"oldtool[_-](\w+)";
"#,
    )
    .unwrap();

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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

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
        .any(|m| m.replace.contains("newtool_") || m.replace.contains("_newtool"));
    assert!(has_underscores, "Should have snake_case replacements");

    let has_hyphens = content_matches
        .iter()
        .any(|m| m.replace.contains("newtool-") || m.replace.contains("-newtool"));
    assert!(has_hyphens, "Should have kebab-case replacements");
}

#[test]
fn test_namespace_separator_prevents_coercion() {
    let temp_dir = TempDir::new().unwrap();

    // Test that namespace separators prevent coercion - each identifier is treated independently
    fs::write(
        temp_dir.path().join("paths.rs"),
        r#"
use oldtool::core::Engine;
use oldtool::utils::helper;
let path1 = "src/oldtool/main.rs";
let path2 = "./oldtool/config.toml";
let path3 = "/usr/bin/oldtool";
let url = "https://github.com/user/oldtool";
let module = oldtool::scanner::scan();
let nested = oldtool::core::pattern::Match;
"#,
    )
    .unwrap();

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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

    let content_matches = &plan.matches;

    // All "oldtool" matches should be replaced with "newtool" (lowercase)
    // regardless of context, because separators prevent coercion
    for m in content_matches {
        if m.content == "oldtool" {
            assert_eq!(
                m.replace, "newtool",
                "Should be 'newtool' not coerced after separator"
            );
            // Coercion should NOT be applied when after a separator
            assert!(
                m.coercion_applied.is_none(),
                "Should not have coercion applied after separator"
            );
        }
    }
}

#[test]
fn test_mixed_style_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Test cases where coercion might be skipped due to mixed styles
    fs::write(
        temp_dir.path().join("mixed.rs"),
        r"
// These have mixed styles in the same identifier - coercion might be skipped
let weird1 = oldtool_someCAMEL-case;
let weird2 = oldtool-some_MIXED_Case;
let weird3 = oldtool.some-weird_MIX;

// These are on mixed-style lines but individual contexts should still work
let snake_case_var = oldtool_core; let camelVar = oldtoolService;
",
    )
    .unwrap();

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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

    // All matches should still do replacement, even if coercion is skipped
    assert!(!plan.matches.is_empty());

    // Every match should have the replacement in an appropriate style
    for m in &plan.matches {
        // The replacement should contain the new pattern in some form
        let after_lower = m.replace.to_lowercase();
        assert!(
            after_lower.contains("newtool"),
            "Expected replacement to contain new pattern, got: {}",
            m.replace
        );
    }
}

#[test]
fn test_language_specific_defaults() {
    // Test Rust file defaults (should prefer snake_case for modules)
    let _result = apply_coercion("oldtool.rs", "oldtool", "newtool");
    // For now this should be None since we need to implement language-specific logic
    // When implemented, this should prefer snake_case

    // Test JavaScript/TypeScript defaults (should prefer kebab-case)
    let _result = apply_coercion("oldtool.js", "oldtool", "newtool");
    // When implemented, should prefer kebab-case

    // Test Python defaults (should prefer snake_case)
    let _result = apply_coercion("oldtool.py", "oldtool", "newtool");
    // When implemented, should prefer snake_case

    // Test Java defaults (should prefer PascalCase for classes)
    let _result = apply_coercion("Oldtool.java", "Oldtool", "Newtool");
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
name = "oldtool-core"
version = "0.1.0"

[dependencies]
oldtool = { path = "../oldtool" }
"#,
    )
    .unwrap();

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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: temp_dir.path().join("plan.json"),
        coerce_separators: CoercionMode::Auto,
        enable_plural_variants: true,
    };

    let plan = scan_repository(temp_dir.path(), "oldtool", "newtool", &options).unwrap();

    // In Cargo.toml, crate names should use hyphens
    let toml_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.file_name().unwrap() == "Cargo.toml")
        .collect();

    assert!(!toml_matches.is_empty());

    // The "oldtool-core" name should become "newtool-core"
    let name_match = toml_matches
        .iter()
        .find(|m| m.content.contains("oldtool-core"));
    if let Some(m) = name_match {
        assert!(
            m.replace.contains("newtool-core"),
            "Cargo.toml crate names should use hyphen style"
        );
    }
}

#[test]
fn test_mixed_separators_no_coercion() {
    // Test files/identifiers with mixed separators (should not be coerced)
    let result = apply_coercion("oldtool-core_lib.rs", "oldtool", "newtool");
    assert!(
        result.is_none(),
        "Mixed separator containers should not be coerced"
    );

    let result = apply_coercion("oldtool_core-service", "oldtool", "newtool");
    assert!(
        result.is_none(),
        "Mixed separator containers should not be coerced"
    );
}

#[test]
fn test_style_memory_consistency() {
    // This test is for future functionality where we remember style choices
    // and apply them consistently across the same basename

    // When we rename "oldtool.rs" -> "newtool.rs" (snake_case)
    // Then other references to "oldtool.rs" should also use snake_case style

    // For now, just test that the basic style detection is consistent
    assert_eq!(detect_style("oldtool_core.rs"), Style::Snake);
    assert_eq!(detect_style("oldtool-core.js"), Style::Kebab);
    assert_eq!(detect_style("OldtoolCore.java"), Style::Pascal);
    assert_eq!(detect_style("oldtoolCore.ts"), Style::Camel);
}
