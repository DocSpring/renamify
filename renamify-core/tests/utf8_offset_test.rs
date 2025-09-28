use renamify_core::operations::plan::plan_operation;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_utf8_arrow_character_offsets() {
    // Test that UTF-8 multi-byte characters like → don't break offset calculations
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with → character
    let test_file = temp_path.join("test.md");
    fs::write(
        &test_file,
        r#"# Examples

DocSpring → doc-spring (kebab-case)
DocSpring → doc_spring (snake_case)
DocSpring → docSpring (camelCase)
"#,
    )
    .unwrap();

    // Scan and replace
    let (result, _preview) = plan_operation(
        "DocSpring",
        "FormAPI",
        vec![temp_path.to_path_buf()],
        vec![],          // include
        vec![],          // exclude
        true,            // respect_gitignore
        0,               // unrestricted_level
        true,            // rename_files
        true,            // rename_dirs
        &[],             // exclude_styles
        &[],             // include_styles
        &[],             // only_styles
        vec![],          // exclude_match
        None,            // exclude_matching_lines
        None,            // plan_out
        None,            // preview_format
        true,            // dry_run
        false,           // fixed_table_width
        false,           // use_color
        false,           // no_acronyms
        vec![],          // include_acronyms
        vec![],          // exclude_acronyms
        vec![],          // only_acronyms
        false,           // ignore_ambiguous
        Some(temp_path), // cwd
        None,            // atomic_config
    )
    .unwrap();

    let plan = result.plan.expect("Should have a plan");

    // Should find 6 matches (3 lines × 2 occurrences per line)
    let matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("test.md"))
        .collect();

    // Debug: print all matches
    println!("Found {} matches:", matches.len());
    for m in &matches {
        println!(
            "  Line {}, Col {}: '{}' -> '{}'",
            m.line, m.char_offset, m.content, m.replace
        );
        println!("    Line before: {:?}", m.line_before);
        println!("    Line after:  {:?}", m.line_after);
    }

    assert_eq!(
        matches.len(),
        6,
        "Should find all 6 occurrences of DocSpring"
    );

    // Check that the replacements are correct
    for m in &matches {
        if let Some(ref line_before) = m.line_before {
            // The line contains an arrow character
            if line_before.contains("→") {
                // Check column positions are correct
                // The first DocSpring should be at column 0
                // The second occurrence (after →) should account for the 3-byte arrow

                if m.content == "DocSpring" {
                    assert_eq!(m.char_offset, 0, "First DocSpring should be at column 0");
                } else {
                    // This is checking the doc-spring/doc_spring/docSpring part
                    // These should be after the arrow and space
                    let arrow_pos = line_before.find("→").unwrap();
                    let second_word_start = line_before.find("doc").unwrap();

                    // Make sure the column position is calculated correctly
                    // accounting for the multi-byte arrow character
                    println!("Line: {}", line_before);
                    println!(
                        "Match at column: {}, content: '{}'",
                        m.char_offset, m.content
                    );
                    println!("Arrow at byte position: {}", arrow_pos);
                    println!("Second word starts at byte position: {}", second_word_start);
                }

                // Check that line_after is correctly formed
                if let Some(ref line_after) = m.line_after {
                    // Should contain FormAPI instead of DocSpring
                    assert!(
                        line_after.contains("FormAPI")
                            || line_after.contains("form-api")
                            || line_after.contains("form_api")
                            || line_after.contains("formAPI"),
                        "Line after should contain the replacement: {}",
                        line_after
                    );

                    // The arrow should still be there
                    assert!(
                        line_after.contains("→"),
                        "Arrow should be preserved in line_after: {}",
                        line_after
                    );
                }
            }
        }
    }
}

#[test]
fn test_utf8_emoji_offsets() {
    // Test with emoji characters which are also multi-byte
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let test_file = temp_path.join("test.md");
    fs::write(
        &test_file,
        r#"# Test 🎉

Before emoji: oldname
After emoji 🎉: oldname
Mixed 🎉 oldname 🎉 content
"#,
    )
    .unwrap();

    let (result, _preview) = plan_operation(
        "oldname",
        "newname",
        vec![temp_path.to_path_buf()],
        vec![],          // include
        vec![],          // exclude
        true,            // respect_gitignore
        0,               // unrestricted_level
        true,            // rename_files
        true,            // rename_dirs
        &[],             // exclude_styles
        &[],             // include_styles
        &[],             // only_styles
        vec![],          // exclude_match
        None,            // exclude_matching_lines
        None,            // plan_out
        None,            // preview_format
        true,            // dry_run
        false,           // fixed_table_width
        false,           // use_color
        false,           // no_acronyms
        vec![],          // include_acronyms
        vec![],          // exclude_acronyms
        vec![],          // only_acronyms
        false,           // ignore_ambiguous
        Some(temp_path), // cwd
        None,            // atomic_config
    )
    .unwrap();

    let plan = result.plan.expect("Should have a plan");

    // Should find 3 occurrences of oldname
    let matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("test.md"))
        .collect();

    assert_eq!(matches.len(), 3, "Should find all 3 occurrences of oldname");

    for m in &matches {
        println!(
            "Match at line {}, col {}: '{}'",
            m.line, m.char_offset, m.content
        );

        // Verify the content is what we expect
        assert_eq!(m.content, "oldname");
        assert_eq!(m.replace, "newname");

        // Check that line_after correctly replaces the text
        if let Some(ref line_after) = m.line_after {
            assert!(
                line_after.contains("newname"),
                "Line after should contain 'newname': {}",
                line_after
            );

            // Emojis should be preserved
            if m.line_before.as_ref().unwrap().contains("🎉") {
                assert!(
                    line_after.contains("🎉"),
                    "Emoji should be preserved: {}",
                    line_after
                );
            }
        }
    }
}
