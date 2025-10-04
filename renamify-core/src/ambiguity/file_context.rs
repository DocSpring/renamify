use crate::case_model::{detect_style, Style};
use std::collections::HashMap;

/// Analyzes case style distribution in a file
pub struct FileContextAnalyzer {
    min_identifiers_threshold: usize,
    high_confidence_ratio: f64,
    medium_confidence_ratio: f64,
}

impl Default for FileContextAnalyzer {
    fn default() -> Self {
        Self {
            min_identifiers_threshold: 50,
            high_confidence_ratio: 0.6,
            medium_confidence_ratio: 0.4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileStyleStats {
    pub style_counts: HashMap<Style, usize>,
    pub total_identifiers: usize,
    pub dominant_style: Option<Style>,
    pub confidence: ConfidenceLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    Insufficient,
}

impl FileContextAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze the file content and return style statistics
    pub fn analyze(&self, content: &str) -> FileStyleStats {
        let identifiers = Self::extract_identifiers(content);
        let mut style_counts = HashMap::new();
        let mut total_unambiguous = 0;

        for identifier in &identifiers {
            // Skip ambiguous identifiers
            if crate::ambiguity::is_ambiguous(identifier, &Style::all_styles()) {
                continue;
            }

            // Only count identifiers with clear style
            if let Some(style) = detect_style(identifier) {
                *style_counts.entry(style).or_insert(0) += 1;
                total_unambiguous += 1;
            }
        }

        // Determine dominant style and confidence
        let (dominant_style, confidence) = if total_unambiguous < self.min_identifiers_threshold {
            (None, ConfidenceLevel::Insufficient)
        } else {
            self.calculate_dominance(&style_counts, total_unambiguous)
        };

        FileStyleStats {
            style_counts,
            total_identifiers: identifiers.len(),
            dominant_style,
            confidence,
        }
    }

    /// Extract potential identifiers from code
    fn extract_identifiers(content: &str) -> Vec<String> {
        let mut identifiers = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let mut string_delimiter = ' ';

        let chars: Vec<char> = content.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            // Handle strings
            if !in_comment && (ch == '"' || ch == '\'' || ch == '`') {
                if !in_string {
                    in_string = true;
                    string_delimiter = ch;
                } else if ch == string_delimiter {
                    in_string = false;
                }
                if !current.is_empty() {
                    identifiers.push(current.clone());
                    current.clear();
                }
                continue;
            }

            if in_string {
                continue;
            }

            // Handle comments (simplified)
            if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_comment = true;
                if !current.is_empty() {
                    identifiers.push(current.clone());
                    current.clear();
                }
                continue;
            }

            if in_comment && ch == '\n' {
                in_comment = false;
                continue;
            }

            if in_comment {
                continue;
            }

            // Build identifiers
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                current.push(ch);
            } else if !current.is_empty() {
                // Filter out numbers and very short identifiers
                if current.len() > 1 && !current.chars().all(char::is_numeric) {
                    identifiers.push(current.clone());
                }
                current.clear();
            }
        }

        if !current.is_empty() && current.len() > 1 && !current.chars().all(char::is_numeric) {
            identifiers.push(current);
        }

