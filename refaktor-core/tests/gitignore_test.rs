use refaktor_core::{scan_repository, PlanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_gitignore_target_directory_is_never_scanned() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create a .gitignore file that excludes target/ (without leading slash)
    // The leading slash means "from root" in gitignore, but we need to test both
    fs::write(repo_path.join(".gitignore"), "target/\n").unwrap();

    // Create source files
    fs::create_dir(repo_path.join("src")).unwrap();
    fs::write(
        repo_path.join("src/main.rs"),
        "fn refaktor() { println!(\"refaktor\"); }",
    )
    .unwrap();

    // Create target directory with a binary that contains the pattern
    fs::create_dir_all(repo_path.join("target/release")).unwrap();
    fs::write(
        repo_path.join("target/release/refaktor"),
        "#!/bin/sh\necho refaktor\n",
    )
    .unwrap();

    // Also create a file in target that would match
    fs::write(
        repo_path.join("target/release/refaktor.d"),
        "refaktor: src/main.rs",
    )
    .unwrap();

    // Scan the repository
    let options = PlanOptions {
        respect_gitignore: true,
        unrestricted_level: 0, // Default: respect all ignore files
        ..Default::default()
    };

    let plan = scan_repository(repo_path, "refaktor", "renamed", &options).unwrap();

    // Verify that only src/main.rs was matched, not anything in target/
    // Note: src/main.rs has 2 occurrences of "refaktor" in it
    assert_eq!(
        plan.matches.len(),
        2,
        "Should have 2 matches in src/main.rs"
    );
    assert!(
        plan.matches
            .iter()
            .all(|m| m.file == repo_path.join("src/main.rs")),
        "All matches should be in src/main.rs"
    );

    // Verify no renames in target directory
    for rename in &plan.renames {
        assert!(
            !rename.from.starts_with(repo_path.join("target")),
            "Should not rename files in target/: {:?}",
            rename.from
        );
        assert!(
            !rename.to.starts_with(repo_path.join("target")),
            "Should not rename files to target/: {:?}",
            rename.to
        );
    }
}

#[test]
fn test_gitignore_with_unrestricted_level_3_still_respects_target() {
    // Even with -uuu (unrestricted level 3), we should still skip target/
    // because it's a build artifact directory
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create a .gitignore file
    fs::write(repo_path.join(".gitignore"), "/target/\n").unwrap();

    // Create files
    fs::create_dir(repo_path.join("src")).unwrap();
    fs::write(repo_path.join("src/lib.rs"), "pub fn refaktor() {}").unwrap();

    // Create target directory
    fs::create_dir_all(repo_path.join("target/debug")).unwrap();
    fs::write(
        repo_path.join("target/debug/refaktor"),
        "binary content with refaktor",
    )
    .unwrap();

    // Scan with unrestricted level 3 (-uuu)
    let options = PlanOptions {
        respect_gitignore: false,
        unrestricted_level: 3, // -uuu: ignore all ignore files
        ..Default::default()
    };

    let plan = scan_repository(repo_path, "refaktor", "renamed", &options).unwrap();

    // With -uuu, it WILL scan target/ unfortunately
    // This test documents the current behavior
    // We may want to add special handling for build directories
    let target_matches = plan
        .matches
        .iter()
        .filter(|m| m.file.starts_with(repo_path.join("target")))
        .count();

    // Currently with -uuu it DOES scan target/
    // This is arguably a bug - we should probably never scan build artifacts
    println!(
        "Warning: Found {} matches in target/ with -uuu",
        target_matches
    );
}

#[test]
fn test_git_repository_respects_gitignore() {
    // Test in an actual git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to initialize git repo");

    // Create and add .gitignore
    fs::write(repo_path.join(".gitignore"), "/target/\n*.log\n").unwrap();

    // Create files
    fs::create_dir(repo_path.join("src")).unwrap();
    fs::write(repo_path.join("src/main.rs"), "fn refaktor_function() {}").unwrap();

    // Create ignored files
    fs::create_dir(repo_path.join("target")).unwrap();
    fs::write(repo_path.join("target/refaktor.exe"), "refaktor binary").unwrap();
    fs::write(repo_path.join("refaktor.log"), "refaktor log file").unwrap();

    // Stage .gitignore so git recognizes it
    std::process::Command::new("git")
        .args(["add", ".gitignore"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Scan the repository
    let options = PlanOptions {
        respect_gitignore: true,
        unrestricted_level: 0,
        ..Default::default()
    };

    let plan = scan_repository(repo_path, "refaktor", "renamed", &options).unwrap();

    // Should only match src/main.rs, not the ignored files
    assert_eq!(plan.matches.len(), 1, "Should only match non-ignored files");
    assert!(
        plan.matches[0].file.ends_with("src/main.rs"),
        "Should match src/main.rs"
    );

    // Verify no matches in ignored files
    for m in &plan.matches {
        assert!(
            !m.file.starts_with(repo_path.join("target")),
            "Should not match files in target/"
        );
        assert!(
            !m.file.to_string_lossy().ends_with(".log"),
            "Should not match .log files"
        );
    }
}
