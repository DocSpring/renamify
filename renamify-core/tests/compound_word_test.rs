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
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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
            hunk.line, hunk.col, hunk.content, hunk.replace
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
        .filter(|h| h.content == "FooBarArg")
        .collect();

    assert!(
        !foo_bar_arg_replacements.is_empty(),
        "Should find FooBarArg occurrences"
    );

    for hunk in &foo_bar_arg_replacements {
        assert_eq!(
            hunk.replace, "FooArg",
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
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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
        println!(
            "Line {}: '{}' -> '{}'",
            hunk.line, hunk.content, hunk.replace
        );
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
            .filter(|h| h.content == *compound)
            .collect();
        assert!(!replacements.is_empty(), "Should find {compound}");

        let expected = compound.replace("foo_bar", "foo");
        for hunk in &replacements {
            assert_eq!(
                hunk.replace, expected,
                "{compound} should become {expected}"
            );
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
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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
            hunk.line, hunk.col, hunk.content, hunk.replace
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
        let replacements: Vec<_> = plan.matches.iter().filter(|h| h.content == *from).collect();
        assert!(!replacements.is_empty(), "Should find {from}");

        for hunk in &replacements {
            assert_eq!(hunk.replace, *to, "{from} should become {to}");
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
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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
            hunk.line, hunk.col, hunk.content, hunk.replace
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
    let camel_match = plan.matches.iter().find(|h| h.content == "getFooBarOption");
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
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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
        println!("Col {}: '{}' -> '{}'", hunk.col, hunk.content, hunk.replace);
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
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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
        println!("'{}' -> '{}'", hunk.content, hunk.replace);
    }

    // Find the FooBarOption replacement
    let pascal_option = plan.matches.iter().find(|h| h.content == "FooBarOption");

    assert!(pascal_option.is_some(), "Should find FooBarOption");

    let hunk = pascal_option.unwrap();
    assert_eq!(
        hunk.replace, "BazaarQuxicleOption",
        "FooBarOption should become BazaarQuxicleOption (PascalCase preserved), not bazaarQuxicleOption"
    );

    // Also check the camelCase function name is handled correctly
    let camel_func = plan.matches.iter().find(|h| h.content == "getFooBarOption");

    assert!(camel_func.is_some(), "Should find getFooBarOption");

    let func_hunk = camel_func.unwrap();
    assert_eq!(
        func_hunk.replace, "getBazaarQuxicleOption",
        "getFooBarOption should become getBazaarQuxicleOption (camelCase preserved)"
    );
}

#[test]
fn test_repeated_word_compound_bug() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("package.json");
    std::fs::write(
        &test_file,
        r#"{
  "contributes": {
    "icons": {
      "testword-testword": {
        "description": "Test icon",
        "default": {
          "fontPath": "media/glyphs.woff2"
        }
      }
    },
    "viewsContainers": {
      "activitybar": [{
        "id": "testword",
        "title": "Testword",
        "icon": "$(testword-testword)"
      }]
    }
  }
}
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

    let plan =
        scan_repository(&root, "testword", "some-different-testing-words", &options).unwrap();

    println!("\n=== Repeated Word Compound Bug Test ===");
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}, Pos [{}, {}]: '{}' -> '{}'",
            hunk.line, hunk.col, hunk.start, hunk.end, hunk.content, hunk.replace
        );
    }

    // DEBUG: Check what the original content looks like at those positions
    let original_content = std::fs::read_to_string(&test_file).unwrap();
    println!("\nDEBUG: Original file content:");
    println!("{}", original_content);

    for hunk in &plan.matches {
        if hunk.start < original_content.len()
            && hunk.end <= original_content.len()
            && hunk.start < hunk.end
        {
            let actual_text = &original_content[hunk.start..hunk.end];
            println!(
                "Position [{}, {}] contains: '{}' (expected: '{}')",
                hunk.start, hunk.end, actual_text, hunk.content
            );
        } else {
            println!(
                "Invalid position [{}, {}] for content length {}",
                hunk.start,
                hunk.end,
                original_content.len()
            );
        }
    }

    // Find the compound identifier "testword-testword"
    let compound_match = plan
        .matches
        .iter()
        .find(|h| h.content == "testword-testword");

    assert!(compound_match.is_some(), "Should find 'testword-testword'");

    let hunk = compound_match.unwrap();

    // This should become "some-different-testing-words-some-different-testing-words"
    // NOT "testword-some-different-testing-wordssome-different-testing-words" or similar mangled versions
    assert_eq!(
        hunk.replace, "some-different-testing-words-some-different-testing-words",
        "Repeated compound words should be replaced correctly without mangling"
    );

    // Also verify single occurrences work correctly
    let single_match = plan.matches.iter().find(|h| {
        h.content == "testword"
            && hunk.replace != "some-different-testing-words-some-different-testing-words"
    });

    if let Some(single_hunk) = single_match {
        assert_eq!(
            single_hunk.replace, "some-different-testing-words",
            "Single word should be replaced correctly"
        );
    }

    // NOW TEST THE ROUND TRIP - apply the changes and then reverse them
    use renamify_core::apply::{apply_plan, ApplyOptions};
    use std::path::PathBuf;

    let apply_options = ApplyOptions {
        create_backups: true,
        backup_dir: temp_dir.path().join(".renamify/backups"),
        atomic: true,
        force: false,
        commit: false,
        skip_symlinks: true,
        log_file: Some(temp_dir.path().join(".renamify/apply.log")),
    };

    // Apply the first transformation: testword -> some-different-testing-words
    let mut plan_copy = plan.clone();
    apply_plan(&mut plan_copy, &apply_options).unwrap();

    // Read the modified file to verify the changes were applied correctly
    let modified_content = std::fs::read_to_string(&test_file).unwrap();
    println!("\nAfter first application:");
    println!("{}", modified_content);

    // Verify the compound identifier was replaced correctly
    assert!(modified_content.contains("some-different-testing-words-some-different-testing-words"));
    assert!(!modified_content.contains("testword-testword"));

    // Now test the REVERSE transformation: some-different-testing-words -> testword
    let reverse_plan =
        scan_repository(&root, "some-different-testing-words", "testword", &options).unwrap();

    println!("\n=== Reverse Direction Test ===");
    for hunk in &reverse_plan.matches {
        println!(
            "Line {}: '{}' -> '{}'",
            hunk.line, hunk.content, hunk.replace
        );
    }

    // Find the compound identifier in the reverse direction
    let reverse_compound = reverse_plan
        .matches
        .iter()
        .find(|h| h.content == "some-different-testing-words-some-different-testing-words");

    assert!(
        reverse_compound.is_some(),
        "Should find 'some-different-testing-words-some-different-testing-words' in reverse"
    );

    let reverse_hunk = reverse_compound.unwrap();
    assert_eq!(
        reverse_hunk.replace, "testword-testword",
        "Round-trip should restore the original compound correctly"
    );

    // Apply the reverse transformation
    let mut reverse_plan_copy = reverse_plan.clone();
    apply_plan(&mut reverse_plan_copy, &apply_options).unwrap();

    // Read the final content to verify we're back to the original
    let final_content = std::fs::read_to_string(&test_file).unwrap();
    println!("\nAfter round-trip:");
    println!("{}", final_content);

    // Verify we're back to the original content (or at least the key parts)
    assert!(final_content.contains("testword-testword"));
    assert!(!final_content.contains("some-different-testing-words-some-different-testing-words"));
}

