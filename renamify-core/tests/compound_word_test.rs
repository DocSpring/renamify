use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_compound_pascal_case_replacement() {
    // This test demonstrates the EXPECTED behavior for compound word replacements
    // When replacing "foo_bar" with "foo", compound words like
    // "FooBarArg" should become "FooArg"

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with compound Pascal case identifiers
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r"struct FooBarArg { }
impl From<FooBarArg> for FooBar {
    fn from(arg: FooBarArg) -> FooBar {
        match arg {
            FooBarArg::Table => FooBar::Table,
            FooBarArg::Diff => FooBar::Diff,
        }
    }
}
struct ShouldReplaceFooBarPlease { }
fn getFooBarOption() -> FooBarOption { }",
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
        styles: Some(vec![renamify_core::Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound Pascal Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}'",
            hunk.line, hunk.col, hunk.before, hunk.after
        );
        if let Some(line_after) = &hunk.line_after {
            println!("  After: {line_after}");
        }
    }

    // Should find:
    // Line 1: FooBarArg -> FooArg
    // Line 2: FooBarArg -> FooArg, FooBar -> Foo
    // Line 3: FooBarArg -> FooArg, FooBar -> Foo
    // Line 5: FooBarArg -> FooArg (twice), FooBar -> Foo (twice)
    // Line 6: FooBarArg -> FooArg, FooBar -> Foo
    // Line 9: ShouldReplaceFooBarPlease -> ShouldReplaceFooPlease
    // Line 10: FooBarOption -> FooOption (Pascal only, not getFooBarOption)

    // Total: 11 replacements (Pascal only)
    assert_eq!(
        plan.stats.total_matches, 11,
        "Should find all compound Pascal case variants"
    );

    // Verify FooBarArg is replaced with FooArg
    let foo_bar_arg_replacements: Vec<_> = plan
        .matches
        .iter()
        .filter(|h| h.before == "FooBarArg")
        .collect();

    assert!(
        !foo_bar_arg_replacements.is_empty(),
        "Should find FooBarArg occurrences"
    );

    for hunk in &foo_bar_arg_replacements {
        assert_eq!(
            hunk.after, "FooArg",
            "FooBarArg should be replaced with FooArg"
        );
    }
}

#[test]
fn test_compound_snake_case_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Test with snake_case compounds
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r"let foo_bar_arg = get_foo_bar_arg();
let foo_bar_option = foo_bar_arg.to_option();
match foo_bar_type {
    FooBarType::Json => foo_bar_json(),
}",
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
        styles: Some(vec![
            renamify_core::Style::Snake,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound Snake Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }

    // Should find and replace:
    // foo_bar_arg -> foo_arg
    // foo_bar_option -> foo_option
    // foo_bar_type -> foo_type
    // FooBarType -> FooType
    // foo_bar_json -> foo_json

    let snake_compounds = vec![
        "foo_bar_arg",
        "foo_bar_option",
        "foo_bar_type",
        "foo_bar_json",
    ];
    for compound in &snake_compounds {
        let replacements: Vec<_> = plan
            .matches
            .iter()
            .filter(|h| h.before == *compound)
            .collect();
        assert!(!replacements.is_empty(), "Should find {compound}");

        let expected = compound.replace("foo_bar", "foo");
        for hunk in &replacements {
            assert_eq!(hunk.after, expected, "{compound} should become {expected}");
        }
    }
}

#[test]
fn test_compound_camel_case_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Test with camelCase compounds
    let test_file = root.join("main.js");
    std::fs::write(
        &test_file,
        r"const fooBarArg = getFooBarArg();
const fooBarOption = fooBarArg.toOption();
function setFooBarType(fooBarType) {
    this.fooBarType = fooBarType;
}",
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
        styles: Some(vec![renamify_core::Style::Camel]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound Camel Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}'",
            hunk.line, hunk.col, hunk.before, hunk.after
        );
    }

    // Should find and replace:
    // fooBarArg -> fooArg (2 times on lines 1 and 2)
    // getFooBarArg -> getFooArg (line 1)
    // fooBarOption -> fooOption (line 2)
    // setFooBarType -> setFooType (line 3)
    // fooBarType -> fooType (3 times on lines 3 and 4)

    assert_eq!(
        plan.stats.total_matches, 8,
        "Should find all camelCase compounds"
    );

    // Verify camelCase compounds are properly replaced
    let camel_compounds = vec![
        ("fooBarArg", "fooArg"),
        ("fooBarOption", "fooOption"),
        ("fooBarType", "fooType"),
    ];

    for (from, to) in &camel_compounds {
        let replacements: Vec<_> = plan.matches.iter().filter(|h| h.before == *from).collect();
        assert!(!replacements.is_empty(), "Should find {from}");

        for hunk in &replacements {
            assert_eq!(hunk.after, *to, "{from} should become {to}");
        }
    }
}

