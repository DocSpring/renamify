use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Default acronyms commonly used in development
pub const DEFAULT_ACRONYMS: &[&str] = &[
    "2FA", "AMD", "AMD64", "API", "ARM", "ARM64", "CLI", "CORS", "CPU", "CSP", "CSV", "CSS", "DB",
    "DNS", "EC2", "FTP", "GIF", "GPU", "GUI", "HTML", "HTTP", "HTTPS", "ID", "IDE", "IP", "JSON",
    "JSONB", "JWT", "K8S", "MCP", "MFA", "OAuth", "OTP", "PDF", "PIN", "PNG", "QR", "RAM", "S3",
    "SCSS", "SDK", "SQL", "SSH", "SSL", "SVG", "TCP", "TLS", "TOML", "UDP", "UI", "UID", "URI",
    "URL", "UTM", "UUID", "UX", "XML", "XSS", "YAML",
];

// Global default acronym set - initialized once and reused
static DEFAULT_ACRONYM_SET: OnceLock<AcronymSet> = OnceLock::new();

/// Get the default acronym set (lazily initialized once)
pub fn get_default_acronym_set() -> &'static AcronymSet {
    DEFAULT_ACRONYM_SET.get_or_init(AcronymSet::default)
}

/// Trie node for efficient acronym matching
#[derive(Debug, Clone, Default)]
struct TrieNode {
    is_end: bool,
    children: HashMap<char, TrieNode>,
}

/// Manages a set of known acronyms for detection
#[derive(Debug, Clone)]
pub struct AcronymSet {
    acronyms: HashSet<String>,
    trie: TrieNode,
    enabled: bool,
}

impl Default for AcronymSet {
    fn default() -> Self {
        let mut set = Self {
            acronyms: HashSet::new(),
            trie: TrieNode::default(),
            enabled: true,
        };
        for &acronym in DEFAULT_ACRONYMS {
            set.add(acronym);
        }
        set
    }
}

impl AcronymSet {
    /// Create a new empty acronym set
    pub fn new() -> Self {
        Self {
            acronyms: HashSet::new(),
            trie: TrieNode::default(),
            enabled: true,
        }
    }

    /// Create a disabled acronym set (no acronym detection)
    pub fn disabled() -> Self {
        Self {
            acronyms: HashSet::new(),
            trie: TrieNode::default(),
            enabled: false,
        }
    }

    /// Create from a specific set of acronyms
    pub fn from_list(acronyms: &[String]) -> Self {
        let mut set = Self {
            acronyms: HashSet::new(),
            trie: TrieNode::default(),
            enabled: true,
        };
        for acronym in acronyms {
            set.add(acronym);
        }
        set
    }

    /// Add acronyms to the set
    pub fn include(&mut self, acronyms: Vec<String>) {
        for acronym in acronyms {
            self.acronyms.insert(acronym);
        }
    }

    /// Remove acronyms from the set
    pub fn exclude(&mut self, acronyms: Vec<String>) {
        for acronym in acronyms {
            self.acronyms.remove(&acronym);
        }
    }

    /// Add a single acronym to the set
    pub fn add(&mut self, acronym: &str) {
        let upper = acronym.to_uppercase();
        self.acronyms.insert(upper.clone());

        // Also add to trie for efficient matching
        let mut node = &mut self.trie;
        for ch in upper.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_end = true;

        // Also add lowercase version to trie (for k8s, s3, etc)
        let lower = acronym.to_lowercase();
        let mut node = &mut self.trie;
        for ch in lower.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_end = true;
    }

    /// Remove a single acronym from the set
    pub fn remove(&mut self, acronym: &str) {
        self.acronyms.remove(&acronym.to_uppercase());
        // Note: We don't remove from trie as it's complex and rebuild is simpler
        // For now, we'll check against the HashSet for final validation
    }

    /// Replace the set with only these acronyms
    pub fn only(&mut self, acronyms: Vec<String>) {
        self.acronyms.clear();
        for acronym in acronyms {
            self.acronyms.insert(acronym);
        }
    }

    /// Disable acronym detection entirely
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if a string is a known acronym
    pub fn is_acronym(&self, s: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.acronyms.contains(s)
    }

    /// Check if a string looks like an acronym (all caps, 2+ chars) and is in our set
    pub fn is_acronym_token(&self, s: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // Must be at least 2 uppercase letters
        if s.len() < 2 {
            return false;
        }

        // Check if all uppercase letters (allowing numbers)
        if !s
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return false;
        }