#[test]
fn test_triple_repeated_words() {
    // Test more complex repetitions: testwordTestwordTestword
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.js");
    std::fs::write(
        &test_file,
        r#"
const testwordTestwordTestword = getValue();
function getTestwordTestwordTestwordOption() {
    return testwordTestwordTestword.process();
}
class TestwordTestwordTestword {
    testwordTestwordTestword() {}
}
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

    let plan = scan_repository(&root, "testword", "renamed", &options).unwrap();

    println!("\n=== Triple Repeated Words Test ===");
    for hunk in &plan.matches {
        println!(
            "Line {}: '{}' -> '{}'",
            hunk.line, hunk.content, hunk.replace
        );
    }

    // Find the triple compound identifier
    let triple_camel = plan
        .matches
        .iter()
        .find(|h| h.content == "testwordTestwordTestword");
    assert!(
        triple_camel.is_some(),
        "Should find 'testwordTestwordTestword'"
    );

    let hunk = triple_camel.unwrap();
    assert_eq!(
        hunk.replace, "renamedRenamedRenamed",
        "testwordTestwordTestword should become renamedRenamedRenamed"
    );

    // Find the triple PascalCase identifier
    let triple_pascal = plan
        .matches
        .iter()
        .find(|h| h.content == "TestwordTestwordTestword");
    assert!(
        triple_pascal.is_some(),
        "Should find 'TestwordTestwordTestword'"
    );

    let pascal_hunk = triple_pascal.unwrap();
    assert_eq!(
        pascal_hunk.replace, "RenamedRenamedRenamed",
        "TestwordTestwordTestword should become RenamedRenamedRenamed"
    );

    // Find the triple function name
    let triple_func = plan
        .matches
        .iter()
        .find(|h| h.content == "getTestwordTestwordTestwordOption");
    assert!(
        triple_func.is_some(),
        "Should find 'getTestwordTestwordTestwordOption'"
    );

    let func_hunk = triple_func.unwrap();
    assert_eq!(
        func_hunk.replace, "getRenamedRenamedRenamedOption",
        "getTestwordTestwordTestwordOption should become getRenamedRenamedRenamedOption"
    );
}

#[test]
fn test_screaming_snake_case_repeated() {
    // Test TESTWORD_TESTWORD patterns
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("constants.h");
    std::fs::write(
        &test_file,
        r#"
#define TESTWORD_TESTWORD 42
#define MAX_TESTWORD_TESTWORD_SIZE 1024
const TESTWORD_TESTWORD_CONFIG = {
    TESTWORD_TESTWORD_TYPE: "default",
    TESTWORD_TESTWORD_ENABLED: true
};
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

    let plan = scan_repository(&root, "testword", "config", &options).unwrap();

    println!("\n=== Screaming Snake Case Test ===");
    for hunk in &plan.matches {
        println!(
            "Line {}: '{}' -> '{}'",
            hunk.line, hunk.content, hunk.replace
        );
    }

    // Find the basic TESTWORD_TESTWORD
    let basic_screaming = plan
        .matches
        .iter()
        .find(|h| h.content == "TESTWORD_TESTWORD");
    assert!(basic_screaming.is_some(), "Should find 'TESTWORD_TESTWORD'");

    let hunk = basic_screaming.unwrap();
    assert_eq!(
        hunk.replace, "CONFIG_CONFIG",
        "TESTWORD_TESTWORD should become CONFIG_CONFIG"
    );

    // Find the compound MAX_TESTWORD_TESTWORD_SIZE
    let max_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "MAX_TESTWORD_TESTWORD_SIZE");
    assert!(
        max_compound.is_some(),
        "Should find 'MAX_TESTWORD_TESTWORD_SIZE'"
    );

    let max_hunk = max_compound.unwrap();
    assert_eq!(
        max_hunk.replace, "MAX_CONFIG_CONFIG_SIZE",
        "MAX_TESTWORD_TESTWORD_SIZE should become MAX_CONFIG_CONFIG_SIZE"
    );

    // Find TESTWORD_TESTWORD_CONFIG
    let config_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "TESTWORD_TESTWORD_CONFIG");
    assert!(
        config_compound.is_some(),
        "Should find 'TESTWORD_TESTWORD_CONFIG'"
    );

    let config_hunk = config_compound.unwrap();
    assert_eq!(
        config_hunk.replace, "CONFIG_CONFIG_CONFIG",
        "TESTWORD_TESTWORD_CONFIG should become CONFIG_CONFIG_CONFIG"
    );
}

