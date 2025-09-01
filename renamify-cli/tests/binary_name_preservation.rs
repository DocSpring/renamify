use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test that binary names in Cargo.toml are always kept in snake_case
/// This is critical for build systems and scripts that depend on predictable binary names
#[test]
fn test_binary_name_stays_snake_case() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test Cargo.toml with enough context to show snake_case is predominant
    let cargo_toml_content = r#"[package]
name = "mytool"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
description = "Smart search & replace for code and files"

[[bin]]
name = "mytool"
path = "src/main.rs"

[dependencies]
mytool-core = { path = "../mytool-core" }
anyhow = { workspace = true }
clap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
ctrlc = "3.4"
clap_complete = "4.5"
dirs = "6.0"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
assert_fs = { workspace = true }
tempfile = { workspace = true }
"#;

    let cargo_toml_path = temp_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

    // Test renaming from ambiguous single word to clearly snake_case multi-word name
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("rename")
        .arg("mytool")
        .arg("awesome_super_test_utility")
        .arg("--dry-run")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .current_dir(temp_path);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON output to check the replacements
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Debug: print all matches to find the right line
    // eprintln!("All matches:");
    // for m in matches {
    //     eprintln!("  Line {}: {} -> {}",
    //         m["line"].as_u64().unwrap(),
    //         m["content"].as_str().unwrap(),
    //         m["replace"].as_str().unwrap()
    //     );
    // }

    // Find the binary name replacement (line 12 in the TOML, under [[bin]])
    let binary_replacement = matches.iter().find(|m| m["line"].as_u64().unwrap() == 12);

    // The binary name should be replaced with snake_case version
    assert!(
        binary_replacement.is_some(),
        "Should find binary name replacement"
    );
    let replacement = binary_replacement.unwrap();
    let replace_value = replacement["replace"].as_str().unwrap();
    assert_eq!(
        replace_value, "awesome_super_test_utility",
        "Binary name should be snake_case 'awesome_super_test_utility', not '{}'",
        replace_value
    );
    assert_ne!(
        replace_value, "AwesomeSuperTestUtility",
        "Binary name should not be PascalCase"
    );
}

#[test]
fn test_binary_name_with_camel_case_input() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test Cargo.toml
    let cargo_toml_content = r#"
[package]
name = "old_service"
version = "1.0.0"

[[bin]]
name = "old_service"
path = "src/main.rs"
"#;

    let cargo_toml_path = temp_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

    // Test renaming to camelCase - binary name should become camelCase
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("rename")
        .arg("old_service")
        .arg("newServiceModule")
        .arg("--dry-run")
        .arg("--quiet")
        .arg("--output")
        .arg("json")
        .current_dir(temp_path);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Find the binary name replacement
    let binary_replacement = matches.iter().find(|m| {
        m["line"].as_u64().unwrap() == 3 // Line 3 is the binary name line
    });

    assert!(
        binary_replacement.is_some(),
        "Should find binary name replacement"
    );
    let replacement = binary_replacement.unwrap();
    let replace_value = replacement["replace"].as_str().unwrap();

    assert_eq!(
        replace_value, "new_service_module",
        "Binary name should be 'new_service_module' matching the predominantly style in the file"
    );
    assert_ne!(
        replace_value, "newServiceModule",
        "Binary name should not be newServiceModule"
    );
}
