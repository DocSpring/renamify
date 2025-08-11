use refaktor_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_compound_pascal_case_replacement() {
    // This test demonstrates the EXPECTED behavior for compound word replacements
    // When replacing "preview_format" with "preview", compound words like
    // "PreviewFormatArg" should become "PreviewArg"
    
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    // Create test file with compound Pascal case identifiers
    let test_file = root.join("main.rs");
    std::fs::write(&test_file, 
        r#"struct PreviewFormatArg { }
impl From<PreviewFormatArg> for PreviewFormat {
    fn from(arg: PreviewFormatArg) -> PreviewFormat {
        match arg {
            PreviewFormatArg::Table => PreviewFormat::Table,
            PreviewFormatArg::Diff => PreviewFormat::Diff,
        }
    }
}
struct ShouldReplacePreviewFormatPlease { }
fn getPreviewFormatOption() -> PreviewFormatOption { }"#
    ).unwrap();
    
    let options = PlanOptions { exclude_match: vec![], 
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![refaktor_core::Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound Pascal Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Line {}, Col {}: '{}' -> '{}'", 
                 hunk.line, hunk.col, hunk.before, hunk.after);
        if let Some(line_after) = &hunk.line_after {
            println!("  After: {}", line_after);
        }
    }
    
    // Should find:
    // Line 1: PreviewFormatArg -> PreviewArg
    // Line 2: PreviewFormatArg -> PreviewArg, PreviewFormat -> Preview  
    // Line 3: PreviewFormatArg -> PreviewArg, PreviewFormat -> Preview
    // Line 5: PreviewFormatArg -> PreviewArg (twice), PreviewFormat -> Preview (twice)
    // Line 6: PreviewFormatArg -> PreviewArg, PreviewFormat -> Preview
    // Line 9: ShouldReplacePreviewFormatPlease -> ShouldReplacePreviewPlease
    // Line 10: PreviewFormatOption -> PreviewOption (Pascal only, not getPreviewFormatOption)
    
    // Total: 11 replacements (Pascal only)
    assert_eq!(plan.stats.total_matches, 11, 
               "Should find all compound Pascal case variants");
    
    // Verify PreviewFormatArg is replaced with PreviewArg
    let preview_format_arg_replacements: Vec<_> = plan.matches.iter()
        .filter(|h| h.before == "PreviewFormatArg")
        .collect();
    
    assert!(!preview_format_arg_replacements.is_empty(), 
            "Should find PreviewFormatArg occurrences");
    
    for hunk in &preview_format_arg_replacements {
        assert_eq!(hunk.after, "PreviewArg", 
                   "PreviewFormatArg should be replaced with PreviewArg");
    }
}

#[test]
fn test_compound_snake_case_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    // Test with snake_case compounds
    let test_file = root.join("main.rs");
    std::fs::write(&test_file, 
        r#"let preview_format_arg = get_preview_format_arg();
let preview_format_option = preview_format_arg.to_option();
match preview_format_type {
    PreviewFormatType::Json => preview_format_json(),
}"#
    ).unwrap();
    
    let options = PlanOptions { exclude_match: vec![], 
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![refaktor_core::Style::Snake, refaktor_core::Style::Pascal]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound Snake Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }
    
    // Should find and replace:
    // preview_format_arg -> preview_arg
    // preview_format_option -> preview_option  
    // preview_format_type -> preview_type
    // PreviewFormatType -> PreviewType
    // preview_format_json -> preview_json
    
    let snake_compounds = vec!["preview_format_arg", "preview_format_option", "preview_format_type", "preview_format_json"];
    for compound in &snake_compounds {
        let replacements: Vec<_> = plan.matches.iter()
            .filter(|h| h.before == *compound)
            .collect();
        assert!(!replacements.is_empty(), "Should find {}", compound);
        
        let expected = compound.replace("preview_format", "preview");
        for hunk in &replacements {
            assert_eq!(hunk.after, expected, "{} should become {}", compound, expected);
        }
    }
}

