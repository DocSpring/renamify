#[cfg(windows)]
use renamify_core::{apply_operation, scan_repository, PlanOptions};
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use tempfile::TempDir;

#[test]
#[cfg(windows)]
fn test_patches_have_crlf_line_endings_on_windows() {
    // This test ensures that on Windows, all generated patches have consistent CRLF line endings
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a test file with CRLF line endings (Windows style)
    let test_file = root.join("test.txt");
    let content = "Line 1 with old_name\r\nLine 2 with old_name\r\nLine 3 with old_name\r\n";
    fs::write(&test_file, content).unwrap();

    // Create renamify directory
    let renamify_dir = root.join(".renamify");
    fs::create_dir_all(&renamify_dir).unwrap();

    // Create and apply a plan
    let options = PlanOptions {
        unrestricted_level: 0,
        rename_files: false,
        rename_dirs: false,
        ..Default::default()
    };

    let plan = scan_repository(root, "old_name", "new_name", &options).unwrap();

    // Write the plan
    let plan_path = renamify_dir.join("plan.json");
    let plan_json = serde_json::to_string_pretty(&plan).unwrap();
    fs::write(&plan_path, plan_json).unwrap();

    // Apply the plan - this should create patches
    apply_operation(None, None, false, false, Some(root)).unwrap();

    // Find all patch files
    let backups_dir = renamify_dir.join("backups");
    let patch_files = find_patch_files(&backups_dir);

    assert!(
        !patch_files.is_empty(),
        "Should have created at least one patch file"
    );

    // Check each patch file for consistent CRLF line endings
    for patch_file in patch_files {
        let patch_content = fs::read(&patch_file).unwrap();

        // Check that the patch uses CRLF consistently
        let patch_str = String::from_utf8_lossy(&patch_content);

        // Count LF that are not preceded by CR
        let mut bare_lf_count = 0;
        let bytes = patch_content.as_slice();
        for i in 0..bytes.len() {
            if bytes[i] == b'\n' {
                if i == 0 || bytes[i - 1] != b'\r' {
                    bare_lf_count += 1;
                }
            }
        }

        assert_eq!(
            bare_lf_count, 0,
            "Patch file {:?} has {} bare LF characters without CR. Patch should use CRLF consistently on Windows.\nPatch preview: {}",
            patch_file,
            bare_lf_count,
            &patch_str.chars().take(500).collect::<String>()
        );

        // Also verify that CRLF sequences exist (not just all LF)
        assert!(
            patch_str.contains("\r\n"),
            "Patch file {:?} should contain CRLF line endings on Windows",
            patch_file
        );
    }
}

#[cfg(windows)]
fn find_patch_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut patch_files = Vec::new();
    if dir.exists() {
        for entry in walkdir::WalkDir::new(dir) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("patch") {
                    patch_files.push(path.to_path_buf());
                }
            }
        }
    }
    patch_files
}

#[test]
#[cfg(windows)]
fn test_patch_application_with_crlf_files() {
    // Test that patches can be successfully applied to files with CRLF line endings
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test files with CRLF line endings
    let file1 = root.join("file1.txt");
    let file2 = root.join("file2.txt");

    fs::write(&file1, "First file with old_name here\r\nSecond line\r\n").unwrap();
    fs::write(&file2, "Another file with old_name\r\nMore content\r\n").unwrap();

    // Create renamify directory
    let renamify_dir = root.join(".renamify");
    fs::create_dir_all(&renamify_dir).unwrap();

    // Scan and apply
    let options = PlanOptions::default();
    let plan = scan_repository(root, "old_name", "new_name", &options).unwrap();

    let plan_path = renamify_dir.join("plan.json");
    fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    // Apply should succeed
    apply_operation(None, None, false, false, Some(root))
        .expect("Apply should succeed with CRLF files");

    // Verify files were modified
    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert!(content1.contains("new_name"));
    assert!(content2.contains("new_name"));

    // Most importantly: undo should also work
    use renamify_core::undo_operation;
    undo_operation("latest", Some(root)).expect("Undo should succeed with CRLF patches and files");

    // Verify files were restored
    let restored1 = fs::read_to_string(&file1).unwrap();
    let restored2 = fs::read_to_string(&file2).unwrap();

    assert!(restored1.contains("old_name"));
    assert!(restored2.contains("old_name"));
    assert!(!restored1.contains("new_name"));
    assert!(!restored2.contains("new_name"));
}

// This test runs on all platforms to ensure we don't break non-Windows systems
#[test]
fn test_line_endings_preserved_per_platform() {
    use renamify_core::{apply_operation, scan_repository, undo_operation, PlanOptions};
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a test file with platform-appropriate line endings
    let test_file = root.join("test.md");
    #[cfg(windows)]
    let content = "# Header with old_name\r\n\r\nContent here\r\n";
    #[cfg(not(windows))]
    let content = "# Header with old_name\n\nContent here\n";

    fs::write(&test_file, content).unwrap();

    let renamify_dir = root.join(".renamify");
    fs::create_dir_all(&renamify_dir).unwrap();

    let options = PlanOptions::default();
    let plan = scan_repository(root, "old_name", "new_name", &options).unwrap();

    let plan_path = renamify_dir.join("plan.json");
    fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    // Apply and undo should both work
    apply_operation(None, None, false, false, Some(root))
        .expect("Apply should work on all platforms");

    undo_operation("latest", Some(root)).expect("Undo should work on all platforms");

    // Verify content is restored exactly
    let restored = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        restored, content,
        "Content should be restored exactly with original line endings"
    );
}
