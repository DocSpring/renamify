use renamify_core::case_model::generate_variant_map;

#[test]
fn test_exact_pascal_case_preserves_user_casing() {
    // When user provides PascalCase input and it matches exactly,
    // the replacement should be exactly what they typed
    let map = generate_variant_map("DocSpring", "FormAPI", None);

    // Exact match should preserve exact casing
    assert_eq!(
        map.get("DocSpring"),
        Some(&"FormAPI".to_string()),
        "PascalCase exact match should preserve user's exact casing (FormAPI, not FormApi)"
    );

    // Other variants should still work correctly
    assert_eq!(map.get("doc_spring"), Some(&"form_api".to_string()));
    assert_eq!(map.get("docSpring"), Some(&"formAPI".to_string()));
    assert_eq!(map.get("DOC_SPRING"), Some(&"FORM_API".to_string()));
}

#[test]
fn test_multiple_acronyms_in_replacement() {
    let map = generate_variant_map("OldName", "HTTPSAPIClient", None);

    // Exact PascalCase match should preserve all acronyms as typed
    assert_eq!(
        map.get("OldName"),
        Some(&"HTTPSAPIClient".to_string()),
        "Should preserve HTTPS and API as typed by user"
    );

    // snake_case should recognize acronyms
    assert_eq!(map.get("old_name"), Some(&"https_api_client".to_string()));

    // SCREAMING_SNAKE should preserve acronyms
    assert_eq!(map.get("OLD_NAME"), Some(&"HTTPS_API_CLIENT".to_string()));
}

#[test]
fn test_mixed_case_preservation() {
    // Test various mixed case patterns that users might explicitly want
    let test_cases = vec![
        ("MyService", "IoTDevice", "IoTDevice"), // IoT is a specific casing
        ("MyService", "macOSApp", "macOSApp"),   // macOS is a specific casing
        ("MyService", "XMLParser", "XMLParser"), // All caps acronym
        ("MyService", "Html5Video", "Html5Video"), // Mixed acronym/number
        ("MyService", "PDFReader", "PDFReader"), // Common acronym
    ];

    for (search, replace, expected) in test_cases {
        let map = generate_variant_map(search, replace, None);
        assert_eq!(
            map.get(search),
            Some(&expected.to_string()),
            "Failed for {} -> {}",
            search,
            replace
        );
    }
}

#[test]
fn test_camel_case_preserves_acronyms() {
    let map = generate_variant_map("oldName", "newAPIClient", None);

    // camelCase exact match should preserve the acronym
    assert_eq!(
        map.get("oldName"),
        Some(&"newAPIClient".to_string()),
        "camelCase should preserve API acronym as typed"
    );

    // When converting from PascalCase to camelCase
    let map2 = generate_variant_map("OldName", "NewAPIClient", None);
    assert_eq!(
        map2.get("oldName"),
        Some(&"newAPIClient".to_string()),
        "Should properly handle acronyms when converting to camelCase"
    );
}

#[test]
fn test_user_intent_overrides_conventions() {
    // If user explicitly types something unconventional, respect it
    let map = generate_variant_map("StandardName", "WeIrDCaSe", None);

    // Exact match should preserve exactly what user typed
    assert_eq!(
        map.get("StandardName"),
        Some(&"WeIrDCaSe".to_string()),
        "Should preserve user's exact casing even if unconventional"
    );
}

#[test]
fn test_atomic_mode_with_acronyms() {
    use renamify_core::atomic::AtomicConfig;
    use renamify_core::case_model::generate_variant_map_with_atomic;

    let atomic_config = AtomicConfig::from_flags_and_config(
        true, // both atomic
        false,
        false,
        vec![],
    );

    let map = generate_variant_map_with_atomic("DocSpring", "FormAPI", None, Some(&atomic_config));

    // With atomic mode, should still preserve exact casing for PascalCase
    assert_eq!(
        map.get("DocSpring"),
        Some(&"FormAPI".to_string()),
        "Atomic mode should preserve FormAPI exactly as typed"
    );

    // snake_case becomes all lowercase (atomic)
    assert_eq!(
        map.get("docspring"),
        Some(&"formapi".to_string()),
        "Atomic snake_case should be all lowercase"
    );

    // SCREAMING becomes all uppercase (atomic)
    assert_eq!(
        map.get("DOCSPRING"),
        Some(&"FORMAPI".to_string()),
        "Atomic SCREAMING should be all uppercase"
    );

    // camelCase should preserve the acronym but with atomic behavior
    assert_eq!(
        map.get("docSpring"),
        Some(&"formAPI".to_string()),
        "Atomic camelCase should preserve acronym casing"
    );
}

#[test]
fn test_real_world_examples() {
    // Real examples that users would expect to work
    let test_cases = vec![
        ("GitLab", "GitHub", "GitHub"),             // Both are well-known
        ("MySQL", "PostgreSQL", "PostgreSQL"),      // Database names
        ("OAuth", "SAML", "SAML"),                  // Auth protocols
        ("iPhone", "iPad", "iPad"),                 // Apple products
        ("JavaScript", "TypeScript", "TypeScript"), // Languages
        ("DocuSign", "DocSpring", "DocSpring"),     // Services
    ];

    for (search, replace, expected) in test_cases {
        let map = generate_variant_map(search, replace, None);
        assert_eq!(
            map.get(search),
            Some(&expected.to_string()),
            "Failed for {} -> {}",
            search,
            replace
        );
    }
}
