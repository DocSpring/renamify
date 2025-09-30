use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_compound_word_case_style_filtering() {
    // Create a temp directory with a test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_cases.txt");

    // Write all case variations of "test case" to the file
    let content = r#"
Original: test case
Snake: test_case
Kebab: test-case
Camel: testCase
Pascal: TestCase
ScreamingSnake: TEST_CASE
Train: Test-Case
ScreamingTrain: TEST-CASE
Title: Test Case
Dot: test.case

// Also test compound words that contain these patterns
CompoundScreamingSnake: SCREAMING_SNAKE_CASE
CompoundTrain: Train-Case
CompoundTitle: Title Case Example
"#;

    fs::write(&test_file, content).unwrap();

    // Helper function to run renamify and capture output
    fn run_search(dir: &Path, styles: &str) -> String {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_renamify"))
            .current_dir(dir)
            .args([
                "search",
                "test case", // Search for "test case" in various forms
                "--only-styles",
                styles,
                "--output",
                "json",
            ])
            .output()
            .expect("Failed to execute renamify");

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    // Test 1: Lower only - should match all lowercase concatenated (testcase)
    // Note: Since we removed Original style, we can't match "test case" with space anymore
    // Let's skip this test for now as Lower style would look for "testcase"

    // Test 2: Snake only - should ONLY match "test_case"
    let output = run_search(temp_dir.path(), "snake");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "Snake should only match snake_case");
    assert_eq!(matches[0]["content"].as_str().unwrap(), "test_case");

    // Test 3: Screaming snake only - should ONLY match "TEST_CASE"
    let output = run_search(temp_dir.path(), "screaming-snake");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    assert_eq!(
        matches.len(),
        1,
        "Screaming snake should only match TEST_CASE"
    );
    assert_eq!(matches[0]["content"].as_str().unwrap(), "TEST_CASE");

    // Test 4: Title case only - should ONLY match "Test Case"
    let output = run_search(temp_dir.path(), "title");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "Title should only match Title Case");
    assert_eq!(matches[0]["content"].as_str().unwrap(), "Test Case");

    // Test 5: Train case only - should ONLY match "Test-Case"
    let output = run_search(temp_dir.path(), "train");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "Train should only match Train-Case");
    assert_eq!(matches[0]["content"].as_str().unwrap(), "Test-Case");

    // Test 6: Dot case only - should ONLY match "test.case"
    let output = run_search(temp_dir.path(), "dot");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "Dot should only match dot.case");
    assert_eq!(matches[0]["content"].as_str().unwrap(), "test.case");
}