#[test]
fn test_compound_pascal_and_camel_case_replacement() {
    // Same test as pascal-only but with both styles enabled
    // This should find MORE matches including getFooBarOption

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Same test file as Pascal test
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r"struct FooBarArg { }
impl From<FooBarArg> for FooBar {
    fn from(arg: FooBarArg) -> FooBar {
        match arg {
            FooBarArg::Table => FooBar::Table,
            FooBarArg::Diff => FooBar::Diff,
        }
    }
}
struct ShouldReplaceFooBarPlease { }
fn getFooBarOption() -> FooBarOption { }",
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
        styles: Some(vec![
            renamify_core::Style::Pascal,
            renamify_core::Style::Camel,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound Pascal + Camel Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}'",
            hunk.line, hunk.col, hunk.before, hunk.after
        );
    }

    // Should find:
    // Line 1: FooBarArg -> FooArg
    // Line 2: FooBarArg -> FooArg, FooBar -> Foo
    // Line 3: FooBarArg -> FooArg, FooBar -> Foo
    // Line 5: FooBarArg -> FooArg (twice), FooBar -> Foo (twice)
    // Line 6: FooBarArg -> FooArg, FooBar -> Foo
    // Line 9: ShouldReplaceFooBarPlease -> ShouldReplaceFooPlease
    // Line 10: getFooBarOption -> getFooOption (ADDITIONAL because Camel is included)
    // Line 10: FooBarOption -> FooOption

    // Total: 12 replacements (one more than Pascal-only)
    assert_eq!(
        plan.stats.total_matches, 12,
        "Should find all compound Pascal AND Camel case variants"
    );

    // Verify we found the camelCase function name
    let camel_match = plan.matches.iter().find(|h| h.before == "getFooBarOption");
    assert!(
        camel_match.is_some(),
        "Should find getFooBarOption when Camel style is included"
    );
}

#[test]
fn test_multiple_compounds_same_line() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Test multiple compound words on the same line
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        "fn convert(foo_bar_arg: FooBarArg) -> FooBarOption { }\n",
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
        styles: Some(vec![
            renamify_core::Style::Snake,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Multiple Compounds Same Line Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Col {}: '{}' -> '{}'", hunk.col, hunk.before, hunk.after);
    }

    // Should find:
    // foo_bar_arg -> foo_arg
    // FooBarArg -> FooArg
    // FooBarOption -> FooOption
    assert_eq!(
        plan.stats.total_matches, 3,
        "Should find all three compound variants on the same line"
    );

    // Verify all are on line 1 but different columns
    for hunk in &plan.matches {
        assert_eq!(hunk.line, 1);
    }
}

#[test]
fn test_compound_case_preservation_bug() {
    // This test verifies that compound replacements preserve the original case style
    // Bug: FooBarOption -> bazaarQuxicleOption (wrong!)
    // Should be: FooBarOption -> BazaarQuxicleOption (correct)

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with PascalCase compound identifier
    let test_file = root.join("main.rs");
    std::fs::write(&test_file, r"fn getFooBarOption() -> FooBarOption { }").unwrap();

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
        styles: None, // Use all default styles
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "bazaar_quxicle", &options).unwrap();

    println!("\n=== Case Preservation Test ===");
    for hunk in &plan.matches {
        println!("'{}' -> '{}'", hunk.before, hunk.after);
    }

    // Find the FooBarOption replacement
    let pascal_option = plan.matches.iter().find(|h| h.before == "FooBarOption");

    assert!(pascal_option.is_some(), "Should find FooBarOption");

    let hunk = pascal_option.unwrap();
    assert_eq!(
        hunk.after, "BazaarQuxicleOption",
        "FooBarOption should become BazaarQuxicleOption (PascalCase preserved), not bazaarQuxicleOption"
    );

    // Also check the camelCase function name is handled correctly
    let camel_func = plan.matches.iter().find(|h| h.before == "getFooBarOption");

    assert!(camel_func.is_some(), "Should find getFooBarOption");

    let func_hunk = camel_func.unwrap();
    assert_eq!(
        func_hunk.after, "getBazaarQuxicleOption",
        "getFooBarOption should become getBazaarQuxicleOption (camelCase preserved)"
    );
}
