// Tests for Title Case and space-separated matching bugs
//
// These tests capture issues where:
// 1. Space-separated patterns are matched in kebab-case contexts where they shouldn't be
// 2. Title Case replacements lose their space separators

use renamify_core::case_model::{generate_variant_map, Style};
use renamify_core::scan_repository;
use renamify_core::scanner::PlanOptions;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_space_separated_false_positive_in_kebab_list() {
    // Bug: "server gateway" should NOT match inside a space-separated list
    // where "mock-server" and "gateway-api-preview" are clearly their own kebab-case words

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("config.yml");

    // Write test content with kebab-case words in a space-separated list
    fs::write(
        &file_path,
        r#"SERVICES: "postgres mock-oauth mock-server gateway-api-preview"
"#,
    )
    .unwrap();

    // Try to rename "server-gateway" to "rack-gateway"
    // This should NOT match "mock-server gateway-api-preview" as if they were one identifier
    let opts = PlanOptions {
        styles: Some(vec![
            Style::Kebab,
            Style::Snake,
            Style::Pascal,
            Style::Camel,
            Style::ScreamingSnake,
            Style::Train,
            Style::ScreamingTrain,
            Style::Title,
            Style::Sentence,
            Style::LowerSentence,
            Style::UpperSentence,
        ]),
        plan_out: temp_dir.path().join(".renamify/plan.json"),
        ..Default::default()
    };

    let plan = scan_repository(temp_dir.path(), "server-gateway", "rack-gateway", &opts).unwrap();

    // Should have NO matches in this file because "mock-server" and "gateway-api-preview"
    // are separate kebab-case identifiers, not a "Server Gateway" Title Case phrase
    let matches_in_file: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("config.yml"))
        .collect();

    assert_eq!(
        matches_in_file.len(),
        0,
        "Should not match space-separated Title Case in kebab-case context. Found matches: {:#?}",
        matches_in_file
    );
}

#[test]
fn test_title_case_preserves_spaces_in_description() {
    // Bug: "Install Server Gateway CLI" should become "Install Rack Gateway CLI"
    // NOT "Install Rackgateway CLI" (which loses the space)

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("Taskfile.go.yml");

    fs::write(
        &file_path,
        r#"tasks:
  install:
    desc: Install Server Gateway CLI
"#,
    )
    .unwrap();

    let opts = PlanOptions {
        styles: Some(vec![
            Style::Kebab,
            Style::Snake,
            Style::Pascal,
            Style::Camel,
            Style::ScreamingSnake,
            Style::Train,
            Style::ScreamingTrain,
            Style::Title,
            Style::Sentence,
            Style::LowerSentence,
            Style::UpperSentence,
        ]),
        plan_out: temp_dir.path().join(".renamify/plan.json"),
        ..Default::default()
    };

    let plan = scan_repository(temp_dir.path(), "server-gateway", "rack-gateway", &opts).unwrap();

    // Should match "Server Gateway" in Title Case
    let matches_in_file: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("Taskfile.go.yml"))
        .collect();

    assert_eq!(matches_in_file.len(), 1, "Should find one Title Case match");

    // The replacement should be "Rack Gateway" (with space), not "Rackgateway"
    assert_eq!(
        matches_in_file[0].replace, "Rack Gateway",
        "Title Case replacement should preserve spaces"
    );
}

#[test]
fn test_title_case_preserves_spaces_in_markdown_heading() {
    // Bug: "## Server Gateway Architecture" should become "## Rack Gateway Architecture"
    // NOT "## Rackgateway Architecture"

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("REFERENCE.md");

    fs::write(
        &file_path,
        r#"# Documentation

## Server Gateway Architecture

This describes the architecture.
"#,
    )
    .unwrap();

    let opts = PlanOptions {
        styles: Some(vec![
            Style::Kebab,
            Style::Snake,
            Style::Pascal,
            Style::Camel,
            Style::ScreamingSnake,
            Style::Train,
            Style::ScreamingTrain,
            Style::Title,
            Style::Sentence,
            Style::LowerSentence,
            Style::UpperSentence,
        ]),
        plan_out: temp_dir.path().join(".renamify/plan.json"),
        ..Default::default()
    };

    let plan = scan_repository(temp_dir.path(), "server-gateway", "rack-gateway", &opts).unwrap();

    let matches_in_file: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("REFERENCE.md"))
        .collect();

    assert_eq!(
        matches_in_file.len(),
        1,
        "Should find one Title Case match in markdown heading"
    );

    assert_eq!(
        matches_in_file[0].replace, "Rack Gateway",
        "Title Case in markdown heading should preserve spaces"
    );
}

