use renamify_core::operations::plan::plan_operation;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ambiguous_identifier_resolution_in_python() {
    // This test verifies that ambiguous identifiers get resolved based on context
    // The identifier "api" is ambiguous - could be snake_case, camelCase, etc.
    // We're renaming to a PascalCase replacement "ServiceHandler"
    // The resolver should adapt the replacement based on Python context

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a Python file with different contexts
    // "api" is ambiguous - could be snake, camel, kebab, lower
    // "API" is ambiguous - could be SCREAMING_SNAKE, Pascal, Upper
    let py_file = temp_path.join("service.py");
    fs::write(
        &py_file,
        r#"class API:
    """API service class"""
    pass

def api():
    """api function"""
    return None

api_key = "secret"
API_URL = "https://example.com"

# Using the API
my_api = api()
APIClass = API
"#,
    )
    .unwrap();

    // Plan to rename ambiguous "api" to PascalCase "ServiceHandler"
    // This will match both "api" (lowercase) and "API" (uppercase)
    let (plan_result, _) = plan_operation(
        "api",
        "ServiceHandler",
        vec![],          // empty = current dir
        vec![],          // includes
        vec![],          // excludes
        true,            // respect_gitignore
        0,               // unrestricted_level
        true,            // rename_files
        true,            // rename_dirs
        &[],             // exclude_styles
        &[],             // include_styles
        &[],             // only_styles
        vec![],          // exclude_match
        None,            // exclude_matching_lines
        None,            // plan_out
        None,            // preview_format
        true,            // dry_run
        false,           // fixed_table_width
        false,           // use_color
        false,           // no_acronyms
        vec![],          // include_acronyms
        vec![],          // exclude_acronyms
        vec![],          // only_acronyms
        false,           // ignore_ambiguous
        Some(temp_path), // working_dir
        None,            // atomic_config
    )
    .unwrap();

    let matches = &plan_result.plan.unwrap().matches;

    // Debug: print all matches to see what we're getting
    eprintln!("Found {} matches", matches.len());
    for m in matches {
        eprintln!(
            "  Line {}, Col {}: '{}' -> '{}'",
            m.line, m.char_offset, m.content, m.replace
        );
    }

    // Find the class definition - should be PascalCase
    let class_match = matches.iter()
        .find(|m| m.line == 1 && m.char_offset == 6)  // "class api" - char_offset is 0-based
        .expect("Should find class definition");

    // This SHOULD be "ServiceHandler" (PascalCase) if ambiguity resolver works
    // But will likely be "serviceHandler" (camelCase) with current broken implementation
    assert_eq!(
        class_match.replace, "ServiceHandler",
        "Class name should be PascalCase in Python, got: {}",
        class_match.replace
    );

    // Find the function definition - should be snake_case
    let func_match = matches.iter()
        .find(|m| m.line == 5 && m.char_offset == 4)  // "def api" - char_offset is 0-based
        .expect("Should find function definition");

    // This SHOULD be "service_handler" (snake_case) if ambiguity resolver works
    assert_eq!(
        func_match.replace, "service_handler",
        "Function name should be snake_case in Python, got: {}",
        func_match.replace
    );

    // Lowercase variable should be snake_case
    let var_match = matches
        .iter()
        .find(|m| m.line == 9 && m.char_offset == 0 && m.content == "api_key")
        .expect("Should find variable assignment");

    assert_eq!(
        var_match.replace, "service_handler_key",
        "Variable should be snake_case, got: {}",
        var_match.replace
    );

    // Uppercase constant should be SCREAMING_SNAKE_CASE
    let const_match = matches
        .iter()
        .find(|m| m.line == 10 && m.char_offset == 0 && m.content == "API_URL")
        .expect("Should find constant assignment");

    assert_eq!(
        const_match.replace, "SERVICE_HANDLER_URL",
        "Constant should be SCREAMING_SNAKE_CASE, got: {}",
        const_match.replace
    );
}

#[test]
fn test_ambiguous_identifier_resolution_in_javascript() {
    // Test JavaScript-specific resolution
    // "api" is ambiguous, renamed to "DataService"

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let js_file = temp_path.join("app.js");
    fs::write(
        &js_file,
        r#"class Api {
  constructor() {}
}

const api = new Api();
let apiUrl = 'https://example.com';
var api_key = process.env.API_KEY;

function api() {
  return fetch(apiUrl);
}

const API_TIMEOUT = 5000;
"#,
    )
    .unwrap();

    let (plan_result, _) = plan_operation(
        "api",
        "DataService",
        vec![],
        vec![],
        vec![],
        true,
        0,
        true,
        true,
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
        Some(temp_path),
        None,
    )
    .unwrap();

    let matches = &plan_result.plan.unwrap().matches;

    // Class should be PascalCase
    let class_match = matches
        .iter()
        .find(|m| m.line == 1 && m.content == "Api")
        .expect("Should find class");

    assert_eq!(
        class_match.replace, "DataService",
        "JS class should be PascalCase, got: {}",
        class_match.replace
    );

    // const variable should be camelCase
    let const_match = matches
        .iter()
        .find(|m| m.line == 5 && m.char_offset == 6)
        .expect("Should find const variable");

    assert_eq!(
        const_match.replace, "dataService",
        "JS const should be camelCase, got: {}",
        const_match.replace
    );
}

#[test]
fn test_ambiguous_replacement_uses_context() {
    // This tests when BOTH search and replace are ambiguous
    // "api" -> "backend" (both are ambiguous single words)

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let py_file = temp_path.join("models.py");
    fs::write(
        &py_file,
        r#"class Api:
    pass

def get_api():
    return api()
"#,
    )
    .unwrap();

    let (plan_result, _) = plan_operation(
        "api",
        "backend",
        vec![],
        vec![],
        vec![],
        true,
        0,
        true,
        true,
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
        Some(temp_path),
        None,
    )
    .unwrap();

    let matches = &plan_result.plan.unwrap().matches;

    // Class should become "Backend" (PascalCase)
    let class_match = matches
        .iter()
        .find(|m| m.line == 1)
        .expect("Should find class");

    assert_eq!(
        class_match.replace, "Backend",
        "Python class with ambiguous replacement should be PascalCase, got: {}",
        class_match.replace
    );

    // Function reference should be "backend" (snake_case)
    let func_ref = matches
        .iter()
        .find(|m| m.line == 5 && m.char_offset > 10)
        .expect("Should find function call");

    assert_eq!(
        func_ref.replace, "backend",
        "Function reference should be snake_case, got: {}",
        func_ref.replace
    );
}
