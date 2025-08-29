use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_dot_renamify_directory_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with .renamify references similar to the actual codebase
    let test_file = root.join("main.rs");
    std::fs::write(
        &test_file,
        r#"let renamify_dir = PathBuf::from(".renamify");
undo_renaming(&id, &renamify_dir)
    .context("Failed to undo renaming")?;
// Check if .renamify is already ignored
if is_renamify_ignored()? {
    return Ok(());
}
temp_dir.child(".renamify").create_dir_all().unwrap();
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
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "renamify", "renamed_renaming_tool", &options).unwrap();

    // Should find all occurrences including .renamify

    // Should find:
    // - ".renamify" (in PathBuf::from)
    // - renamify_dir (variable name, 2 occurrences)
    // - is_renamify_ignored (function name)
    // - ".renamify" (in temp_dir.child)
    assert!(
        plan.stats.total_matches == 6,
        "Should find 6 renamify occurrences (including .renamify). Found {}",
        plan.stats.total_matches
    );

    // Verify that .renamify is being replaced (in string literals)
    let dot_renamify_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|h| {
            h.content == "renamify"
                && h.line_before
                    .as_ref()
                    .is_some_and(|l| l.contains(".renamify"))
        })
        .collect();
    assert_eq!(
        dot_renamify_matches.len(),
        3,
        "Should find 3 .renamify string literals. Found {}",
        dot_renamify_matches.len()
    );

    let hunk1 = &dot_renamify_matches[0];
    assert!(
        hunk1.line_after
            .as_ref()
            .is_some_and(|l| l.contains(".renamed_renaming_tool")),
        "Should replace PathBuf::from(\".renamify\") with PathBuf::from(\".renamed_renaming_tool\"). line_after={:?}",
        hunk1.line_after
    );
    let hunk2 = &dot_renamify_matches[1];
    assert!(
        hunk2.line_after
            .as_ref()
            .is_some_and(|l| l.contains(".renamed_renaming_tool")),
        "Should replace // Check if .renamify with // Check if .renamed_renaming_tool. line_after={:?}",
        hunk2.line_after
    );
    let hunk3 = &dot_renamify_matches[2];
    assert!(
        hunk3.line_after
            .as_ref()
            .is_some_and(|l| l.contains(".renamed_renaming_tool")),
        "Should replace child(\".renamify\") with child(\".renamed_renaming_tool\"). line_after={:?}",
        hunk3.line_after
    );
}

#[test]
fn test_import_statement_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with import statements like in the actual code
    let test_file = root.join("main.rs");
    std::fs::write(&test_file,
        r"use renamify_core::{
    apply_plan, ApplyOptions, Plan, PlanOptions, scan_repository, write_plan,
    write_preview, Style, History, format_history, get_status, undo_renaming, redo_renaming,
};
let preview_output = renamify_core::preview::render_plan(&plan, preview_format.into(), Some(use_color))?;
coerce_separators: renamify_core::scanner::CoercionMode::Auto,
"
    ).unwrap();

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
        styles: Some(vec![renamify_core::Style::Snake]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(
        &root,
        "renamify_core",
        "renamed_renaming_tool_core",
        &options,
    )
    .unwrap();

    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}'",
            hunk.line, hunk.col, hunk.content, hunk.replace
        );
    }

    // Should find all 3 occurrences of renamify_core
    assert_eq!(
        plan.stats.total_matches, 3,
        "Should find all renamify_core module references"
    );

    // Verify each is properly replaced
    for hunk in &plan.matches {
        assert_eq!(hunk.content, "renamify_core");
        assert_eq!(hunk.replace, "renamed_renaming_tool_core");
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

Binary: `renamify`

Commands:

- `renamify plan <old> <new> [opts]`
- `renamify apply [--plan PATH | --id ID] [--atomic true] [--commit]`
- `renamify undo <id>`
- `renamify redo <id>`
- `renamify history [--limit N]`
- `renamify status`
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

    let plan = scan_repository(&root, "renamify", "renamed_renaming_tool", &options).unwrap();

    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}: '{}'",
            hunk.line,
            hunk.line_before.as_ref().unwrap_or(&hunk.content)
        );
    }

    // Should find all 7 occurrences of "renamify" in the markdown
    assert_eq!(
        plan.stats.total_matches, 7,
        "Should find all renamify commands in markdown"
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

    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();

    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!(
            "Line {}, Col {}: '{}' -> '{}' in '{}'",
            hunk.line,
            hunk.col,
            hunk.content,
            hunk.replace,
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
