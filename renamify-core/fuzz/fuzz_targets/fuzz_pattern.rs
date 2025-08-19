#![no_main]

use libfuzzer_sys::fuzz_target;
use renamify_core::pattern::{build_pattern, find_matches};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let input = String::from_utf8_lossy(data);
    let lines: Vec<String> = input
        .lines()
        .take(20)
        .map(|s| s.chars().take(100).collect())
        .collect();

    if lines.is_empty() {
        return;
    }

    if let Ok(pattern) = build_pattern(&lines) {
        let test_content = lines.join(" ").into_bytes();
        let _ = find_matches(&pattern, &test_content, "fuzz.txt");

        for variant in &lines {
            let _ = pattern.identify_variant(variant.as_bytes());
        }
    }
});
