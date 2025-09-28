use renamify_core::{scan_repository, PlanOptions};
use std::time::Instant;
use tempfile::TempDir;

#[test]
fn test_tokenization_performance() {
    use renamify_core::acronym::AcronymSet;
    use renamify_core::case_model::parse_to_tokens_with_acronyms;

    let acronym_set = AcronymSet::default();

    // Test various inputs
    let test_cases = vec![
        "XMLHttpRequest",
        "getUserIDFromDatabase",
        "APIClientForHTTPSConnections",
        "simpleSnakeCaseIdentifier",
        "CamelCaseWithXMLAndHTTPAcronyms",
        // Long string to stress test
        "ThisIsAVeryLongIdentifierWithManyWordsAndSomeACRONYMSLikeHTTPAndXMLAndAPIAndIDAndUIAndMoreWordsToMakeItEvenLonger",
    ];

    let start = Instant::now();

    // Run tokenization many times to get measurable timing
    for _ in 0..1000 {
        for test_case in &test_cases {
            let _ = parse_to_tokens_with_acronyms(test_case, &acronym_set);
        }
    }

    let duration = start.elapsed();

    // Helpful for diagnosing any future regression when running with `--nocapture`.
    eprintln!("tokenization benchmark duration: {:?}", duration);

    // Windows CI runners are noticeably slower (~140ms) despite no regression, so keep a
    // conservative guardrail that still flags real slowdowns while avoiding flaky failures.
    // A genuine regression (e.g. inadvertent O(n^2) behaviour) will comfortably exceed 250ms.
    assert!(
        duration.as_millis() < 250,
        "Tokenization took {:?}, expected < 250ms. Performance regression detected!",
        duration
    );
}

#[test]
fn test_scan_performance_small_repo() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create 100 small files
    for i in 0..100 {
        let file = root.join(format!("file{}.rs", i));
        std::fs::write(
            &file,
            "fn oldName() { let old_name = 42; OldName::new(); OLD_NAME }\n".repeat(10),
        )
        .unwrap();
    }

    let options = PlanOptions::default();

    let start = Instant::now();
    let _plan = scan_repository(&root, "oldName", "newName", &options).unwrap();
    let duration = start.elapsed();

    // Scanning 100 small files should take less than 1 second
    assert!(
        duration.as_secs() < 1,
        "Scanning 100 files took {:?}, expected < 1s. Performance regression detected!",
        duration
    );
}

#[test]
#[ignore] // Run with: cargo test --ignored test_scan_performance_large_file
fn test_scan_performance_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create one large file (1MB)
    let file = root.join("large.rs");
    let line = "fn oldName() { let old_name = 42; OldName::new(); OLD_NAME }\n";
    let content = line.repeat(16_000); // ~1MB
    std::fs::write(&file, content).unwrap();

    let options = PlanOptions::default();

    let start = Instant::now();
    let _plan = scan_repository(&root, "oldName", "newName", &options).unwrap();
    let duration = start.elapsed();

    // Scanning a 1MB file should take less than 500ms
    assert!(
        duration.as_millis() < 500,
        "Scanning 1MB file took {:?}, expected < 500ms. Performance regression detected!",
        duration
    );
}