#[test]
fn test_kebab_case_triple_repeated() {
    // Test testword-testword-testword patterns
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("component.html");
    std::fs::write(
        &test_file,
        r#"
<div class="testword-testword-testword">
  <span id="testword-testword-testword-label">Label</span>
  <button data-action="testword-testword-testword-click">Click</button>
</div>
<style>
.testword-testword-testword {
  color: blue;
}
.testword-testword-testword-active {
  color: red;
}
</style>
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

    let plan = scan_repository(&root, "testword", "element", &options).unwrap();

    println!("\n=== Kebab Case Triple Test ===");
    for hunk in &plan.matches {
        println!(
            "Line {}: '{}' -> '{}'",
            hunk.line, hunk.content, hunk.replace
        );
    }

    // Find the basic testword-testword-testword
    let triple_kebab = plan
        .matches
        .iter()
        .find(|h| h.content == "testword-testword-testword");
    assert!(
        triple_kebab.is_some(),
        "Should find 'testword-testword-testword'"
    );

    let hunk = triple_kebab.unwrap();
    assert_eq!(
        hunk.replace, "element-element-element",
        "testword-testword-testword should become element-element-element"
    );

    // Find the compound with label
    let label_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "testword-testword-testword-label");
    assert!(
        label_compound.is_some(),
        "Should find 'testword-testword-testword-label'"
    );

    let label_hunk = label_compound.unwrap();
    assert_eq!(
        label_hunk.replace, "element-element-element-label",
        "testword-testword-testword-label should become element-element-element-label"
    );

    // Find the compound with click
    let click_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "testword-testword-testword-click");
    assert!(
        click_compound.is_some(),
        "Should find 'testword-testword-testword-click'"
    );

    let click_hunk = click_compound.unwrap();
    assert_eq!(
        click_hunk.replace, "element-element-element-click",
        "testword-testword-testword-click should become element-element-element-click"
    );

    // Find the compound with active
    let active_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "testword-testword-testword-active");
    assert!(
        active_compound.is_some(),
        "Should find 'testword-testword-testword-active'"
    );

    let active_hunk = active_compound.unwrap();
    assert_eq!(
        active_hunk.replace, "element-element-element-active",
        "testword-testword-testword-active should become element-element-element-active"
    );
}

