use refaktor_core::{scan_repository, PlanOptions};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_compound_replacement_at_start() {
    // Pattern at the beginning of compound word
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    let test_file = root.join("test.rs");
    std::fs::write(&test_file, 
        r#"// Snake case
let preview_format_arg = 1;
let preview_format_option = 2;

// Camel case  
let previewFormatArg = 3;
let previewFormatOption = 4;

// Pascal case
type PreviewFormatArg = String;
type PreviewFormatOption = i32;"#
    ).unwrap();
    
    let options = PlanOptions {
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![
            refaktor_core::Style::Snake,
            refaktor_core::Style::Camel,
            refaktor_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound at Start Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }
    
    // Should replace:
    // preview_format_arg -> preview_arg
    // preview_format_option -> preview_option
    // previewFormatArg -> previewArg
    // previewFormatOption -> previewOption
    // PreviewFormatArg -> PreviewArg
    // PreviewFormatOption -> PreviewOption
    
    assert_eq!(plan.stats.total_matches, 6, "Should find all compounds starting with pattern");
    
    // Verify replacements
    let expected = vec![
        ("preview_format_arg", "preview_arg"),
        ("preview_format_option", "preview_option"),
        ("previewFormatArg", "previewArg"),
        ("previewFormatOption", "previewOption"),
        ("PreviewFormatArg", "PreviewArg"),
        ("PreviewFormatOption", "PreviewOption"),
    ];
    
    for (from, to) in expected {
        let found = plan.matches.iter()
            .any(|h| h.before == from && h.after == to);
        assert!(found, "Should replace {} with {}", from, to);
    }
}

#[test]
fn test_compound_replacement_in_middle() {
    // Pattern in the middle of compound word
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    let test_file = root.join("test.rs");
    std::fs::write(&test_file, 
        r#"// Snake case
let should_preview_format_please = 1;
let get_preview_format_option = 2;

// Camel case  
let shouldPreviewFormatPlease = 3;
let getPreviewFormatOption = 4;

// Pascal case
type ShouldPreviewFormatPlease = String;
type GetPreviewFormatOption = i32;"#
    ).unwrap();
    
    let options = PlanOptions {
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![
            refaktor_core::Style::Snake,
            refaktor_core::Style::Camel,
            refaktor_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound in Middle Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }
    
    // Should replace:
    // should_preview_format_please -> should_preview_please
    // get_preview_format_option -> get_preview_option
    // shouldPreviewFormatPlease -> shouldPreviewPlease
    // getPreviewFormatOption -> getPreviewOption
    // ShouldPreviewFormatPlease -> ShouldPreviewPlease
    // GetPreviewFormatOption -> GetPreviewOption
    
    assert_eq!(plan.stats.total_matches, 6, "Should find all compounds with pattern in middle");
    
    // Verify replacements preserve prefix and suffix
    let expected = vec![
        ("should_preview_format_please", "should_preview_please"),
        ("get_preview_format_option", "get_preview_option"),
        ("shouldPreviewFormatPlease", "shouldPreviewPlease"),
        ("getPreviewFormatOption", "getPreviewOption"),
        ("ShouldPreviewFormatPlease", "ShouldPreviewPlease"),
        ("GetPreviewFormatOption", "GetPreviewOption"),
    ];
    
    for (from, to) in expected {
        let found = plan.matches.iter()
            .any(|h| h.before == from && h.after == to);
        assert!(found, "Should replace {} with {}", from, to);
    }
}

#[test]
fn test_compound_replacement_at_end() {
    // Pattern at the end of compound word
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    let test_file = root.join("test.rs");
    std::fs::write(&test_file, 
        r#"// Snake case
let get_preview_format = 1;
let load_preview_format = 2;

// Camel case  
let getPreviewFormat = 3;
let loadPreviewFormat = 4;

// Pascal case
type GetPreviewFormat = String;
type LoadPreviewFormat = i32;"#
    ).unwrap();
    
    let options = PlanOptions {
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![
            refaktor_core::Style::Snake,
            refaktor_core::Style::Camel,
            refaktor_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Compound at End Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }
    
    // Should replace:
    // get_preview_format -> get_preview
    // load_preview_format -> load_preview
    // getPreviewFormat -> getPreview
    // loadPreviewFormat -> loadPreview
    // GetPreviewFormat -> GetPreview
    // LoadPreviewFormat -> LoadPreview
    
    assert_eq!(plan.stats.total_matches, 6, "Should find all compounds ending with pattern");
    
    // Verify replacements preserve prefix
    let expected = vec![
        ("get_preview_format", "get_preview"),
        ("load_preview_format", "load_preview"),
        ("getPreviewFormat", "getPreview"),
        ("loadPreviewFormat", "loadPreview"),
        ("GetPreviewFormat", "GetPreview"),
        ("LoadPreviewFormat", "LoadPreview"),
    ];
    
    for (from, to) in expected {
        let found = plan.matches.iter()
            .any(|h| h.before == from && h.after == to);
        assert!(found, "Should replace {} with {}", from, to);
    }
}

#[test]
fn test_exact_match_not_compound() {
    // Should still match exact occurrences that aren't compounds
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    let test_file = root.join("test.rs");
    std::fs::write(&test_file, 
        r#"let preview_format = get_preview_format();
let PreviewFormat = PreviewFormat::new();
let previewFormat = getPreviewFormat();"#
    ).unwrap();
    
    let options = PlanOptions {
        includes: vec![],
        excludes: vec![],
        respect_gitignore: false,
        unrestricted_level: 0,
        styles: Some(vec![
            refaktor_core::Style::Snake,
            refaktor_core::Style::Camel,
            refaktor_core::Style::Pascal,
        ]),
        rename_files: false,
        rename_dirs: false,
        rename_root: false,
        plan_out: PathBuf::from("plan.json"),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto,
    };
    
    let plan = scan_repository(&root, "preview_format", "preview", &options).unwrap();
    
    println!("\n=== Exact Match Test ===");
    for hunk in &plan.matches {
        println!("Line {}: '{}' -> '{}'", hunk.line, hunk.before, hunk.after);
    }
    
    // Should find both exact matches AND compounds
    // Line 1: preview_format (exact), get_preview_format (compound)
    // Line 2: PreviewFormat twice (exact)
    // Line 3: previewFormat (exact), getPreviewFormat (compound)
    
    assert_eq!(plan.stats.total_matches, 6, "Should find both exact and compound matches");
}