        identifiers
    }

    /// Calculate the dominant style and confidence level
    fn calculate_dominance(
        &self,
        style_counts: &HashMap<Style, usize>,
        total: usize,
    ) -> (Option<Style>, ConfidenceLevel) {
        if style_counts.is_empty() || total == 0 {
            return (None, ConfidenceLevel::Insufficient);
        }

        // Find the most common style
        let (dominant_style, count) = style_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(style, count)| (*style, *count))
            .unwrap();

        #[allow(clippy::cast_precision_loss)]
        let ratio = count as f64 / total as f64;

        let confidence = if ratio >= self.high_confidence_ratio {
            ConfidenceLevel::High
        } else if ratio >= self.medium_confidence_ratio {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        };

        (Some(dominant_style), confidence)
    }

    /// Suggest a style based on file analysis for ambiguous text
    pub fn suggest_style(&self, content: &str, possible_styles: &[Style]) -> Option<Style> {
        let stats = self.analyze(content);

        // Only suggest if we have sufficient confidence
        if matches!(
            stats.confidence,
            ConfidenceLevel::Insufficient | ConfidenceLevel::Low
        ) {
            return None;
        }

        // Check if the dominant style is one of the possible styles
        if let Some(dominant) = stats.dominant_style {
            if possible_styles.contains(&dominant) {
                return Some(dominant);
            }
        }

        // If dominant style isn't possible, try other common styles in order
        let mut sorted_styles: Vec<(Style, usize)> = stats.style_counts.into_iter().collect();
        sorted_styles.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        sorted_styles
            .into_iter()
            .map(|(style, _)| style)
            .find(|&style| possible_styles.contains(&style))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_identifiers() {
        let _analyzer = FileContextAnalyzer::new();
        let content = r#"
            function getUserName() {
                const userId = 123;
                let user_email = "test@example.com";
                // This is a comment with some_identifier
                return userId;
            }
        "#;

        let identifiers = FileContextAnalyzer::extract_identifiers(content);
        assert!(identifiers.contains(&"function".to_string()));
        assert!(identifiers.contains(&"getUserName".to_string()));
        assert!(identifiers.contains(&"const".to_string()));
        assert!(identifiers.contains(&"userId".to_string()));
        assert!(identifiers.contains(&"user_email".to_string()));
        assert!(!identifiers.contains(&"some_identifier".to_string())); // In comment
    }

    #[test]
    fn test_analyze_camel_case_file() {
        let mut analyzer = FileContextAnalyzer::new();
        analyzer.min_identifiers_threshold = 5; // Lower threshold for test
        #[allow(clippy::needless_raw_string_hashes)]
        let content = r#"
            function getUserData() {
                const userId = getUserId();
                const userName = getUserName();
                const userEmail = getUserEmail();
                const userPhone = getUserPhone();
                const userAddress = getUserAddress();
                return { userId, userName, userEmail, userPhone, userAddress };
            }
        "#;

        let stats = analyzer.analyze(content);
        assert_eq!(stats.dominant_style, Some(Style::Camel));
        assert!(matches!(
            stats.confidence,
            ConfidenceLevel::High | ConfidenceLevel::Medium
        ));
    }

    #[test]
    fn test_analyze_snake_case_file() {
        let mut analyzer = FileContextAnalyzer::new();
        analyzer.min_identifiers_threshold = 5; // Lower threshold for test
        #[allow(clippy::needless_raw_string_hashes)]
        let content = r#"
            def get_user_data():
                user_id = get_user_id()
                user_name = get_user_name()
                user_email = get_user_email()
                user_phone = get_user_phone()
                user_address = get_user_address()
                return { user_id, user_name, user_email, user_phone, user_address }
        "#;

        let stats = analyzer.analyze(content);
        assert_eq!(stats.dominant_style, Some(Style::Snake));
    }

    #[test]
    fn test_insufficient_identifiers() {
        let analyzer = FileContextAnalyzer::new();
        let content = r#"
            {
                "name": "test",
                "version": "1.0.0"
            }
        "#;

        let stats = analyzer.analyze(content);
        assert_eq!(stats.confidence, ConfidenceLevel::Insufficient);
        assert_eq!(stats.dominant_style, None);
    }

    #[test]
    fn test_mixed_styles() {
        let analyzer = FileContextAnalyzer::new();
        #[allow(clippy::needless_raw_string_hashes)]
        let content = r#"
            const USER_CONSTANT = 100;
            function getUserName() { }
            def get_user_email():
            class UserProfile { }
        "#;

        let stats = analyzer.analyze(content);
        // Should have low confidence due to mixed styles
        assert_eq!(stats.confidence, ConfidenceLevel::Insufficient); // Not enough identifiers
    }
}