        // Must have at least one letter
        if !s.chars().any(|c| c.is_ascii_uppercase()) {
            return false;
        }

        // Check if it's in our known set
        self.acronyms.contains(s)
    }

    /// Find the longest acronym match starting at the given position in text
    pub fn find_longest_match<'a>(&self, text: &'a str, start_pos: usize) -> Option<&'a str> {
        if !self.enabled || start_pos >= text.len() {
            return None;
        }

        let bytes = text.as_bytes();
        let mut node = &self.trie;
        let mut last_match_end = None;
        let mut i = start_pos;

        while i < bytes.len() {
            let ch = bytes[i] as char;

            // Check both uppercase and lowercase versions
            let upper_ch = ch.to_ascii_uppercase();
            let lower_ch = ch.to_ascii_lowercase();

            // Try to continue in trie
            if let Some(next_node) = node
                .children
                .get(&ch)
                .or_else(|| node.children.get(&upper_ch))
                .or_else(|| node.children.get(&lower_ch))
            {
                node = next_node;
                if node.is_end {
                    // Found a valid acronym ending here
                    last_match_end = Some(i + 1);
                }
                i += 1;
            } else {
                break;
            }
        }

        last_match_end.map(|end| &text[start_pos..end])
    }
}

/// Token type for classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Title,   // ^[A-Z][a-z0-9]*$
    Acronym, // ^[A-Z]{2,}$ and in AcronymSet
    Lower,   // ^[a-z0-9]+$
    Other,   // Anything else
}

/// Classify a single token
pub fn classify_token(token: &str, acronym_set: &AcronymSet) -> TokenType {
    if token.is_empty() {
        return TokenType::Other;
    }

    let chars: Vec<char> = token.chars().collect();
    let first = chars[0];

    // Check for all lowercase (with possible numbers)
    if first.is_ascii_lowercase() {
        if token
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return TokenType::Lower;
        }
        return TokenType::Other;
    }

    // Check for Title case: starts with uppercase, rest are lowercase/digits
    if first.is_ascii_uppercase() {
        if token.len() == 1 {
            // Single uppercase letter - could be acronym if in set
            if acronym_set.is_acronym(token) {
                return TokenType::Acronym;
            }
            return TokenType::Title;
        }

        // Check if rest are lowercase/digits (Title case)
        let rest_lower = token
            .chars()
            .skip(1)
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        if rest_lower {
            return TokenType::Title;
        }

        // Check if it's an acronym (all uppercase)
        if acronym_set.is_acronym_token(token) {
            return TokenType::Acronym;
        }
    }

    TokenType::Other
}

/// Container style for hyphenated identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyphenContainerStyle {
    TrainCase,            // All Title tokens
    TrainCaseWithAcronym, // Title tokens with trailing Acronym tokens
    KebabCase,            // All Lower tokens
    ScreamingKebab,       // All Acronym tokens (even if not in set)
    HyphenCaps,           // First Title, rest Lower or Acronym
    Mixed,                // Everything else
}

/// Classify a hyphen-separated identifier
pub fn classify_hyphen_container(
    tokens: &[&str],
    acronym_set: &AcronymSet,
) -> HyphenContainerStyle {
    if tokens.is_empty() {
        return HyphenContainerStyle::Mixed;
    }

    let classified: Vec<TokenType> = tokens
        .iter()
        .map(|t| classify_token(t, acronym_set))
        .collect();

    // All lowercase -> kebab
    if classified.iter().all(|t| matches!(t, TokenType::Lower)) {
        return HyphenContainerStyle::KebabCase;
    }

    // All uppercase (even if not acronyms) -> screaming kebab
    let all_upper = tokens.iter().all(|t| {
        !t.is_empty()
            && t.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    });
    if all_upper {
        return HyphenContainerStyle::ScreamingKebab;
    }

    // Count Title and Acronym tokens
    let caps_like_count = classified
        .iter()
        .filter(|t| matches!(t, TokenType::Title | TokenType::Acronym))
        .count();

    // All Title/Acronym -> could be Train-Case or Train-Case-with-acronym
    if caps_like_count == classified.len() {
        // Check if we have trailing acronyms
        let has_acronym = classified.iter().any(|t| matches!(t, TokenType::Acronym));
        if has_acronym {
            return HyphenContainerStyle::TrainCaseWithAcronym;
        }
        return HyphenContainerStyle::TrainCase;
    }

    // First Title, rest Lower or Acronym -> HyphenCaps
    if matches!(classified[0], TokenType::Title) {
        let rest_ok = classified[1..]
            .iter()
            .all(|t| matches!(t, TokenType::Lower | TokenType::Acronym));
        if rest_ok {
            return HyphenContainerStyle::HyphenCaps;
        }
    }

    HyphenContainerStyle::Mixed
}

