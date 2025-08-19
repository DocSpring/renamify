use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_compound_replacement_at_start() {
    // Pattern at the beginning of compound word
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        r"// Snake case
let foo_bar_arg = 1;
let foo_bar_option = 2;

// Camel case  
let fooBarArg = 3;
let fooBarOption = 4;

// Pascal case
type FooBarArg = String;
type FooBarOption = i32;",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
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
            renamify_core::Style::Camel,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound at Start Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }

    // Should replace:
    // foo_bar_arg -> foo_arg
    // foo_bar_option -> foo_option
    // fooBarArg -> fooArg
    // fooBarOption -> fooOption
    // FooBarArg -> FooArg
    // FooBarOption -> FooOption

    assert_eq!(
        plan.stats.total_matches, 6,
        "Should find all compounds starting with pattern"
    );

    // Verify replacements
    let expected = vec![
        ("foo_bar_arg", "foo_arg"),
        ("foo_bar_option", "foo_option"),
        ("fooBarArg", "fooArg"),
        ("fooBarOption", "fooOption"),
        ("FooBarArg", "FooArg"),
        ("FooBarOption", "FooOption"),
    ];

    for (from, to) in expected {
        let found = plan
            .matches
            .iter()
            .any(|h| h.before == from && h.after == to);
        assert!(found, "Should replace {from} with {to}");
    }
}

#[test]
fn test_compound_replacement_in_middle() {
    // Pattern in the middle of compound word
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        r"// Snake case
let should_foo_bar_please = 1;
let get_foo_bar_option = 2;

// Camel case  
let shouldFooBarPlease = 3;
let getFooBarOption = 4;

// Pascal case
type ShouldFooBarPlease = String;
type GetFooBarOption = i32;",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
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
            renamify_core::Style::Camel,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound in Middle Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }

    // Should replace:
    // should_foo_bar_please -> should_foo_please
    // get_foo_bar_option -> get_foo_option
    // shouldFooBarPlease -> shouldFooPlease
    // getFooBarOption -> getFooOption
    // ShouldFooBarPlease -> ShouldFooPlease
    // GetFooBarOption -> GetFooOption

    assert_eq!(
        plan.stats.total_matches, 6,
        "Should find all compounds with pattern in middle"
    );

    // Verify replacements preserve prefix and suffix
    let expected = vec![
        ("should_foo_bar_please", "should_foo_please"),
        ("get_foo_bar_option", "get_foo_option"),
        ("shouldFooBarPlease", "shouldFooPlease"),
        ("getFooBarOption", "getFooOption"),
        ("ShouldFooBarPlease", "ShouldFooPlease"),
        ("GetFooBarOption", "GetFooOption"),
    ];

    for (from, to) in expected {
        let found = plan
            .matches
            .iter()
            .any(|h| h.before == from && h.after == to);
        assert!(found, "Should replace {from} with {to}");
    }
}

#[test]
fn test_compound_replacement_at_end() {
    // Pattern at the end of compound word
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        r"// Snake case
let get_foo_bar = 1;
let load_foo_bar = 2;

// Camel case  
let getFooBar = 3;
let loadFooBar = 4;

// Pascal case
type GetFooBar = String;
type LoadFooBar = i32;",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
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
            renamify_core::Style::Camel,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Compound at End Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }

    // Should replace:
    // get_foo_bar -> get_foo
    // load_foo_bar -> load_foo
    // getFooBar -> getFoo
    // loadFooBar -> loadFoo
    // GetFooBar -> GetFoo
    // LoadFooBar -> LoadFoo

    assert_eq!(
        plan.stats.total_matches, 6,
        "Should find all compounds ending with pattern"
    );

    // Verify replacements preserve prefix
    let expected = vec![
        ("get_foo_bar", "get_foo"),
        ("load_foo_bar", "load_foo"),
        ("getFooBar", "getFoo"),
        ("loadFooBar", "loadFoo"),
        ("GetFooBar", "GetFoo"),
        ("LoadFooBar", "LoadFoo"),
    ];

    for (from, to) in expected {
        let found = plan
            .matches
            .iter()
            .any(|h| h.before == from && h.after == to);
        assert!(found, "Should replace {from} with {to}");
    }
}

#[test]
fn test_exact_match_not_compound() {
    // Should still match exact occurrences that aren't compounds
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        r"let foo_bar = get_foo_bar();
let FooBar = FooBar::new();
let fooBar = getFooBar();",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
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
            renamify_core::Style::Camel,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "foo_bar", "foo", &options).unwrap();

    println!("\n=== Exact Match Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }

    // Should find both exact matches AND compounds
    // Line 1: foo_bar (exact), get_foo_bar (compound)
    // Line 2: FooBar twice (exact)
    // Line 3: fooBar (exact), getFooBar (compound)

    assert_eq!(
        plan.stats.total_matches, 6,
        "Should find both exact and compound matches"
    );
}
