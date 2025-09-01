use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test that binary names in Cargo.toml are always kept in snake_case
/// This is critical for build systems and scripts that depend on predictable binary names
#[test]
fn test_binary_name_stays_snake_case() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test Cargo.toml with a binary name
    let cargo_toml_content = r#"
[package]
name = "tool"
version = "1.0.0"

[[bin]]
name = "tool"
path = "src/main.rs"
"#;

    let cargo_toml_path = temp_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

    // Test renaming to PascalCase - binary name should stay snake_case
    let mut cmd = Command::cargo_bin("renamify").unwrap();
    cmd.arg("rename")
        .arg("tool")
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

    // Find the binary name replacement (line 7 in the TOML, under [[bin]])
    let binary_replacement = matches.iter().find(|m| m["line"].as_u64().unwrap() == 7);

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
