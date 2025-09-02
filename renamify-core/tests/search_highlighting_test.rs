use renamify_core::preview::render_matches;
use renamify_core::scanner::{MatchHunk, Plan, Stats};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_search_highlighting_positions() {
    // Test that search highlighting correctly highlights the matched text
    // Bug: highlighting is off by one character - shows "ore_ext/" instead of "core_ext"

    // Create a plan with a match that mimics the real search results
    let mut matches = vec![];

    // The actual line from the Rails codebase
    let line = r#"require "active_support/core_ext/hash""#;

    // Add match as the scanner would create it (0-based column)
    matches.push(MatchHunk {
        file: PathBuf::from("test/xml_mini_test.rb"),
        line: 6,
        byte_offset: 24, // Position of 'c' in 'core_ext' (0-based)
        char_offset: 24,
        variant: "core_ext".to_string(),
        content: "core_ext".to_string(),
        replace: "".to_string(), // Empty for search
        start: 24,
        end: 32,
        line_before: Some(line.to_string()),
        line_after: None,
        coercion_applied: None,
        original_file: None,
        renamed_file: None,
        patch_hash: None,
    });

    let plan = Plan {
        id: "test".to_string(),
        created_at: "123456".to_string(),
        search: "core_ext".to_string(),
        replace: "".to_string(),
        styles: vec![],
        includes: vec![],
        excludes: vec![],
        matches,
        paths: vec![],
        stats: Stats {
            files_scanned: 1,
            total_matches: 1,
            matches_by_variant: HashMap::new(),
            files_with_matches: 1,
        },
        version: "0.1.0".to_string(),
        created_directories: None,
    };

    // Render with color to test highlighting
    let output = render_matches(&plan, true);

    // The output should highlight exactly "core_ext", not "ore_ext/"
    // The ANSI codes should be:
    // - Text before match: "require \"active_support/"
    // - Highlighted match: \u{1b}[48;2;0;169;88m\u{1b}[38;2;255;255;255mcore_ext\u{1b}[0m
    // - Text after match: "/hash\""

    // Check that the line contains the correct highlighting
    let expected_highlighted = format!(
        "    6:25: require \"active_support/{}{}{}/hash\"",
        "\u{1b}[48;2;0;169;88;38;2;255;255;255m", // Green bg, white fg combined
        "core_ext",
        "\u{1b}[0m" // Reset
    );

    assert!(
        output.contains(&expected_highlighted),
        "Output should contain correctly highlighted 'core_ext', not 'ore_ext/'\nExpected line: {}\nActual output:\n{}",
        expected_highlighted,
        output
    );
}

#[test]
fn test_search_highlighting_extracts_correct_substring() {
    // Direct test of the highlighting logic
    let line = r#"require "active_support/core_ext/hash""#;
    let col = 24usize; // 0-based position of 'c' in 'core_ext'
    let content = "core_ext";

    // This is what the highlighting code does - extract substring at column
    let extracted = &line[col..col + content.len()];
    assert_eq!(
        extracted, "core_ext",
        "Should extract 'core_ext' at column 24, not '{}'",
        extracted
    );

    // Test with column 25 (which would be wrong - off by one)
    let wrong_col = 25usize;
    let wrong_extracted = &line[wrong_col..wrong_col + content.len()];
    assert_eq!(
        wrong_extracted, "ore_ext/",
        "Column 25 would incorrectly extract 'ore_ext/'"
    );
}

#[test]
fn test_search_highlighting_multiple_matches() {
    // Test with multiple matches to ensure all are highlighted correctly
    let mut matches = vec![];

    // Add multiple matches
    let lines = [
        (r#"require "active_support/core_ext/hash""#, 24),
        (r#"require "active_support/core_ext/big_decimal""#, 24),
        (r#"module CoreExt"#, 7), // PascalCase variant at different position
    ];

    for (i, (line, col)) in lines.iter().enumerate() {
        let variant = if i == 2 { "CoreExt" } else { "core_ext" };
        let content = if i == 2 { "CoreExt" } else { "core_ext" };

        matches.push(MatchHunk {
            file: PathBuf::from("test.rb"),
            line: (i + 1) as u64,
            byte_offset: *col as u32,
            char_offset: *col as u32,
            variant: variant.to_string(),
            content: content.to_string(),
            replace: "".to_string(),
            start: *col,
            end: col + content.len(),
            line_before: Some(line.to_string()),
            line_after: None,
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        });
    }

    let plan = Plan {
        id: "test".to_string(),
        created_at: "123456".to_string(),
        search: "core_ext".to_string(),
        replace: "".to_string(),
        styles: vec![],
        includes: vec![],
        excludes: vec![],
        matches,
        paths: vec![],
        stats: Stats {
            files_scanned: 1,
            total_matches: 3,
            matches_by_variant: HashMap::new(),
            files_with_matches: 1,
        },
        version: "0.1.0".to_string(),
        created_directories: None,
    };

    let output = render_matches(&plan, false); // No color for easier testing

    // Verify all matches are shown with correct positions
    assert!(output.contains("1:24:"));
    assert!(output.contains("2:24:"));
    assert!(output.contains("3:7:"));
}
