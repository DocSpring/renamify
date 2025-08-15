use renamify_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_coercion_preserves_case_after_slash() {
    // Test that "DocSpring/oldproject" becomes "DocSpring/newproject" not "DocSpring/Newproject"
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.txt");
    std::fs::write(
        &test_file,
        r#"DocSpring/oldproject
github.com/DocSpring/oldproject
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after slash
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after slash separator"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_parenthesis() {
    // Test that "DocSpring(oldproject)" becomes "DocSpring(newproject)" not "DocSpring(Newproject)"
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.txt");
    std::fs::write(
        &test_file,
        r#"DocSpring(oldproject)
GetInstance(oldproject)
MACRO(oldproject)
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after parenthesis
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after parenthesis"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_bracket() {
    // Test that "DocSpring[oldproject]" becomes "DocSpring[newproject]" not "DocSpring[Newproject]"
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.txt");
    std::fs::write(
        &test_file,
        r#"DocSpring[oldproject]
Config[oldproject]
SETTINGS[oldproject]
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after bracket
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after bracket"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_colon() {
    // Test that "DocSpring:oldproject" becomes "DocSpring:newproject" not "DocSpring:Newproject"
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.txt");
    std::fs::write(
        &test_file,
        r#"DocSpring:oldproject
namespace:oldproject
Module:oldproject
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after colon
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after colon"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_at_sign() {
    // Test that "@docspring/oldproject" becomes "@docspring/newproject"
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("package.json");
    std::fs::write(
        &test_file,
        r#"{
  "dependencies": {
    "@docspring/oldproject": "^1.0.0",
    "@MyCompany/oldproject": "^2.0.0"
  }
}"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after slash in npm scope
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' in npm scope"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_dot() {
    // Test that "DocSpring.oldproject" becomes "DocSpring.newproject" not "DocSpring.Newproject"
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("test.java");
    std::fs::write(
        &test_file,
        r#"import com.DocSpring.oldproject;
import org.Example.oldproject;
System.oldproject.init();
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after dot
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after dot separator"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_equals() {
    // Test that "PROJECT=oldproject" and "Project=oldproject" preserve case
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("config.txt");
    std::fs::write(
        &test_file,
        r#"PROJECT=oldproject
Project=oldproject
project=oldproject
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after equals
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after equals sign"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_after_space() {
    // Test that "DocSpring oldproject" preserves case
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("comments.txt");
    std::fs::write(
        &test_file,
        r#"# DocSpring oldproject configuration
// Initialize oldproject here
/* Using oldproject library */
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase after space
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' after space"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_with_quotes() {
    // Test various quote contexts
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("strings.txt");
    std::fs::write(
        &test_file,
        r#""DocSpring/oldproject"
'DocSpring/oldproject'
`DocSpring/oldproject`
"MyClass:oldproject"
'Factory(oldproject)'
"Config[oldproject]"
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase in all contexts
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' in quoted strings"
            );
        }
    }
}

#[test]
fn test_coercion_preserves_case_with_curly_braces() {
    // Test that "${oldproject}" and "{oldproject}" preserve case
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("template.txt");
    std::fs::write(
        &test_file,
        r#"${oldproject}
{oldproject}
{{oldproject}}
MyClass{oldproject}
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // Should preserve lowercase in template contexts
    for m in &plan.matches {
        if m.before == "oldproject" {
            assert_eq!(
                m.after, "newproject",
                "Should be 'newproject' not 'Newproject' in template/brace contexts"
            );
        }
    }
}

#[test]
fn test_coercion_applies_correctly_for_compound_identifiers() {
    // Test that coercion DOES apply when it's genuinely part of a compound identifier
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let test_file = root.join("code.rs");
    std::fs::write(
        &test_file,
        r#"let oldproject_config = Config::new();
let OldprojectManager = Manager::new();
const OLDPROJECT_VERSION = "1.0";
"#,
    )
    .unwrap();

    let options = PlanOptions {
        exclude_match: vec![],
        no_acronyms: false,
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

    let plan = scan_repository(&root, "oldproject", "newproject", &options).unwrap();

    // These should apply coercion appropriately
    for m in &plan.matches {
        match m.before.as_str() {
            "oldproject_config" => assert_eq!(m.after, "newproject_config"),
            "OldprojectManager" => assert_eq!(m.after, "NewprojectManager"),
            "OLDPROJECT_VERSION" => assert_eq!(m.after, "NEWPROJECT_VERSION"),
            _ => {},
        }
    }
}