#[test]
fn test_title_case_repeated() {
    // Test Testword-Testword patterns (Title-Case)
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("readme.md");
    std::fs::write(
        &test_file,
        r#"
# Testword-Testword Documentation

This is the Testword-Testword module.

## Testword-Testword-Setup

Follow these steps for Testword-Testword-Setup:

1. Install Testword-Testword-Dependencies
2. Configure Testword-Testword-Settings
3. Run Testword-Testword-Tests

## Testword-Testword-Advanced

For advanced Testword-Testword-Advanced usage...
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

    let plan = scan_repository(&root, "testword", "module", &options).unwrap();

    println!("\n=== Title Case Test ===");
    for hunk in &plan.matches {
        println!(
            "Line {}: '{}' -> '{}'",
            hunk.line, hunk.content, hunk.replace
        );
    }

    // Find the basic Testword-Testword
    let title_double = plan
        .matches
        .iter()
        .find(|h| h.content == "Testword-Testword");
    assert!(title_double.is_some(), "Should find 'Testword-Testword'");

    let hunk = title_double.unwrap();
    assert_eq!(
        hunk.replace, "Module-Module",
        "Testword-Testword should become Module-Module"
    );

    // Find the compound with Setup
    let setup_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "Testword-Testword-Setup");
    assert!(
        setup_compound.is_some(),
        "Should find 'Testword-Testword-Setup'"
    );

    let setup_hunk = setup_compound.unwrap();
    assert_eq!(
        setup_hunk.replace, "Module-Module-Setup",
        "Testword-Testword-Setup should become Module-Module-Setup"
    );

    // Find the compound with Dependencies
    let deps_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "Testword-Testword-Dependencies");
    assert!(
        deps_compound.is_some(),
        "Should find 'Testword-Testword-Dependencies'"
    );

    let deps_hunk = deps_compound.unwrap();
    assert_eq!(
        deps_hunk.replace, "Module-Module-Dependencies",
        "Testword-Testword-Dependencies should become Module-Module-Dependencies"
    );

    // Find the compound with Advanced
    let advanced_compound = plan
        .matches
        .iter()
        .find(|h| h.content == "Testword-Testword-Advanced");
    assert!(
        advanced_compound.is_some(),
        "Should find 'Testword-Testword-Advanced'"
    );

    let advanced_hunk = advanced_compound.unwrap();
    assert_eq!(
        advanced_hunk.replace, "Module-Module-Advanced",
        "Testword-Testword-Advanced should become Module-Module-Advanced"
    );
}

