use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_no_acronyms_flag() {
    // Test that acronym detection can be disabled
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with XMLHttpRequest
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let req = XMLHttpRequest::new();\nlet api = API_KEY;\n",
    )
    .unwrap();

    // Test with acronyms disabled
    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: true, // Disable acronym detection
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
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "XMLHttpRequest", "NewRequest", &options).unwrap();

    // When acronyms are disabled, XMLHttpRequest will still match as an exact string
    // but won't generate case variants properly
    assert_eq!(
        plan.matches.len(),
        1,
        "XMLHttpRequest should still match as exact string when acronyms are disabled"
    );
    assert_eq!(plan.matches[0].before, "XMLHttpRequest");
    assert_eq!(plan.matches[0].after, "NewRequest");

    // But API_KEY would still match since it's an exact match
    let plan2 = scan_repository(&root, "API_KEY", "NEW_KEY", &options).unwrap();
    assert_eq!(
        plan2.matches.len(),
        1,
        "API_KEY should still match as exact text even with acronyms disabled"
    );
}

#[test]
fn test_include_acronyms_flag() {
    // Test adding custom acronyms
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with custom acronym
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let k8s = K8SCluster::new();\nlet gcp = GCPProvider::init();\n",
    )
    .unwrap();

    // Test with custom acronyms included
    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
        include_acronyms: vec!["K8S".to_string(), "GCP".to_string()], // Add custom acronyms
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
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "K8SCluster", "KubeCluster", &options).unwrap();

    // K8SCluster should be found and replaced
    assert!(
        !plan.matches.is_empty(),
        "K8SCluster should be found with K8S as an acronym"
    );
    assert_eq!(plan.matches[0].before, "K8SCluster");
    assert_eq!(plan.matches[0].after, "KubeCluster");

    let plan2 = scan_repository(&root, "GCPProvider", "CloudProvider", &options).unwrap();
    assert!(
        !plan2.matches.is_empty(),
        "GCPProvider should be found with GCP as an acronym"
    );
}

#[test]
fn test_exclude_acronyms_flag() {
    // Test excluding default acronyms
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with ID (a default acronym)
    let test_file = root.join("test.rs");
    std::fs::write(&test_file, "let user_id = getUserID();\n").unwrap();

    // Test with ID excluded from acronyms
    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec!["ID".to_string()], // Exclude ID from acronyms
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
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "getUserID", "getUserIdentifier", &options).unwrap();

    // With ID excluded, getUserID will still match as exact string
    // but won't generate proper case variants
    assert_eq!(
        plan.matches.len(),
        1,
        "getUserID should still match as exact string when ID is excluded from acronyms"
    );
    assert_eq!(plan.matches[0].before, "getUserID");
    assert_eq!(plan.matches[0].after, "getUserIdentifier");
}

#[test]
fn test_only_acronyms_flag() {
    // Test replacing the entire acronym list
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with various acronyms
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let api = APIClient::new();\nlet aws = AWSProvider::init();\nlet id = getUserID();\n",
    )
    .unwrap();

    // Test with only AWS as an acronym (replacing the default list)
    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec!["AWS".to_string()], // Only AWS is an acronym
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

    // AWSProvider should match
    let plan1 = scan_repository(&root, "AWSProvider", "CloudProvider", &options).unwrap();
    assert!(
        !plan1.matches.is_empty(),
        "AWSProvider should match with AWS in only_acronyms"
    );

    // APIClient will still match as exact string (API is not in the list)
    let plan2 = scan_repository(&root, "APIClient", "InterfaceClient", &options).unwrap();
    assert_eq!(
        plan2.matches.len(),
        1,
        "APIClient should still match as exact string when API is not in only_acronyms"
    );

    // getUserID will still match as exact string (ID is not in the list)
    let plan3 = scan_repository(&root, "getUserID", "getUserIdentifier", &options).unwrap();
    assert_eq!(
        plan3.matches.len(),
        1,
        "getUserID should still match as exact string when ID is not in only_acronyms"
    );
}

#[test]
fn test_acronym_case_insensitive() {
    // Test that acronyms are case-insensitive
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with mixed case acronyms
    let test_file = root.join("test.rs");
    std::fs::write(&test_file, "let api = ApiClient::new();\n").unwrap();

    // Test with api in lowercase
    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
        include_acronyms: vec!["api".to_string()], // lowercase
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
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    // Should still work with ApiClient (Api is matched as API)
    let plan = scan_repository(&root, "ApiClient", "InterfaceClient", &options).unwrap();
    assert!(
        !plan.matches.is_empty(),
        "ApiClient should match even when 'api' is provided in lowercase"
    );
}
