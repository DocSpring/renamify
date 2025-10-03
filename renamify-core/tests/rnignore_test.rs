use renamify_core::{scan_repository, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rnignore_respected_by_default() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    fs::write(temp_dir.path().join("test.txt"), "old_name in test").unwrap();
    fs::write(temp_dir.path().join("ignored.txt"), "old_name in ignored").unwrap();

    // Create .rnignore file that ignores ignored.txt
    fs::write(temp_dir.path().join(".rnignore"), "ignored.txt").unwrap();

    let opts = PlanOptions::default();
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();

    // Should find matches in test.txt and .rnignore (hidden file), but not in ignored.txt (excluded by .rnignore)
    assert_eq!(plan.stats.files_scanned, 2);
    assert_eq!(plan.matches.len(), 1); // Only test.txt contains "old_name", not .rnignore
    assert!(plan.matches[0].file.to_str().unwrap().contains("test.txt"));
    assert!(!plan
        .matches
        .iter()
        .any(|m| m.file.to_str().unwrap().contains("ignored.txt")));
}

#[test]
fn test_rnignore_with_patterns() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files in different directories
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::create_dir_all(temp_dir.path().join("build")).unwrap();

    fs::write(temp_dir.path().join("src/main.rs"), "old_name in src").unwrap();
    fs::write(
        temp_dir.path().join("build/output.txt"),
        "old_name in build",
    )
    .unwrap();
    fs::write(temp_dir.path().join("test.txt"), "old_name in root").unwrap();

    // Create .rnignore that ignores build directory
    fs::write(temp_dir.path().join(".rnignore"), "build/\n*.log").unwrap();

    let opts = PlanOptions::default();
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();

    // Should find matches in src/main.rs, test.txt, and .rnignore (hidden file), but not in build/
    assert_eq!(plan.stats.files_scanned, 3);
    assert_eq!(plan.matches.len(), 2); // Only src/main.rs and test.txt contain "old_name"

    let file_paths: Vec<String> = plan
        .matches
        .iter()
        .map(|m| {
            let path = m.file.to_str().unwrap().to_string();
            // Normalize to forward slashes for consistent comparison
            if cfg!(windows) {
                path.replace('\\', "/")
            } else {
                path
            }
        })
        .collect();

    assert!(file_paths.iter().any(|p| p.contains("src/main.rs")));
    assert!(file_paths.iter().any(|p| p.contains("test.txt")));
    assert!(!file_paths.iter().any(|p| p.contains("build")));
}

#[test]
fn test_rnignore_with_unrestricted_level() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    fs::write(temp_dir.path().join("test.txt"), "old_name in test").unwrap();
    fs::write(temp_dir.path().join("ignored.txt"), "old_name in ignored").unwrap();

    // Create .rnignore file
    fs::write(temp_dir.path().join(".rnignore"), "ignored.txt").unwrap();

    // Test with unrestricted level 0 (default - respects .rnignore, includes hidden files)
    let opts = PlanOptions {
        unrestricted_level: 0,
        ..Default::default()
    };
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();
    assert_eq!(plan.stats.files_scanned, 2); // test.txt and .rnignore (hidden), not ignored.txt

    // Test with unrestricted level 1 (still respects .rnignore, includes hidden files)
    let opts = PlanOptions {
        unrestricted_level: 1,
        ..Default::default()
    };
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();
    assert_eq!(plan.stats.files_scanned, 2); // test.txt and .rnignore (hidden), not ignored.txt

    // Test with unrestricted level 2 (ignores all ignore files including .rnignore)
    let opts = PlanOptions {
        unrestricted_level: 2,
        ..Default::default()
    };
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();
    // Should scan test.txt, ignored.txt, and .rnignore itself
    assert_eq!(plan.stats.files_scanned, 3);
}

#[test]
fn test_rnignore_in_subdirectory() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested directory structure
    fs::create_dir_all(temp_dir.path().join("subdir")).unwrap();

    // Create files
    fs::write(temp_dir.path().join("root.txt"), "old_name in root").unwrap();
    fs::write(
        temp_dir.path().join("subdir/test.txt"),
        "old_name in subdir",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("subdir/ignored.txt"),
        "old_name ignored",
    )
    .unwrap();

    // Create .rnignore in subdirectory
    fs::write(temp_dir.path().join("subdir/.rnignore"), "ignored.txt").unwrap();

    let opts = PlanOptions::default();
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();

    // Should find matches in root.txt, subdir/test.txt, and subdir/.rnignore (hidden), but not subdir/ignored.txt
    assert_eq!(plan.stats.files_scanned, 3);
    assert_eq!(plan.matches.len(), 2); // Only root.txt and subdir/test.txt contain "old_name"

    let file_paths: Vec<String> = plan
        .matches
        .iter()
        .map(|m| {
            let path = m.file.to_str().unwrap().to_string();
            // Normalize to forward slashes for consistent comparison
            if cfg!(windows) {
                path.replace('\\', "/")
            } else {
                path
            }
        })
        .collect();

    assert!(file_paths.iter().any(|p| p.contains("root.txt")));
    assert!(file_paths.iter().any(|p| p.contains("subdir/test.txt")));
    assert!(!file_paths.iter().any(|p| p.contains("ignored.txt")));
}

#[test]
fn test_rnignore_combined_with_gitignore() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    fs::write(temp_dir.path().join("test.txt"), "old_name in test").unwrap();
    fs::write(
        temp_dir.path().join("git_ignored.txt"),
        "old_name git ignored",
    )
    .unwrap();
    fs::write(temp_dir.path().join("rnignored.txt"), "old_name rf ignored").unwrap();
    fs::write(
        temp_dir.path().join("both_ignored.txt"),
        "old_name both ignored",
    )
    .unwrap();

    // Create .gitignore
    fs::write(
        temp_dir.path().join(".gitignore"),
        "git_ignored.txt\nboth_ignored.txt",
    )
    .unwrap();

    // Create .rnignore
    fs::write(
        temp_dir.path().join(".rnignore"),
        "rnignored.txt\nboth_ignored.txt",
    )
    .unwrap();

    let opts = PlanOptions::default();
    let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();

    // Now .gitignore works even outside git repositories (we treat it as a custom ignore file)
    // So both .gitignore and .rnignore should be respected
    // Should scan: test.txt, .gitignore (hidden), .rnignore (hidden)
    // Should NOT scan: git_ignored.txt (in .gitignore), rnignored.txt (in .rnignore), both_ignored.txt (in .rnignore)
    assert_eq!(plan.stats.files_scanned, 3);
    assert_eq!(plan.matches.len(), 1); // Only test.txt contains "old_name"

    let file_paths: Vec<String> = plan
        .matches
        .iter()
        .map(|m| m.file.to_str().unwrap().to_string())
        .collect();

    assert!(file_paths.iter().any(|p| p.contains("test.txt")));
    // All other files should be ignored
}