#[test]
fn test_compound_camel_case_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    // Test with camelCase compounds
    let test_file = root.join("main.js");
    std::fs::write(&test_file, 
        r#"const previewFormatArg = getPreviewFormatArg();
const previewFormatOption = previewFormatArg.toOption();
function setPreviewFormatType(previewFormatType) {
    this.previewFormatType = previewFormatType;
}"#
    ).unwrap();
    
    let options = PlanOptions { exclude_match: vec![], 
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![refaktor_core::Style::Camel]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound Camel Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }
    
    // Should find and replace:
    // previewFormatArg -> previewArg (2 times on lines 1 and 2)
    // getPreviewFormatArg -> getPreviewArg (line 1)
    // previewFormatOption -> previewOption (line 2)
    // setPreviewFormatType -> setPreviewType (line 3)
    // previewFormatType -> previewType (3 times on lines 3 and 4)
    
    assert_eq!(plan.stats.total_matches, 8, "Should find all camelCase compounds");
    
    // Verify camelCase compounds are properly replaced
    let camel_compounds = vec![
        ("previewFormatArg", "previewArg"),
        ("previewFormatOption", "previewOption"),
        ("previewFormatType", "previewType"),
    ];
    
    for (from, to) in &camel_compounds {
        let replacements: Vec<_> = plan.matches.iter()
            .filter(|h| h.before == *from)
            .collect();
        assert!(!replacements.is_empty(), "Should find {}", from);
        
        for hunk in &replacements {
            assert_eq!(hunk.after, *to, "{} should become {}", from, to);
        }
    }
}

#[test]
fn test_compound_pascal_and_camel_case_replacement() {
    // Same test as pascal-only but with both styles enabled
    // This should find MORE matches including getPreviewFormatOption
    
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    // Same test file as Pascal test
    let test_file = root.join("main.rs");
    std::fs::write(&test_file, 
        r#"struct PreviewFormatArg { }
impl From<PreviewFormatArg> for PreviewFormat {
    fn from(arg: PreviewFormatArg) -> PreviewFormat {
        match arg {
            PreviewFormatArg::Table => PreviewFormat::Table,
            PreviewFormatArg::Diff => PreviewFormat::Diff,
        }
    }
}
struct ShouldReplacePreviewFormatPlease { }
fn getPreviewFormatOption() -> PreviewFormatOption { }"#
    ).unwrap();
    
    let options = PlanOptions { exclude_match: vec![], 
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![refaktor_core::Style::Pascal, refaktor_core::Style::Camel]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound Pascal + Camel Case Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Line {}, Col {}: '{}' -> '{}'", 
                 hunk.line, hunk.col, hunk.before, hunk.after);
    }
    
    // Should find:
    // Line 1: PreviewFormatArg -> PreviewArg
    // Line 2: PreviewFormatArg -> PreviewArg, PreviewFormat -> Preview  
    // Line 3: PreviewFormatArg -> PreviewArg, PreviewFormat -> Preview
    // Line 5: PreviewFormatArg -> PreviewArg (twice), PreviewFormat -> Preview (twice)
    // Line 6: PreviewFormatArg -> PreviewArg, PreviewFormat -> Preview
    // Line 9: ShouldReplacePreviewFormatPlease -> ShouldReplacePreviewPlease
    // Line 10: getPreviewFormatOption -> getPreviewOption (ADDITIONAL because Camel is included)
    // Line 10: PreviewFormatOption -> PreviewOption
    
    // Total: 12 replacements (one more than Pascal-only)
    assert_eq!(plan.stats.total_matches, 12, 
               "Should find all compound Pascal AND Camel case variants");
    
    // Verify we found the camelCase function name
    let camel_match = plan.matches.iter()
        .find(|h| h.before == "getPreviewFormatOption");
    assert!(camel_match.is_some(), "Should find getPreviewFormatOption when Camel style is included");
}

#[test]
fn test_multiple_compounds_same_line() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    // Test multiple compound words on the same line
    let test_file = root.join("main.rs");
    std::fs::write(&test_file, 
        "fn convert(preview_format_arg: PreviewFormatArg) -> PreviewFormatOption { }\n"
    ).unwrap();
    
    let options = PlanOptions { exclude_match: vec![], 
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![
            refaktor_core::Style::Snake,
            refaktor_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Multiple Compounds Same Line Test ===");
    println!("Total matches: {}", plan.stats.total_matches);
    for hunk in &plan.matches {
        println!("Col {}: '{}' -> '{}'", hunk.col, hunk.before, hunk.after);
    }
    
    // Should find:
    // preview_format_arg -> preview_arg
    // PreviewFormatArg -> PreviewArg
    // PreviewFormatOption -> PreviewOption
    assert_eq!(plan.stats.total_matches, 3, 
               "Should find all three compound variants on the same line");
    
    // Verify all are on line 1 but different columns
    for hunk in &plan.matches {
        assert_eq!(hunk.line, 1);
    }
    
    let columns: Vec<u32> = plan.matches.iter().map(|h| h.col).collect();
    assert_eq!(columns.len(), 3);
    assert!(columns[0] < columns[1] && columns[1] < columns[2], 
            "Matches should be at different column positions");
}