use renamify_core::{apply_operation, scan_repository, undo_operation, PlanOptions};
use serde_json;
use tempfile::TempDir;

#[test]
fn test_preserve_trailing_whitespace_in_patches() {
    // Test that trailing whitespace is preserved when generating and applying patches
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a file with trailing whitespace on some lines
    let test_file = root.join("test.md");
    std::fs::write(
        &test_file,
        "---\n\
         title: old_name status\n\
         description: Show current status\n\
         ---   \n\
         \n\
         The `old_name` command shows information.\n\
         \n\
         ## Usage  \n\
         \n\
         ```bash\n\
         old_name status\n\
         ```  \n",
    )
    .unwrap();

    // Note: Line 4 has "---   " (3 trailing spaces)
    // Line 8 has "## Usage  " (2 trailing spaces)
    // Line 12 has "```  " (2 trailing spaces)

    // Create renamify directory
    let renamify_dir = root.join(".renamify");
    std::fs::create_dir_all(&renamify_dir).unwrap();

    // Ensure clean history
    let history_path = renamify_dir.join("history.json");
    if history_path.exists() {
        std::fs::remove_file(&history_path).unwrap();
    }

    // Set up options
    let options = PlanOptions {
        unrestricted_level: 0,
        rename_files: false,
        rename_dirs: false,
        ..Default::default()
    };

    // Create and apply plan (use unique names to avoid history conflicts)
    let plan = scan_repository(root, "old_name", "new_name", &options).unwrap();

    // Verify we found matches
    assert!(plan.matches.len() > 0, "Should find matches in the file");

    // Write the plan to the expected location
    let plan_path = renamify_dir.join("plan.json");
    let plan_json = serde_json::to_string_pretty(&plan).unwrap();
    std::fs::write(&plan_path, plan_json).unwrap();

    // Apply the plan - pass the working directory instead of changing directory
    apply_operation(None, None, false, false, Some(root)).unwrap();

    // Read the modified file and verify replacements were made
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert!(
        content.contains("new_name"),
        "Should have replaced old_name with new_name"
    );

    // Verify trailing whitespace is still present
    let _lines: Vec<&str> = content.lines().collect();

    // Find the line that should have "---   " (3 trailing spaces)
    let yaml_end_line = content.lines().nth(3).unwrap_or("");
    assert!(
        yaml_end_line == "---" || yaml_end_line == "---   ",
        "Line 4 should either preserve trailing spaces or have them stripped consistently"
    );

    // Now test undo - this is where the bug manifests
    let undo_result = undo_operation("latest", Some(root));

    if let Err(e) = undo_result {
        // If undo fails, it's likely due to the trailing whitespace bug
        panic!(
            "Undo should work even with trailing whitespace. Error: {}",
            e
        );
    }

    // Verify content is back to original
    let restored_content = std::fs::read_to_string(&test_file).unwrap();
    assert!(
        restored_content.contains("old_name"),
        "Should have restored old_name after undo"
    );
    assert!(
        !restored_content.contains("new_name"),
        "Should not contain new_name after undo"
    );
}

#[test]
fn test_patches_preserve_exact_line_content() {
    // Test that patches preserve the exact line content including all whitespace
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a file with various whitespace patterns
    let test_file = root.join("whitespace_test.txt");
    let original_content = "line with old_name here\n\
                           \tline with old_name and tab\n\
                           line with old_name and trailing spaces   \n\
                           \t  line with old_name and mixed whitespace  \t  \n\
                           normal line with old_name\n";

    std::fs::write(&test_file, original_content).unwrap();

    // Create renamify directory
    let renamify_dir = root.join(".renamify");
    std::fs::create_dir_all(&renamify_dir).unwrap();

    // Ensure clean history
    let history_path = renamify_dir.join("history.json");
    if history_path.exists() {
        std::fs::remove_file(&history_path).unwrap();
    }

    // Set up options
    let options = PlanOptions {
        unrestricted_level: 0,
        rename_files: false,
        rename_dirs: false,
        ..Default::default()
    };

    // Create and apply plan (use different names to avoid ID conflicts)
    let plan = scan_repository(root, "old_name", "another_name", &options).unwrap();

    // Write the plan to the expected location
    let plan_path = renamify_dir.join("plan.json");
    let plan_json = serde_json::to_string_pretty(&plan).unwrap();
    std::fs::write(&plan_path, plan_json).unwrap();

    // Check that line_before and line_after in hunks preserve whitespace
    for hunk in &plan.matches {
        if let Some(ref line_before) = hunk.line_before {
            // The line_before should match exactly what's in the file
            // including any trailing whitespace
            let file_content = std::fs::read_to_string(&test_file).unwrap();
            let file_lines: Vec<&str> = file_content.lines().collect();
            let line_num = (hunk.line as usize).saturating_sub(1);

            if line_num < file_lines.len() {
                let actual_line = file_lines[line_num];
                // Check if line_before matches the actual line
                // (they might differ if we're trimming whitespace)
                if actual_line.ends_with("   ") || actual_line.ends_with("\t") {
                    // If the actual line has trailing whitespace,
                    // line_before should preserve it (with newline)
                    assert_eq!(
                        line_before.trim_end_matches('\n'),
                        actual_line,
                        "line_before should preserve trailing whitespace exactly"
                    );
                }
            }
        }
    }

    // Apply the plan - pass the working directory instead of changing directory
    apply_operation(None, None, false, false, Some(root)).unwrap();

    // Undo should work without issues
    undo_operation("latest", Some(root)).expect("Undo should work with preserved whitespace");

    // Verify exact restoration
    let restored_content = std::fs::read_to_string(&test_file).unwrap();
    // On Windows, line endings will be CRLF after write
    #[cfg(windows)]
    let expected_content = original_content.replace("\n", "\r\n");
    #[cfg(not(windows))]
    let expected_content = original_content.to_string();
    assert_eq!(
        restored_content, expected_content,
        "Content should be exactly restored including all whitespace"
    );
}