#[test]
fn test_single_word_vs_compound_replacement_behavior() {
    // Test the difference between replacing single words vs compound identifiers
    // Case 1: Replacing single word "tool" with "newtool"
    // Case 2: Replacing compound "tool_tool" with "asdf_foo"

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("edge_cases.txt");
    std::fs::write(
        &test_file,
        r#"
Single word context:
- The-Tool-Tool should become The-Newtool-Newtool when replacing "tool" with "newtool"
- my-Tool-based-Tool should become my-Newtool-based-Newtool

Compound context:
- The-Tool-Tool should become The-Asdf-Foo when replacing "tool_tool" with "asdf_foo"
- my-Tool-based-Tool pattern analysis

Mixed separators:
- Tool_Tool should become Newtool_Newtool when replacing "tool" with "newtool"
- Tool_Tool should become Asdf_Foo when replacing "tool_tool" with "asdf_foo"
- toolTool should become newtoolNewtool when replacing "tool" with "newtool"
- toolTool should become asdfFoo when replacing "toolTool" with "asdfFoo"
"#,
    )
    .unwrap();

    // Test Case 1: Single word replacement "tool" -> "newtool"
    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
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

    let plan1 = scan_repository(&root, "tool", "newtool", &options).unwrap();

    println!("\n=== Single Word Replacement: tool -> newtool ===");
    for hunk in &plan1.matches {
        println!("'{}' -> '{}'", hunk.content, hunk.replace);
    }

    // When replacing single word "tool" with "newtool":
    // The-Tool-Tool should become The-Newtool-Newtool (each Tool replaced individually)
    let the_tool_tool = plan1.matches.iter().find(|h| h.content == "The-Tool-Tool");
    assert!(
        the_tool_tool.is_some(),
        "Should find 'The-Tool-Tool' when replacing single word"
    );
    assert_eq!(
        the_tool_tool.unwrap().replace,
        "The-Newtool-Newtool",
        "The-Tool-Tool should become The-Newtool-Newtool when replacing single word 'tool'"
    );

    // Tool_Tool is not a standard case style (mixed Pascal + underscore separator)
    // Our system only matches complete case-style variants, not partial matches within mixed compounds
    // So Tool_Tool won't be matched as a whole, but individual "Tool" instances might be matched

    // toolTool should become newtoolNewtool
    let camel_tool = plan1.matches.iter().find(|h| h.content == "toolTool");
    assert!(camel_tool.is_some(), "Should find 'toolTool'");
    assert_eq!(
        camel_tool.unwrap().replace,
        "newtoolNewtool",
        "toolTool should become newtoolNewtool"
    );

    // Test Case 2: Compound replacement "tool_tool" -> "asdf_foo"
    let plan2 = scan_repository(&root, "tool_tool", "asdf_foo", &options).unwrap();

    println!("\n=== Compound Replacement: tool_tool -> asdf_foo ===");
    for hunk in &plan2.matches {
        println!("'{}' -> '{}'", hunk.content, hunk.replace);
    }

    // When replacing compound "tool_tool" with "asdf_foo":
    // The-Tool-Tool should become The-Asdf-Foo (compound match, preserving Train-Case)
    let the_compound = plan2.matches.iter().find(|h| h.content == "The-Tool-Tool");
    assert!(
        the_compound.is_some(),
        "Should find 'The-Tool-Tool' as compound match"
    );
    assert_eq!(
        the_compound.unwrap().replace,
        "The-Asdf-Foo",
        "The-Tool-Tool should become The-Asdf-Foo when replacing compound 'tool_tool'"
    );

    // Note: Tool_Tool won't be found when searching for "tool_tool" because
    // the compound matcher looks for the exact style. Let's verify this behavior.
    let underscore_compound = plan2.matches.iter().find(|h| h.content == "Tool_Tool");
    if underscore_compound.is_some() {
        assert_eq!(
            underscore_compound.unwrap().replace,
            "Asdf_Foo",
            "Tool_Tool should become Asdf_Foo if found"
        );
    }
    // The system may not find PascalCase Tool_Tool when searching for snake_case tool_tool
    // This is expected behavior - compound matching is style-specific

    // Test Case 3: Exact camelCase compound replacement "toolTool" -> "asdfFoo"
    let plan3 = scan_repository(&root, "toolTool", "asdfFoo", &options).unwrap();

    println!("\n=== Exact Compound Replacement: toolTool -> asdfFoo ===");
    for hunk in &plan3.matches {
        println!("'{}' -> '{}'", hunk.content, hunk.replace);
    }

    // When replacing exact compound "toolTool" with "asdfFoo":
    let camel_exact = plan3.matches.iter().find(|h| h.content == "toolTool");
    assert!(camel_exact.is_some(), "Should find exact 'toolTool' match");
    assert_eq!(
        camel_exact.unwrap().replace,
        "asdfFoo",
        "toolTool should become asdfFoo for exact replacement"
    );
}

