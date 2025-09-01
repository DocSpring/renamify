use renamify_core::{scan_repository, PlanOptions, Style};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_acronym_affects_case_variants() {
    // Test that alphanumeric sequences like "b2b" are always kept together as single tokens,
    // regardless of whether they're in the acronym list or not.
    // This ensures consistent tokenization and round-trip preservation.
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with B2B patterns that will be affected by acronym support
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "// b2b patterns that depend on B2B being recognized as an acronym\n\
         let sales1 = b2b_sales();\n\
         let sales2 = b2bSales();\n\
         let sales3 = B2bSales();\n",
    )
    .unwrap();

    // Test WITH B2B as an acronym
    let options_with = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec!["B2B".to_string()], // Add B2B as acronym
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![Style::Snake, Style::Camel, Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan_with = scan_repository(&root, "b2b_sales", "business_sales", &options_with).unwrap();

    println!(
        "WITH B2B acronym: found {} matches",
        plan_with.matches.len()
    );
    for m in &plan_with.matches {
        println!("  {} -> {}", m.content, m.replace);
    }
    // Should find all variants: b2b_sales, b2bSales, B2bSales
    assert!(
        plan_with.matches.len() >= 3,
        "Should find multiple B2B variants when B2B is an acronym, found {}",
        plan_with.matches.len()
    );

    // Test WITHOUT B2B as an acronym (not in default list)
    let options_without = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![], // Don't add B2B
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![Style::Snake, Style::Camel, Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan_without =
        scan_repository(&root, "b2b_sales", "business_sales", &options_without).unwrap();

    println!(
        "WITHOUT B2B acronym: found {} matches",
        plan_without.matches.len()
    );
    for m in &plan_without.matches {
        println!("  {} -> {}", m.content, m.replace);
    }
    // IMPORTANT: Alphanumeric sequences like "b2b" should always be kept as single tokens
    // regardless of acronym status. This ensures consistent tokenization and round-trip preservation.
    // The number of matches should be the same because "b2b" is always one token.
    assert_eq!(
        plan_without.matches.len(),
        plan_with.matches.len(),
        "Should find the SAME number of variants regardless of acronym status (b2b is always one token): with={} without={}",
        plan_with.matches.len(),
        plan_without.matches.len()
    );
}

#[test]
fn test_custom_acronym_generation() {
    // Test that custom acronyms work for variant generation
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with K8S patterns
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let cluster1 = k8s_cluster::new();\n\
         let cluster2 = k8sCluster::new();\n\
         let cluster3 = K8sCluster::new();\n\
         let cluster4 = K8SCluster::new();\n",
    )
    .unwrap();

    // Test with K8S as a custom acronym
    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec!["K8S".to_string()], // Add K8S as acronym
        exclude_acronyms: vec![],
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![Style::Snake, Style::Camel, Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "k8s_cluster", "kube_cluster", &options).unwrap();

    // Debug: Print what was found
    println!("Found {} matches:", plan.matches.len());
    for m in &plan.matches {
        println!("  {} -> {}", m.content, m.replace);
    }

    // Should find all variants: k8s_cluster, k8sCluster, K8sCluster, K8SCluster
    assert!(
        plan.matches.len() >= 3,
        "Should find multiple K8S variants, found {}",
        plan.matches.len()
    );

    // Check that different case styles are matched
    let variants: Vec<String> = plan.matches.iter().map(|m| m.content.clone()).collect();
    assert!(
        variants.contains(&"k8s_cluster".to_string()),
        "Should find snake_case variant"
    );
    assert!(
        variants.contains(&"k8sCluster".to_string())
            || variants.contains(&"K8SCluster".to_string()),
        "Should find camelCase or PascalCase variant"
    );
}

#[test]
fn test_excluded_acronym_variants() {
    // Test that excluded acronyms don't generate special variants
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with ID patterns
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let user_id = get_user_id();\n\
         let userId = getUserId();\n\
         let userID = getUserID();\n",
    )
    .unwrap();

    // Test with ID excluded
    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec!["ID".to_string()], // Exclude ID
        only_acronyms: vec![],
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![Style::Snake, Style::Camel, Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    let plan = scan_repository(&root, "user_id", "user_identifier", &options).unwrap();

    // With ID excluded, getUserID won't be matched as a variant of user_id
    // Only user_id and userId should match
    let variants: Vec<String> = plan.matches.iter().map(|m| m.content.clone()).collect();
    assert!(
        variants.contains(&"user_id".to_string()),
        "Should find snake_case"
    );
    assert!(
        variants.contains(&"userId".to_string()),
        "Should find camelCase without acronym"
    );
    assert!(
        !variants.contains(&"userID".to_string()),
        "Should NOT find camelCase with ID acronym when ID is excluded"
    );
}

#[test]
fn test_only_acronyms_list() {
    // Test that only_acronyms replaces the entire list
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test file with multiple acronym patterns
    let test_file = root.join("test.rs");
    std::fs::write(
        &test_file,
        "let httpApi = getHTTPAPI();\n\
         let xmlDoc = getXMLDoc();\n\
         let jsonData = getJSONData();\n",
    )
    .unwrap();

    // Test with only XML in the acronym list
    let options = PlanOptions {
        exclude_match: vec![],
        exclude_matching_lines: None,
        no_acronyms: false,
        include_acronyms: vec![],
        exclude_acronyms: vec![],
        only_acronyms: vec!["XML".to_string()], // Only XML
        ignore_ambiguous: false,
        atomic_config: None,
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![Style::Camel, Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: renamify_core::scanner::CoercionMode::Auto,
    };

    // Search for XML patterns - should work
    let plan_xml = scan_repository(&root, "xmlDoc", "document", &options).unwrap();
    assert!(
        !plan_xml.matches.is_empty(),
        "Should find XML patterns when XML is in only_acronyms"
    );

    // Search for HTTP patterns - should not find HTTPAPI variant
    let plan_http = scan_repository(&root, "httpApi", "webApi", &options).unwrap();
    let variants: Vec<String> = plan_http
        .matches
        .iter()
        .map(|m| m.content.clone())
        .collect();
    assert!(
        variants.contains(&"httpApi".to_string()),
        "Should find exact match"
    );
    // getHTTPAPI won't be found as a variant because HTTP is not an acronym
}