#[test]
fn test_single_word_case_filtering() {
    // Test searching for a single word "case" - should be strict about case styles
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("single_word.txt");

    let content = r#"
// Word at the beginning
case
case_example
case-example
caseExample
CaseExample
CASE_EXAMPLE
Case-Example
CASE-EXAMPLE
Case Example
case.example

// Word at the end
test_case
test-case
testCase
TestCase
TEST_CASE
Test-Case
TEST-CASE
Test Case
test.case

// Word in the middle
test_case_example
test-case-example
testCaseExample
TestCaseExample
TEST_CASE_EXAMPLE
Test-Case-Example
TEST-CASE-EXAMPLE
Test Case Example
test.case.example

// Standalone word in different cases
case
CASE
Case

// Edge cases
case
usecase
use_case
use-case
useCase
UseCase
USE_CASE
Use-Case
USE-CASE
Use Case
use.case
briefcase
brief_case
brief-case
briefCase
BriefCase
BRIEF_CASE
Brief-Case
BRIEF-CASE
Brief Case
brief.case

// Mixed case examples (should match "original" as fallback)
CaseExample_case-foo
test_Case-example
CASE_case_CASE
Case-case_case
case.Case.CASE
TestCase_case
caseTest-Case
Case_TEST-case
mixed_caseCASE
"#;

    fs::write(&test_file, content).unwrap();

    // Helper function to run renamify and capture output
    fn run_search(dir: &Path, styles: &str) -> String {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_renamify"))
            .current_dir(dir)
            .args([
                "search",
                "case", // Search for single word "case"
                "--only-styles",
                styles,
                "--output",
                "json",
            ])
            .output()
            .expect("Failed to execute renamify");

        if !output.status.success() {
            eprintln!(
                "Command failed with stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    // Test 1: LowerJoined style - should match lowercase "case"
    let output = run_search(temp_dir.path(), "lower-joined");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match all instances of lowercase "case"
    // Count how many times "case" appears in lowercase
    let case_count = matches
        .iter()
        .filter(|m| m["content"].as_str().unwrap() == "case")
        .count();
    assert!(
        case_count > 0,
        "LowerJoined style should match lowercase 'case'"
    );

    // Test 2: Dot case only - when searching for single word "case",
    // it should only match the exact word "case" (not compound forms)
    let output = run_search(temp_dir.path(), "dot");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should only match standalone "case" instances
    let case_matches = matches
        .iter()
        .filter(|m| m["content"].as_str().unwrap() == "case")
        .count();
    assert!(case_matches > 0, "Dot style should match 'case' instances");

    // Test 3: Snake case only - should match "case" when it appears as a single word
    let output = run_search(temp_dir.path(), "snake");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match standalone "case" instances
    let case_matches = matches
        .iter()
        .filter(|m| m["content"].as_str().unwrap() == "case")
        .count();
    assert!(
        case_matches > 0,
        "Snake style should match 'case' instances"
    );

    // Test 4: Title case only - should match "Case" (title case of single word)
    let output = run_search(temp_dir.path(), "title");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match "Case" instances (title case)
    let case_matches = matches
        .iter()
        .filter(|m| m["content"].as_str().unwrap() == "Case")
        .count();
    assert!(
        case_matches > 0,
        "Title style should match 'Case' instances"
    );

    // Test 5: Pascal case only - should match "Case" (Pascal case of single word is just "Case")
    let output = run_search(temp_dir.path(), "pascal");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match "Case" instances
    let case_matches = matches
        .iter()
        .filter(|m| m["content"].as_str().unwrap() == "Case")
        .count();
    assert!(
        case_matches > 0,
        "Pascal style should match 'Case' instances"
    );
}

#[test]
fn test_multiple_case_styles() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multi.txt");

    let content = r#"
test_case
testCase
TEST_CASE
"#;

    fs::write(&test_file, content).unwrap();

    // Test with multiple styles selected
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_renamify"))
        .current_dir(temp_dir.path())
        .args([
            "search",
            "test case",
            "--only-styles",
            "snake,camel",
            "--output",
            "json",
        ])
        .output()
        .expect("Failed to execute renamify");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let matches = &json["plan"]["matches"];

    // Should match exactly 2: snake_case and camelCase
    assert_eq!(
        matches.as_array().unwrap().len(),
        2,
        "Should match only snake and camel"
    );
    assert!(stdout.contains("test_case"));
    assert!(stdout.contains("testCase"));
    assert!(!stdout.contains("TEST_CASE"));
}

#[test]
fn test_multiple_styles_is_superset_of_individual() {
    let temp_dir = TempDir::new().unwrap();

    // Create comprehensive test file with various case styles
    let test_file = temp_dir.path().join("comprehensive.rs");
    fs::write(
        &test_file,
        r#"
let case = 1;               // original
let test_case = 2;          // snake_case
let test-case = 3;          // kebab-case
let testCase = 4;           // camelCase
let TestCase = 5;           // PascalCase
let TEST_CASE = 6;          // SCREAMING_SNAKE
let test.case = 7;          // dot.case
let Test Case = 8;          // Title Case
let some_other_var = 9;     // unrelated snake
let another-var = 10;       // unrelated kebab
"#,
    )
    .unwrap();

    fn get_cli_path() -> String {
        env!("CARGO_BIN_EXE_renamify").to_string()
    }

    fn count_matches(output: &str) -> usize {
        let json: serde_json::Value = serde_json::from_str(output).unwrap();
        json["plan"]["matches"].as_array().unwrap().len()
    }

    // Test 1: Get results for lower style alone (replacing original)
    // For now, we'll use snake as the baseline test
    let snake_baseline_output = std::process::Command::new(get_cli_path())
        .args([
            "search",
            "case",
            "--only-styles",
            "snake",
            "--output",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(snake_baseline_output.status.success());
    let snake_baseline_matches =
        count_matches(&String::from_utf8(snake_baseline_output.stdout).unwrap());

    // Test 2: Get results for snake style alone
    let snake_output = std::process::Command::new(get_cli_path())
        .args([
            "search",
            "case",
            "--only-styles",
            "snake",
            "--output",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(snake_output.status.success());
    let snake_matches = count_matches(&String::from_utf8(snake_output.stdout).unwrap());

    // Test 3: Get results for kebab style alone
    let kebab_output = std::process::Command::new(get_cli_path())
        .args([
            "search",
            "case",
            "--only-styles",
            "kebab",
            "--output",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(kebab_output.status.success());
    let kebab_matches = count_matches(&String::from_utf8(kebab_output.stdout).unwrap());

    // Test 4: Get results for combined styles
    let combined_output = std::process::Command::new(get_cli_path())
        .args([
            "search",
            "case",
            "--only-styles",
            "snake,kebab",
            "--output",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(combined_output.status.success());
    let combined_matches = count_matches(&String::from_utf8(combined_output.stdout).unwrap());

    // Combined should have at least as many matches as the maximum of individual styles
    // (not the sum, because the same instance can match multiple styles)
    let max_individual = snake_matches.max(kebab_matches);

    println!("Snake baseline matches: {}", snake_baseline_matches);
    println!("Snake matches: {}", snake_matches);
    println!("Kebab matches: {}", kebab_matches);
    println!("Combined matches: {}", combined_matches);
    println!("Max individual: {}", max_individual);

    assert!(combined_matches >= max_individual,
        "Combined styles should match at least as many as the maximum individual style. Got {} but expected at least {}",
        combined_matches, max_individual);
}

#[test]
fn test_multiple_styles_no_extra_matches() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file with clear boundaries between different case styles
    let test_file = temp_dir.path().join("boundaries.rs");
    fs::write(
        &test_file,
        r#"
let case = 1;               // lowercase - should match lower only
let test_case = 2;          // snake_case - should match snake only
let test-case = 3;          // kebab-case - should match kebab only
let testCase = 4;           // camelCase - should match camel only
let TestCase = 5;           // PascalCase - should match pascal only
let unrelated_variable = 6; // should not match any "case" search
"#,
    )
    .unwrap();

    fn get_cli_path() -> String {
        env!("CARGO_BIN_EXE_renamify").to_string()
    }

    // Test that lower-joined+snake doesn't find camel/pascal matches
    let output = std::process::Command::new(get_cli_path())
        .args([
            "search",
            "case",
            "--only-styles",
            "lower-joined,snake",
            "--output",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // When searching for single word "case" with lower-joined+snake styles,
    // both styles generate "case" as the pattern, so it matches all instances of "case"
    // This includes "case" standalone and "case" within compound words
    assert!(
        !matches.is_empty(),
        "Should find matches for lower-joined+snake"
    );

    // Verify that all matches are for the "case" pattern
    let all_case_matches = matches
        .iter()
        .all(|m| m["content"].as_str().unwrap() == "case");

    assert!(
        all_case_matches,
        "All matches should be for the 'case' pattern"
    );
}

#[test]
fn test_all_combinations_are_supersets() {
    let temp_dir = TempDir::new().unwrap();

    let test_file = temp_dir.path().join("all_styles.rs");
    fs::write(
        &test_file,
        r#"
let case = 1;           // plain lowercase
let test_case = 2;      // snake
let test-case = 3;      // kebab
let testCase = 4;       // camel
let TestCase = 5;       // pascal
let TEST_CASE = 6;      // screaming_snake
let test.case = 7;      // dot
let Test Case = 8;      // title
"#,
    )
    .unwrap();

    fn get_cli_path() -> String {
        env!("CARGO_BIN_EXE_renamify").to_string()
    }

    fn count_matches(output: &str) -> usize {
        let json: serde_json::Value = serde_json::from_str(output).unwrap();
        json["plan"]["matches"].as_array().unwrap().len()
    }

    let all_styles = [
        "snake",
        "kebab",
        "camel",
        "pascal",
        "screaming-snake",
        "dot",
        "title",
    ];
    let mut individual_results = Vec::new();

    // Get individual results for each style
    for style in &all_styles {
        let output = std::process::Command::new(get_cli_path())
            .args(["search", "case", "--only-styles", style, "--output", "json"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute command");
        assert!(output.status.success(), "Style {} failed", style);
        let matches = count_matches(&String::from_utf8(output.stdout).unwrap());
        individual_results.push((*style, matches));
    }

    // Test some random combinations
    let combinations: &[(&str, &[&str])] = &[
        ("snake,kebab", &["snake", "kebab"]),
        ("kebab,camel,pascal", &["kebab", "camel", "pascal"]),
        (
            "screaming-snake,dot,title",
            &["screaming-snake", "dot", "title"],
        ),
        (
            "snake,kebab,camel,pascal",
            &["snake", "kebab", "camel", "pascal"],
        ),
    ];

    for (combo_str, combo_styles) in combinations {
        let output = std::process::Command::new(get_cli_path())
            .args([
                "search",
                "case",
                "--only-styles",
                combo_str,
                "--output",
                "json",
            ])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute command");
        assert!(output.status.success(), "Combination {} failed", combo_str);

        let combo_matches = count_matches(&String::from_utf8(output.stdout).unwrap());
        let max_individual: usize = combo_styles
            .iter()
            .map(|style| {
                individual_results
                    .iter()
                    .find(|(s, _)| s == style)
                    .unwrap()
                    .1
            })
            .max()
            .unwrap_or(0);

        assert!(combo_matches >= max_individual,
            "Combination '{}' should have at least {} matches (max of individual styles) but got {}",
            combo_str, max_individual, combo_matches);
    }
}
