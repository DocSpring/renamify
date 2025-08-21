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
            .args(&[
                "search",
                "test case",  // Search for "test case" in various forms
                "--only-styles",
                styles,
                "--output",
                "json",
            ])
            .output()
            .expect("Failed to execute renamify");

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    // Test 1: Original only - should ONLY match "test case" exactly
    let output = run_search(temp_dir.path(), "original");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "Original should only match exact case");
    assert_eq!(matches[0]["content"].as_str().unwrap(), "test case");

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
    assert_eq!(matches.len(), 1, "Screaming snake should only match TEST_CASE");
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
            .args(&[
                "search",
                "case",  // Search for single word "case"
                "--only-styles",
                styles,
                "--output",
                "json",
            ])
            .output()
            .expect("Failed to execute renamify");

        if !output.status.success() {
            eprintln!("Command failed with stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    // Test 1: Original only - should ONLY match "case" exactly (lowercase, standalone)
    let output = run_search(temp_dir.path(), "original");
    println!("Output for original: {:?}", output);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should find exactly 4 standalone instances of lowercase "case" (including one in a comment)
    assert_eq!(matches.len(), 4, "Original should find 4 standalone instances of lowercase 'case'");
    for match_item in matches {
        assert_eq!(match_item["content"].as_str().unwrap(), "case");
    }

    // Test 2: Dot case only - should match all dot.case patterns
    let output = run_search(temp_dir.path(), "dot");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match: test.case, use.case, brief.case
    assert_eq!(matches.len(), 3, "Dot style should match 3 dot.case patterns");
    let expected_dot = vec!["test.case", "use.case", "brief.case"];
    for (i, match_item) in matches.iter().enumerate() {
        assert_eq!(match_item["content"].as_str().unwrap(), expected_dot[i]);
    }

    // Test 3: Snake case only - should match all snake_case patterns
    let output = run_search(temp_dir.path(), "snake");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match: test_case, use_case, brief_case
    assert_eq!(matches.len(), 3, "Snake should match 3 snake_case patterns");
    let expected_snake = vec!["test_case", "use_case", "brief_case"];
    for (i, match_item) in matches.iter().enumerate() {
        assert_eq!(match_item["content"].as_str().unwrap(), expected_snake[i]);
    }

    // Test 4: Title case only - should match all "Title Case" patterns
    let output = run_search(temp_dir.path(), "title");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match: Test Case, Use Case, Brief Case
    assert_eq!(matches.len(), 3, "Title should match 3 Title Case patterns");
    let expected_title = vec!["Test Case", "Use Case", "Brief Case"];
    for (i, match_item) in matches.iter().enumerate() {
        assert_eq!(match_item["content"].as_str().unwrap(), expected_title[i]);
    }

    // Test 5: Pascal case only - should match PascalCase patterns
    let output = run_search(temp_dir.path(), "pascal");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();

    // Should match: TestCase, UseCase, BriefCase
    assert_eq!(matches.len(), 3, "Pascal should match 3 PascalCase patterns");
    let expected_pascal = vec!["TestCase", "UseCase", "BriefCase"];
    for (i, match_item) in matches.iter().enumerate() {
        assert_eq!(match_item["content"].as_str().unwrap(), expected_pascal[i]);
    }
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
        .args(&[
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
    assert_eq!(matches.as_array().unwrap().len(), 2, "Should match only snake and camel");
    assert!(stdout.contains("test_case"));
    assert!(stdout.contains("testCase"));
    assert!(!stdout.contains("TEST_CASE"));
}

#[test]
fn test_multiple_styles_is_superset_of_individual() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create comprehensive test file with various case styles
    let test_file = temp_dir.path().join("comprehensive.rs");
    fs::write(&test_file, r#"
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
"#).unwrap();

    fn get_cli_path() -> String {
        env!("CARGO_BIN_EXE_renamify").to_string()
    }

    fn count_matches(output: &str) -> usize {
        let json: serde_json::Value = serde_json::from_str(output).unwrap();
        json["plan"]["matches"].as_array().unwrap().len()
    }

    // Test 1: Get results for original style alone
    let original_output = std::process::Command::new(&get_cli_path())
        .args(&["search", "case", "--only-styles", "original", "--output", "json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(original_output.status.success());
    let original_matches = count_matches(&String::from_utf8(original_output.stdout).unwrap());

    // Test 2: Get results for snake style alone  
    let snake_output = std::process::Command::new(&get_cli_path())
        .args(&["search", "case", "--only-styles", "snake", "--output", "json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(snake_output.status.success());
    let snake_matches = count_matches(&String::from_utf8(snake_output.stdout).unwrap());

    // Test 3: Get results for kebab style alone
    let kebab_output = std::process::Command::new(&get_cli_path())
        .args(&["search", "case", "--only-styles", "kebab", "--output", "json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(kebab_output.status.success());
    let kebab_matches = count_matches(&String::from_utf8(kebab_output.stdout).unwrap());

    // Test 4: Get results for combined styles
    let combined_output = std::process::Command::new(&get_cli_path())
        .args(&["search", "case", "--only-styles", "original,snake,kebab", "--output", "json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    assert!(combined_output.status.success());
    let combined_matches = count_matches(&String::from_utf8(combined_output.stdout).unwrap());

    // CRITICAL: Combined should equal sum of individual results (superset property)
    let expected_combined = original_matches + snake_matches + kebab_matches;
    
    println!("Original matches: {}", original_matches);
    println!("Snake matches: {}", snake_matches);  
    println!("Kebab matches: {}", kebab_matches);
    println!("Combined matches: {}", combined_matches);
    println!("Expected combined: {}", expected_combined);

    assert_eq!(combined_matches, expected_combined, 
        "Combined styles should be exact superset of individual styles. Got {} but expected {}. \
        This indicates the case style filtering is incorrectly finding additional matches when multiple styles are selected.",
        combined_matches, expected_combined);
}

#[test]
fn test_multiple_styles_no_extra_matches() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test file with clear boundaries between different case styles
    let test_file = temp_dir.path().join("boundaries.rs");
    fs::write(&test_file, r#"
let case = 1;               // original - should match original only
let test_case = 2;          // snake_case - should match snake only
let test-case = 3;          // kebab-case - should match kebab only
let testCase = 4;           // camelCase - should match camel only
let TestCase = 5;           // PascalCase - should match pascal only
let unrelated_variable = 6; // should not match any "case" search
"#).unwrap();

    fn get_cli_path() -> String {
        env!("CARGO_BIN_EXE_renamify").to_string()
    }

    // Test that original+snake doesn't find camel/pascal matches
    let output = std::process::Command::new(&get_cli_path())
        .args(&["search", "case", "--only-styles", "original,snake", "--output", "json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let matches = json["plan"]["matches"].as_array().unwrap();
    
    // Should find exactly 2 matches: original and snake
    assert_eq!(matches.len(), 2, "Should find exactly 2 matches for original+snake");
    
    // Verify the specific matches
    let match_contents: Vec<&str> = matches.iter()
        .map(|m| m["content"].as_str().unwrap())
        .collect();
    
    assert!(match_contents.contains(&"case"), "Should find original match");
    assert!(match_contents.contains(&"test_case"), "Should find snake_case match");
    
    // Should NOT find other styles
    assert!(!match_contents.contains(&"test-case"), "Should NOT find kebab-case match");
    assert!(!match_contents.contains(&"testCase"), "Should NOT find camelCase match");
    assert!(!match_contents.contains(&"TestCase"), "Should NOT find PascalCase match");
}

#[test] 
fn test_all_combinations_are_supersets() {
    let temp_dir = TempDir::new().unwrap();
    
    let test_file = temp_dir.path().join("all_styles.rs");
    fs::write(&test_file, r#"
let case = 1;           // original
let test_case = 2;      // snake
let test-case = 3;      // kebab  
let testCase = 4;       // camel
let TestCase = 5;       // pascal
let TEST_CASE = 6;      // screaming_snake
let test.case = 7;      // dot
let Test Case = 8;      // title
"#).unwrap();

    fn get_cli_path() -> String {
        env!("CARGO_BIN_EXE_renamify").to_string()
    }

    fn count_matches(output: &str) -> usize {
        let json: serde_json::Value = serde_json::from_str(output).unwrap();
        json["plan"]["matches"].as_array().unwrap().len()
    }

    let all_styles = ["original", "snake", "kebab", "camel", "pascal", "screaming-snake", "dot", "title"];
    let mut individual_results = Vec::new();
    
    // Get individual results for each style
    for style in &all_styles {
        let output = std::process::Command::new(&get_cli_path())
            .args(&["search", "case", "--only-styles", style, "--output", "json"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute command");
        assert!(output.status.success(), "Style {} failed", style);
        let matches = count_matches(&String::from_utf8(output.stdout).unwrap());
        individual_results.push((*style, matches));
    }

    // Test some random combinations
    let combinations: &[(&str, &[&str])] = &[
        ("original,snake", &["original", "snake"]),
        ("kebab,camel,pascal", &["kebab", "camel", "pascal"]),  
        ("original,screaming-snake,dot", &["original", "screaming-snake", "dot"]),
        ("snake,kebab,camel,pascal", &["snake", "kebab", "camel", "pascal"]),
    ];

    for (combo_str, combo_styles) in combinations {
        let output = std::process::Command::new(&get_cli_path())
            .args(&["search", "case", "--only-styles", combo_str, "--output", "json"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute command");
        assert!(output.status.success(), "Combination {} failed", combo_str);
        
        let combo_matches = count_matches(&String::from_utf8(output.stdout).unwrap());
        let expected_matches: usize = combo_styles.iter()
            .map(|style| individual_results.iter().find(|(s, _)| s == style).unwrap().1)
            .sum();

        assert_eq!(combo_matches, expected_matches,
            "Combination '{}' should have {} matches (sum of individual styles) but got {}",
            combo_str, expected_matches, combo_matches);
    }
}
