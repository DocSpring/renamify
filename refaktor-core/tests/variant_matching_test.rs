use refaktor_core::pattern::build_pattern;
use std::collections::BTreeMap;

#[test]
fn test_variant_matching_preserves_case_in_paths() {
    // This test ensures that when matching "oldproject" in a path context like
    // "DocSpring/oldproject", we correctly identify it as the lowercase variant
    // and replace it with the lowercase replacement "newproject", not "Newproject"

    // Create variant map as the scanner does
    let mut variant_map = BTreeMap::new();
    variant_map.insert("oldproject".to_string(), "newproject".to_string());
    variant_map.insert("Oldproject".to_string(), "Newproject".to_string());
    variant_map.insert("OLDPROJECT".to_string(), "NEWPROJECT".to_string());

    // Build pattern from variant keys
    let variants: Vec<String> = variant_map.keys().cloned().collect();
    let pattern = build_pattern(&variants).unwrap();

    // Test various path contexts
    let test_cases = vec![
        ("DocSpring/oldproject", "oldproject", "newproject"),
        (
            "github.com/DocSpring/oldproject",
            "oldproject",
            "newproject",
        ),
        ("REPO='DocSpring/oldproject'", "oldproject", "newproject"),
        ("path/to/Oldproject", "Oldproject", "Newproject"),
        ("OLDPROJECT", "OLDPROJECT", "NEWPROJECT"),
    ];

    for (input, expected_match, expected_replacement) in test_cases {
        let content = input.as_bytes();

        // Find the match
        let matches: Vec<_> = pattern.regex.find_iter(content).collect();
        assert!(!matches.is_empty(), "Should find a match in '{}'", input);

        let m = &matches[0];
        let match_text = m.as_bytes();
        let match_str = std::str::from_utf8(match_text).unwrap();

        // Verify we matched the right text
        assert_eq!(
            match_str, expected_match,
            "In '{}', should match '{}'",
            input, expected_match
        );

        // Identify the variant
        let variant = pattern
            .identify_variant(match_text)
            .expect("Should identify variant")
            .to_string();

        assert_eq!(
            variant, expected_match,
            "Should identify '{}' as variant '{}'",
            match_str, expected_match
        );

        // Look up replacement in variant map
        let replacement = variant_map
            .get(&variant)
            .expect("Variant should exist in map");

        assert_eq!(
            replacement, expected_replacement,
            "Variant '{}' should map to '{}'",
            variant, expected_replacement
        );
    }
}

#[test]
fn test_variant_map_lookup_is_case_sensitive() {
    // This test verifies that BTreeMap lookups are case-sensitive
    // and that each variant maps to its corresponding case replacement

    let mut map = BTreeMap::new();
    map.insert("oldname".to_string(), "newname".to_string());
    map.insert("Oldname".to_string(), "Newname".to_string());
    map.insert("OldName".to_string(), "NewName".to_string());
    map.insert("OLDNAME".to_string(), "NEWNAME".to_string());

    // Each key should map to its exact case equivalent
    assert_eq!(map.get("oldname"), Some(&"newname".to_string()));
    assert_eq!(map.get("Oldname"), Some(&"Newname".to_string()));
    assert_eq!(map.get("OldName"), Some(&"NewName".to_string()));
    assert_eq!(map.get("OLDNAME"), Some(&"NEWNAME".to_string()));

    // Non-existent keys should return None
    assert_eq!(map.get("oldName"), None);
    assert_eq!(map.get("OLD_NAME"), None);
}

#[test]
fn test_identify_variant_returns_exact_match() {
    // This test ensures that identify_variant returns the exact variant
    // that was matched, preserving case

    let variants = vec![
        "project".to_string(),
        "Project".to_string(),
        "PROJECT".to_string(),
    ];

    let pattern = build_pattern(&variants).unwrap();

    // Test that each variant is identified correctly
    assert_eq!(pattern.identify_variant(b"project"), Some("project"));
    assert_eq!(pattern.identify_variant(b"Project"), Some("Project"));
    assert_eq!(pattern.identify_variant(b"PROJECT"), Some("PROJECT"));

    // Non-matching text should return None
    assert_eq!(pattern.identify_variant(b"ProJect"), None);
    assert_eq!(pattern.identify_variant(b"PROJ"), None);
}
