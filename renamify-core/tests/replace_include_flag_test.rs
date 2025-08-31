use renamify_core::{create_simple_plan, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_replace_include_flag_respects_pattern() {
    // Test that --include flag properly filters both content matches and path renames
    // Bug: replace command shows path renames outside of include pattern

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test structure with files inside and outside include pattern
    std::fs::create_dir_all(root.join("test")).unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();

    // Files that should be included
    std::fs::write(
        root.join("test/test_helper.rb"),
        "require 'core_ext/hash'\ncore_ext is loaded",
    )
    .unwrap();

    std::fs::write(
        root.join("test/core_ext_test.rb"),
        "test core_ext functionality",
    )
    .unwrap();

    // Files that should NOT be included
    std::fs::write(root.join("lib/core_ext.rb"), "module core_ext\nend").unwrap();

    std::fs::write(root.join("lib/app.rb"), "require 'core_ext'\ncore_ext.load").unwrap();

    // Directory that should NOT be renamed
    std::fs::create_dir_all(root.join("lib/core_ext")).unwrap();
    std::fs::write(
        root.join("lib/core_ext/hash.rb"),
        "core_ext hash extensions",
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: true,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        includes: vec!["test/**/*".to_string()], // Only include test directory
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: true,
        rename_dirs: true,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
    };

    let plan = create_simple_plan(
        "core_ext",
        "ruby_extras",
        vec![root.clone()],
        &options,
        false, // literal mode
    )
    .unwrap();

    // Check content matches - should only find matches in test/ files
    for hunk in &plan.matches {
        assert!(
            hunk.file.starts_with("test/"),
            "Content match should only be in test/ directory, but found in: {:?}",
            hunk.file
        );
    }

    // Should find content matches in test/test_helper.rb
    let test_helper_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|h| h.file == PathBuf::from("test/test_helper.rb"))
        .collect();
    assert!(
        !test_helper_matches.is_empty(),
        "Should find content matches in test/test_helper.rb"
    );

    // Should NOT find content matches in lib/ files
    let lib_matches: Vec<_> = plan
        .matches
        .iter()
        .filter(|h| h.file.starts_with("lib/"))
        .collect();
    assert!(
        lib_matches.is_empty(),
        "Should NOT find content matches in lib/ files, but found: {:?}",
        lib_matches
    );

    // Check path renames - should only rename paths in test/ directory
    for rename in &plan.paths {
        assert!(
            rename.path.starts_with("test/"),
            "Path rename should only be in test/ directory, but found: {:?} -> {:?}",
            rename.path,
            rename.new_path
        );
    }

    // Should rename test/core_ext_test.rb
    let test_file_rename = plan
        .paths
        .iter()
        .find(|r| r.path == PathBuf::from("test/core_ext_test.rb"));
    assert!(
        test_file_rename.is_some(),
        "Should rename test/core_ext_test.rb"
    );
    if let Some(rename) = test_file_rename {
        assert_eq!(
            rename.new_path,
            PathBuf::from("test/ruby_extras_test.rb"),
            "test/core_ext_test.rb should be renamed to test/ruby_extras_test.rb"
        );
    }

    // Should NOT rename lib/core_ext.rb
    let lib_file_rename = plan
        .paths
        .iter()
        .find(|r| r.path == PathBuf::from("lib/core_ext.rb"));
    assert!(
        lib_file_rename.is_none(),
        "Should NOT rename lib/core_ext.rb as it's outside include pattern"
    );

    // Should NOT rename lib/core_ext/ directory
    let lib_dir_rename = plan
        .paths
        .iter()
        .find(|r| r.path == PathBuf::from("lib/core_ext"));
    assert!(
        lib_dir_rename.is_none(),
        "Should NOT rename lib/core_ext/ directory as it's outside include pattern"
    );
}

#[test]
fn test_replace_include_flag_with_specific_file() {
    // Test that --include works with a specific file path

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create multiple files
    std::fs::write(root.join("file1.txt"), "old_value in file1").unwrap();

    std::fs::write(root.join("file2.txt"), "old_value in file2").unwrap();

    std::fs::write(root.join("old_value.txt"), "content here").unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: true,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        includes: vec!["file1.txt".to_string()], // Only include file1.txt
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: true,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
    };

    let plan = create_simple_plan(
        "old_value",
        "new_value",
        vec![root.clone()],
        &options,
        false,
    )
    .unwrap();

    // Should only find content match in file1.txt
    assert_eq!(
        plan.matches.len(),
        1,
        "Should find exactly one content match"
    );
    assert_eq!(
        plan.matches[0].file,
        PathBuf::from("file1.txt"),
        "Content match should be in file1.txt"
    );

    // Should NOT rename old_value.txt since it's not in the include pattern
    assert!(
        plan.paths.is_empty(),
        "Should not rename any files when old_value.txt is outside include pattern"
    );
}

#[test]
fn test_replace_include_with_directory_pattern() {
    // Test that --include works with directory patterns like "test/" or "test"

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    std::fs::create_dir_all(root.join("test")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();

    std::fs::write(root.join("test/example.txt"), "pattern here").unwrap();

    std::fs::write(root.join("src/example.txt"), "pattern here too").unwrap();

    // Test with "test/" pattern (with trailing slash)
    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: true,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        includes: vec!["test/".to_string()],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: None,
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Off,
    };

    let plan = create_simple_plan(
        "pattern",
        "replacement",
        vec![root.clone()],
        &options,
        false,
    )
    .unwrap();

    // Should only match in test directory
    assert_eq!(plan.matches.len(), 1, "Should find one match in test/");
    assert!(
        plan.matches[0].file.starts_with("test/"),
        "Match should be in test/ directory"
    );
}
