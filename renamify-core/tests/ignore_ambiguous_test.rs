use renamify_core::operations::plan::plan_operation;
use std::fs;
use tempfile::TempDir;

#[test]
fn ignore_ambiguous_flag_skips_plain_words() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();

    let file_path = root.join("example.rs");
    fs::write(&file_path, "let config = 1;\nfn config() {}\n").expect("write test file");

    // Baseline without ignore flag should capture both occurrences.
    let (baseline, _) = plan_operation(
        "config",
        "setting",
        vec![root.to_path_buf()],
        vec![],
        vec![],
        true,
        0,
        false,
        false,
        &[],
        &[],
        &[],
        vec![],
        None,
        None,
        None,
        true,
        false,
        false,
        false,
        vec![],
        vec![],
        vec![],
        false,
        Some(root),
        None,
    )
    .expect("baseline plan");

    let baseline_plan = baseline.plan.expect("baseline plan missing");
    assert_eq!(
        baseline_plan.stats.total_matches, 2,
        "expected two matches without ignore flag"
    );
    assert!(!baseline_plan.matches.is_empty());

    // Now enable ignore_ambiguous and ensure we skip ambiguous identifiers like `config`.
    let (filtered, _) = plan_operation(
        "config",
        "setting",
        vec![root.to_path_buf()],
        vec![],
        vec![],
        true,
        0,
        false,
        false,
        &[],
        &[],
        &[],
        vec![],
        None,
        None,
        None,
        true,
        false,
        false,
        false,
        vec![],
        vec![],
        vec![],
        true,
        Some(root),
        None,
    )
    .expect("filtered plan");

    let filtered_plan = filtered.plan.expect("filtered plan missing");
    assert_eq!(
        filtered_plan.stats.total_matches, 0,
        "ambiguous matches should be skipped"
    );
    assert!(
        filtered_plan.matches.is_empty(),
        "no matches expected when ignoring ambiguous identifiers"
    );
}