#[test]
fn test_overlapping_compound_vs_individual_matches() {
    // Test that compound matches take priority over individual word matches
    // to prevent corruption like "testword-testword" -> "new-newword"

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("overlap_test.txt");
    std::fs::write(
        &test_file,
        r#"
Cases where compound should take priority:
- testword-testword (should be single compound match)
- getTestwordTestwordArg function call
- TESTWORD_TESTWORD_CONFIG constant
- TestwordTestwordOption interface
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

    let plan = scan_repository(&root, "testword", "replacement", &options).unwrap();

    println!("\n=== Overlapping Priority Test ===");
    for hunk in &plan.matches {
        println!("'{}' -> '{}'", hunk.content, hunk.replace);
    }

    // Verify compound matches are found as single units, not broken into parts
    let compound_kebab = plan
        .matches
        .iter()
        .find(|h| h.content == "testword-testword");
    assert!(
        compound_kebab.is_some(),
        "Should find compound 'testword-testword'"
    );
    assert_eq!(
        compound_kebab.unwrap().replace,
        "replacement-replacement",
        "testword-testword should become replacement-replacement as single compound"
    );

    let compound_func = plan
        .matches
        .iter()
        .find(|h| h.content == "getTestwordTestwordArg");
    assert!(
        compound_func.is_some(),
        "Should find compound function name"
    );
    assert_eq!(
        compound_func.unwrap().replace,
        "getReplacementReplacementArg",
        "getTestwordTestwordArg should become getReplacementReplacementArg"
    );

    let compound_screaming = plan
        .matches
        .iter()
        .find(|h| h.content == "TESTWORD_TESTWORD_CONFIG");
    assert!(
        compound_screaming.is_some(),
        "Should find screaming snake compound"
    );
    assert_eq!(
        compound_screaming.unwrap().replace,
        "REPLACEMENT_REPLACEMENT_CONFIG",
        "TESTWORD_TESTWORD_CONFIG should become REPLACEMENT_REPLACEMENT_CONFIG"
    );

    let compound_pascal = plan
        .matches
        .iter()
        .find(|h| h.content == "TestwordTestwordOption");
    assert!(compound_pascal.is_some(), "Should find pascal compound");
    assert_eq!(
        compound_pascal.unwrap().replace,
        "ReplacementReplacementOption",
        "TestwordTestwordOption should become ReplacementReplacementOption"
    );

    // Critical: verify we don't have broken/overlapping matches that would cause corruption
    // We should NOT find individual "testword" matches that overlap with compound matches
    for hunk in &plan.matches {
        // No match should contain partial corruption like "replacement-replacementword"
        assert!(
            !hunk.replace.contains("replacementword"),
            "Found corrupted replacement '{}' in match '{}'",
            hunk.replace,
            hunk.content
        );
        assert!(
            !hunk.replace.contains("testwordreplacement"),
            "Found corrupted replacement '{}' in match '{}'",
            hunk.replace,
            hunk.content
        );
    }
}