/// Check if search tokens match a subsequence of segment tokens
pub fn matches_subsequence(
    search_tokens: &[String],
    segment_tokens: &[&str],
    acronym_set: &AcronymSet,
) -> Option<(usize, usize)> {
    if search_tokens.is_empty() || segment_tokens.is_empty() {
        return None;
    }

    let search_len = search_tokens.len();
    if search_len > segment_tokens.len() {
        return None;
    }

    // Try each starting position
    for start_idx in 0..=(segment_tokens.len() - search_len) {
        let mut all_match = true;

        for (i, search_token) in search_tokens.iter().enumerate() {
            let segment_token = segment_tokens[start_idx + i];
            let token_type = classify_token(segment_token, acronym_set);

            // Get canonical form based on token type
            let canonical = match token_type {
                TokenType::Title | TokenType::Acronym => segment_token.to_lowercase(),
                TokenType::Lower | TokenType::Other => segment_token.to_string(),
            };

            if &canonical != search_token {
                all_match = false;
                break;
            }
        }

        if all_match {
            return Some((start_idx, start_idx + search_len));
        }
    }

    None
}

/// Extract trailing acronyms from a token sequence
pub fn extract_trailing_acronyms(
    tokens: &[&str],
    _start_idx: usize,
    end_idx: usize,
    acronym_set: &AcronymSet,
) -> Vec<String> {
    let mut acronyms = Vec::new();

    // Look at tokens after the match
    for &token in &tokens[end_idx..] {
        if acronym_set.is_acronym_token(token) {
            acronyms.push(token.to_string());
        } else {
            // Stop at first non-acronym
            break;
        }
    }

    acronyms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_acronym_set() {
        let set = AcronymSet::default();
        assert!(set.is_acronym("API"));
        assert!(set.is_acronym("CLI"));
        assert!(!set.is_acronym("NOTANACRONYM"));
    }

    #[test]
    fn test_classify_token() {
        let set = AcronymSet::default();

        assert_eq!(classify_token("Title", &set), TokenType::Title);
        assert_eq!(classify_token("lower", &set), TokenType::Lower);
        assert_eq!(classify_token("API", &set), TokenType::Acronym);
        assert_eq!(classify_token("CLI", &set), TokenType::Acronym);
        assert_eq!(classify_token("UNKNOWN", &set), TokenType::Other);
        assert_eq!(classify_token("mixedCase", &set), TokenType::Other);
    }

    #[test]
    fn test_classify_hyphen_container() {
        let set = AcronymSet::default();

        // Pure Train-Case
        let tokens = vec!["Rename", "Tool", "Engine"];
        assert_eq!(
            classify_hyphen_container(&tokens, &set),
            HyphenContainerStyle::TrainCase
        );

        // Train-Case with acronym
        let tokens = vec!["Rename", "Tool", "CLI"];
        assert_eq!(
            classify_hyphen_container(&tokens, &set),
            HyphenContainerStyle::TrainCaseWithAcronym
        );

        // Kebab case
        let tokens = vec!["rename", "tool", "engine"];
        assert_eq!(
            classify_hyphen_container(&tokens, &set),
            HyphenContainerStyle::KebabCase
        );

        // Mixed
        let tokens = vec!["Rename", "tool", "Engine"];
        assert_eq!(
            classify_hyphen_container(&tokens, &set),
            HyphenContainerStyle::Mixed
        );
    }

    #[test]
    fn test_matches_subsequence() {
        let set = AcronymSet::default();

        let search = vec!["rename".to_string(), "tool".to_string()];
        let segment = vec!["Rename", "Tool", "CLI"];

        let result = matches_subsequence(&search, &segment, &set);
        assert_eq!(result, Some((0, 2)));

        // No match
        let segment = vec!["Other", "Tool", "CLI"];
        let result = matches_subsequence(&search, &segment, &set);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_trailing_acronyms() {
        let set = AcronymSet::default();

        let tokens = vec!["Rename", "Tool", "CLI", "API"];
        let acronyms = extract_trailing_acronyms(&tokens, 0, 2, &set);
        assert_eq!(acronyms, vec!["CLI", "API"]);

        let tokens = vec!["Rename", "Tool", "Engine"];
        let acronyms = extract_trailing_acronyms(&tokens, 0, 2, &set);
        assert_eq!(acronyms, Vec::<String>::new());
    }
}
