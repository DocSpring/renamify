use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_multiple_matches_per_line() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with multiple matches on same line
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let old_name = old_name + old_name; // old_name\n",
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
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "old_name", "new_name", &options).unwrap();

    // Should find all 4 occurrences on the same line
    assert_eq!(
        plan.stats.total_matches, 4,
        "Should find all 4 occurrences of old_name on the same line"
    );
    assert_eq!(plan.matches.len(), 4, "Should have 4 match hunks");

    // Verify all matches are on the same line but different columns
    for hunk in &plan.matches {
        assert_eq!(hunk.line, 1);
    }

    // Check that columns are different
    let mut columns: Vec<u32> = plan.matches.iter().map(|h| h.col).collect();
    columns.sort_unstable();
    columns.dedup();
    assert_eq!(
        columns.len(),
        4,
        "All matches should have different column positions"
    );
}

#[test]
fn test_module_path_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with module path references
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        "use renamify_core::{Plan, Scanner};\n\
         let result = renamify_core::scan();\n\
         renamify_core::apply();\n",
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
        styles: Some(vec![renamify_core::Style::Snake]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify_core", "smart_search_core", &options).unwrap();

    // Should find all 3 module path references
    assert_eq!(
        plan.stats.total_matches, 3,
        "Should find all module path references"
    );

    // Verify replacements
    for hunk in &plan.matches {
        assert_eq!(hunk.content, "renamify_core");
        assert_eq!(hunk.replace, "smart_search_core");
    }
}

#[test]
fn test_dot_path_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with dot-prefixed paths
    let test_file = root.join("script.sh");
    std::fs::write(
        &test_file,
        "mkdir .renamify\n\
         cd .renamify\n\
         echo 'renamify' > .renamify/config\n\
         rm -rf .renamify\n",
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
        unrestricted_level: 3, // Treat all files as text
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "smart_search", &options).unwrap();

    // Should find all occurrences including dot-prefixed
    assert!(
        plan.stats.total_matches >= 5,
        "Should find renamify in regular text and .renamify paths"
    );

    // Check that .renamify occurrences are found
    let dot_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|h| {
            h.line_before
                .as_ref()
                .is_some_and(|l| l.contains(".renamify"))
        })
        .collect();
    assert!(!dot_matches.is_empty(), "Should find .renamify occurrences");
}

#[test]
fn test_consecutive_occurrences() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with consecutive occurrences
    let test_file = root.join("test.txt");
    std::fs::write(&test_file, "old_nameold_name old_name_old_name\n").unwrap();

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
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "old_name", "new_name", &options).unwrap();

    // Should find the compound identifier and replace ALL occurrences within it
    println!("Found {} matches", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Match: '{}' -> '{}' at col {}",
            hunk.content, hunk.replace, hunk.col
        );
    }

    // Should find old_name_old_name and replace it with new_name_new_name
    assert_eq!(
        plan.stats.total_matches, 1,
        "Should find the compound identifier"
    );
    assert_eq!(plan.matches[0].content, "old_name_old_name");
    assert_eq!(plan.matches[0].replace, "new_name_new_name");
}

#[test]
fn test_camel_case_variant_multiple_per_line() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with multiple camelCase variants on same line
    let test_file = root.join("test.ts");
    std::fs::write(
        &test_file,
        "function getUserName(userName: string): UserName { return new UserName(userName); }\n",
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

    let plan = scan_repository(&root, "user_name", "customer_name", &options).unwrap();

    // Should find getUserName (compound), userName (twice), UserName (twice)
    // getUserName IS matched because it's a compound word containing userName
    assert_eq!(
        plan.stats.total_matches, 5,
        "Should find getUserName (1x), userName (2x) and UserName (2x)"
    );
}

#[test]
fn test_mixed_separators_on_same_line() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with mixed separator styles on same line
    let test_file = root.join("config.yml");
    std::fs::write(
        &test_file,
        "name: user-name, path: user_name, class: UserName, var: userName\n",
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
        styles: Some(vec![
            renamify_core::Style::Snake,
            renamify_core::Style::Kebab,
            renamify_core::Style::Camel,
            renamify_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "user_name", "customer_name", &options).unwrap();

    // Should find all 4 variants
    assert_eq!(
        plan.stats.total_matches, 4,
        "Should find all style variants on the same line"
    );

    // Verify each variant is found
    let variants: Vec<String> = plan.matches.iter().map(|h| h.variant.clone()).collect();
    assert!(variants.contains(&"user-name".to_string()));
    assert!(variants.contains(&"user_name".to_string()));
    assert!(variants.contains(&"UserName".to_string()));
    assert!(variants.contains(&"userName".to_string()));
}

#[test]
fn test_markdown_code_blocks() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create markdown file with code blocks containing the pattern
    let test_file = root.join("README.md");
    std::fs::write(
        &test_file,
        "# Commands\n\
         \n\
         - `renamify plan` - Create a plan\n\
         - `renamify apply` - Apply the plan\n\
         - `renamify undo` - Undo changes\n\
         \n\
         ```bash\n\
         renamify rename old new\n\
         ```\n",
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
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "smart_search", &options).unwrap();

    // Should find all 4 occurrences in markdown
    assert_eq!(
        plan.stats.total_matches, 4,
        "Should find all renamify occurrences in markdown"
    );
}
