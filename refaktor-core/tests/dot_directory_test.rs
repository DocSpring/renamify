use refaktor_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_dot_refaktor_directory_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with .refaktor references similar to the actual codebase
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r#"let refaktor_dir = PathBuf::from(".refaktor");
undo_refactoring(&id, &refaktor_dir)
    .context("Failed to undo refactoring")?;
// Check if .refaktor is already ignored
if is_refaktor_ignored()? {
    return Ok(());
}
temp_dir.child(".refaktor").create_dir_all().unwrap();
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

    let plan = scan_repository(&root, "refaktor", "renamed_refactoring_tool", &options).unwrap();

    // Should find all occurrences including .refaktor

    // Should find:
    // - ".refaktor" (in PathBuf::from)
    // - refaktor_dir (variable name, 2 occurrences)
    // - is_refaktor_ignored (function name)
    // - ".refaktor" (in temp_dir.child)
    assert!(
        plan.stats.total_matches >= 5,
        "Should find all refaktor occurrences including .refaktor"
    );

    // Verify that .refaktor is being replaced (in string literals)
    let dot_refaktor_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|h| {
            h.before == "refaktor"
                && h.line_before
                    .as_ref()
                    .is_some_and(|l| l.contains(".refaktor"))
        })
        .collect();
    assert_eq!(
        dot_refaktor_matches.len(),
        3,
        "Should find all .refaktor string literals"
    );

    for hunk in &dot_refaktor_matches {
        assert!(
            hunk.line_after
                .as_ref()
                .is_some_and(|l| l.contains(".renamed_refactoring_tool")),
            "Should replace .refaktor with .renamed_refactoring_tool"
        );
    }
}

#[test]
fn test_import_statement_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with import statements like in the actual code
    let test_file = root.join("main.rs");
    std::fs::write(&test_file,
        r"use refaktor_core::{
    apply_plan, ApplyOptions, Plan, PlanOptions, scan_repository, write_plan, 
    write_preview, Style, History, format_history, get_status, undo_refactoring, redo_refactoring,
};
let preview_output = refaktor_core::preview::render_plan(&plan, preview_format.into(), Some(use_color))?;
coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
"
    ).unwrap();

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
        styles: Some(vec![refaktor_core::Style::Snake]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(
        &root,
        "refaktor_core",
        "renamed_refactoring_tool_core",
        &options,
    )
    .unwrap();

    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}'",
            hunk.line, hunk.col, hunk.before, hunk.after
        );
    }

    // Should find all 3 occurrences of refaktor_core
    assert_eq!(
        plan.stats.total_matches, 3,
        "Should find all refaktor_core module references"
    );

    // Verify each is properly replaced
    for hunk in &plan.matches {
        assert_eq!(hunk.before, "refaktor_core");
        assert_eq!(hunk.after, "renamed_refactoring_tool_core");
    }
}

#[test]
fn test_binary_name_in_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create markdown file with binary command examples
    let test_file = root.join("README.md");
    std::fs::write(
        &test_file,
        r"## CLI contract

Binary: `refaktor`

Commands:

- `refaktor plan <old> <new> [opts]`
- `refaktor apply [--plan PATH | --id ID] [--atomic true] [--commit]`
- `refaktor undo <id>`
- `refaktor redo <id>`
- `refaktor history [--limit N]`
- `refaktor status`
",
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

    let plan = scan_repository(&root, "refaktor", "renamed_refactoring_tool", &options).unwrap();

    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}: '{}'",
            hunk.line,
            hunk.line_before.as_ref().unwrap_or(&hunk.before)
        );
    }

    // Should find all 7 occurrences of "refaktor" in the markdown
    assert_eq!(
        plan.stats.total_matches, 7,
        "Should find all refaktor commands in markdown"
    );
}

#[test]
fn test_multiple_variants_same_line() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Test case from the actual bug report - multiple instances on same line
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r"        preview_format: PreviewFormatArg,
impl From<PreviewFormatArg> for PreviewFormat {",
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
        styles: Some(vec![refaktor_core::Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();

    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}' in '{}'",
            hunk.line,
            hunk.col,
            hunk.before,
            hunk.after,
            hunk.line_before.as_ref().unwrap_or(&String::new())
        );
    }

    // When searching for 'preview_format' with Pascal style only:

    // Line 1 has preview_format (snake_case - NOT included) and PreviewFormatArg (Pascal compound)
    // Line 2 has two occurrences: PreviewFormatArg and PreviewFormat
    // IMPORTANT: With the new --exclude-styles behavior, when we specify
    // styles: Some(vec![Pascal]), we're ONLY including Pascal style.
    // Since 'preview_format' is snake_case, it won't be included in the variant map.
    // So searching for "preview_format" with only Pascal style will find:
    // - PreviewFormat (Pascal case variant)
    // - PreviewFormatArg (compound words containing the pattern in Pascal style)
    // It will NOT find 'preview_format' because snake_case is not in the styles list
    assert_eq!(
        plan.stats.total_matches, 3,
        "Should find only Pascal variants, not snake_case original"
    );

    // Check that both instances on line 2 are found
    let line2_matches: Vec<_> = plan.matches.iter().filter(|h| h.line == 2).collect();
    assert_eq!(
        line2_matches.len(),
        2,
        "Should find both instances on line 2"
    );
}