#[test]
fn test_title_case_preserves_spaces_in_html() {
    // Bug: "<h3>Configure Server Gateway CLI</h3>" should become "<h3>Configure Rack Gateway CLI</h3>"
    // NOT "<h3>Configure Rackgateway CLI</h3>"

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("template.html");

    fs::write(
        &file_path,
        r#"<html>
<body>
    <h3 style="margin:20px 0 8px;">Configure Server Gateway CLI</h3>
</body>
</html>
"#,
    )
    .unwrap();

    let opts = PlanOptions {
        styles: Some(vec![
            Style::Kebab,
            Style::Snake,
            Style::Pascal,
            Style::Camel,
            Style::ScreamingSnake,
            Style::Train,
            Style::ScreamingTrain,
            Style::Title,
            Style::Sentence,
            Style::LowerSentence,
            Style::UpperSentence,
        ]),
        plan_out: temp_dir.path().join(".renamify/plan.json"),
        ..Default::default()
    };

    let plan = scan_repository(temp_dir.path(), "server-gateway", "rack-gateway", &opts).unwrap();

    let matches_in_file: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("template.html"))
        .collect();

    assert_eq!(
        matches_in_file.len(),
        1,
        "Should find one Title Case match in HTML"
    );

    assert_eq!(
        matches_in_file[0].replace, "Rack Gateway",
        "Title Case in HTML should preserve spaces"
    );
}

#[test]
fn test_title_case_works_correctly_in_go_comments() {
    // This should work fine - verifying that correct Title Case matching still works

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("main.go");

    fs::write(
        &file_path,
        r#"package main

// @title Server Gateway API
// @version 1.0
// @description API for the Server Gateway administration and proxy services.
"#,
    )
    .unwrap();

    let opts = PlanOptions {
        styles: Some(vec![
            Style::Kebab,
            Style::Snake,
            Style::Pascal,
            Style::Camel,
            Style::ScreamingSnake,
            Style::Train,
            Style::ScreamingTrain,
            Style::Title,
            Style::Sentence,
            Style::LowerSentence,
            Style::UpperSentence,
        ]),
        plan_out: temp_dir.path().join(".renamify/plan.json"),
        ..Default::default()
    };

    let plan = scan_repository(temp_dir.path(), "server-gateway", "rack-gateway", &opts).unwrap();

    let matches_in_file: Vec<_> = plan
        .matches
        .iter()
        .filter(|m| m.file.ends_with("main.go"))
        .collect();

    // Should find 2 Title Case matches (one in @title, one in @description)
    assert_eq!(
        matches_in_file.len(),
        2,
        "Should find two Title Case matches in Go comments"
    );

    // Both should preserve spaces
    for m in &matches_in_file {
        assert_eq!(
            m.replace, "Rack Gateway",
            "Title Case replacement should be 'Rack Gateway' with space"
        );
    }
}

#[test]
fn test_exact_title_case_match_is_not_ambiguous() {
    // Bug: Finding an exact match for "Server Gateway" in Title Case
    // should not be considered ambiguous

    // Generate variant map
    let map = generate_variant_map("server-gateway", "rack-gateway", None);

    // Should contain the Title Case variant
    assert!(
        map.contains_key("Server Gateway"),
        "Variant map should contain 'Server Gateway'"
    );

    assert_eq!(
        map.get("Server Gateway"),
        Some(&"Rack Gateway".to_string()),
        "Title Case variant should map to 'Rack Gateway' with space"
    );
}
