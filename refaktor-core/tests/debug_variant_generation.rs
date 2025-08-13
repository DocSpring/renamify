use refaktor_core::acronym::AcronymSet;
use refaktor_core::case_model::{parse_to_tokens_with_acronyms, to_style, Style};

#[test]
fn test_debug_k8s_variant_generation() {
    // Create acronym set with K8S
    let mut acronym_set = AcronymSet::default();
    acronym_set.add("K8S");

    // Parse k8s_cluster with K8S as an acronym
    let tokens = parse_to_tokens_with_acronyms("k8s_cluster", &acronym_set);

    println!("Tokens for 'k8s_cluster': {:?}", tokens);

    // Generate different style variants
    let snake = to_style(&tokens, Style::Snake);
    let camel = to_style(&tokens, Style::Camel);
    let pascal = to_style(&tokens, Style::Pascal);

    println!("Snake: {}", snake);
    println!("Camel: {}", camel);
    println!("Pascal: {}", pascal);

    // Also test the reverse - parsing K8SCluster
    let tokens2 = parse_to_tokens_with_acronyms("K8SCluster", &acronym_set);
    println!("\nTokens for 'K8SCluster': {:?}", tokens2);

    let snake2 = to_style(&tokens2, Style::Snake);
    let camel2 = to_style(&tokens2, Style::Camel);
    let pascal2 = to_style(&tokens2, Style::Pascal);

    println!("Snake: {}", snake2);
    println!("Camel: {}", camel2);
    println!("Pascal: {}", pascal2);

    // Test without K8S as an acronym
    let empty_set = AcronymSet::default();
    let tokens3 = parse_to_tokens_with_acronyms("K8SCluster", &empty_set);
    println!(
        "\nTokens for 'K8SCluster' without K8S acronym: {:?}",
        tokens3
    );
}

#[test]
fn test_debug_variant_map_generation() {
    // We can't access the private function, so let's just test tokenization
    // which is the core of the issue

    // Create acronym set with K8S
    let mut acronym_set = AcronymSet::default();
    acronym_set.add("K8S");

    println!("Testing with K8S as an acronym:");

    // Test tokenizing k8s_cluster
    let tokens1 = parse_to_tokens_with_acronyms("k8s_cluster", &acronym_set);
    println!("'k8s_cluster' tokens: {:?}", tokens1);

    // Generate styles from tokens
    println!("  Snake: {}", to_style(&tokens1, Style::Snake));
    println!("  Camel: {}", to_style(&tokens1, Style::Camel));
    println!("  Pascal: {}", to_style(&tokens1, Style::Pascal));

    // Test tokenizing K8SCluster
    let tokens2 = parse_to_tokens_with_acronyms("K8SCluster", &acronym_set);
    println!("\n'K8SCluster' tokens: {:?}", tokens2);

    println!("  Snake: {}", to_style(&tokens2, Style::Snake));
    println!("  Camel: {}", to_style(&tokens2, Style::Camel));
    println!("  Pascal: {}", to_style(&tokens2, Style::Pascal));

    // Test k8sCluster
    let tokens3 = parse_to_tokens_with_acronyms("k8sCluster", &acronym_set);
    println!("\n'k8sCluster' tokens: {:?}", tokens3);

    println!("  Snake: {}", to_style(&tokens3, Style::Snake));
    println!("  Camel: {}", to_style(&tokens3, Style::Camel));
    println!("  Pascal: {}", to_style(&tokens3, Style::Pascal));
}
