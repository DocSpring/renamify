use aho_corasick::AhoCorasick;
use bstr::ByteSlice;
use regex::bytes::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MatchPattern {
    pub regex: Regex,
    pub variants: Vec<String>,
    matcher: AhoCorasick,
}

impl MatchPattern {
    pub fn identify_variant(&self, text: &[u8]) -> Option<&str> {
        self.matcher
            .find(text)
            .map(|m| self.variants[m.pattern().as_usize()].as_str())
    }
}

pub fn build_pattern(variants: &[String]) -> Result<MatchPattern, regex::Error> {
    if variants.is_empty() {
        return Ok(MatchPattern {
            regex: Regex::new("$^")?,
            variants: vec![],
            matcher: AhoCorasick::new(Vec::<String>::new()).unwrap(),
        });
    }

    let escaped: Vec<String> = variants.iter().map(|v| regex::escape(v)).collect();

    let longest_first = {
        let mut sorted = escaped;
        sorted.sort_by_key(|s| std::cmp::Reverse(s.len()));
        sorted
    };

    let pattern = format!("(?:{})", longest_first.join("|"));

    let regex = RegexBuilder::new(&pattern).unicode(false).build()?;

    let matcher = AhoCorasick::new(variants).unwrap();

    Ok(MatchPattern {
        regex,
        variants: variants.to_vec(),
        matcher,
    })
}

pub fn is_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let left_boundary = if start == 0 {
        true
    } else {
        !bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_'
    };

    let right_boundary = if end >= bytes.len() {
        true
    } else {
        !bytes[end].is_ascii_alphanumeric() && bytes[end] != b'_'
    };

    left_boundary && right_boundary
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
    pub variant: String,
    pub text: String,
}

pub fn find_matches(pattern: &MatchPattern, content: &[u8], file: &str) -> Vec<Match> {
    let mut matches = Vec::new();

    for m in pattern.regex.find_iter(content) {
        if !is_boundary(content, m.start(), m.end()) {
            continue;
        }

        let match_text = m.as_bytes();
        let variant = pattern
            .identify_variant(match_text)
            .unwrap_or_default()
            .to_string();

        let line_number = content[..m.start()].iter().filter(|&&b| b == b'\n').count() + 1;

        let line_start = content[..m.start()]
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |p| p + 1);
        let column = m.start() - line_start + 1;

        matches.push(Match {
            file: file.to_string(),
            line: line_number,
            column,
            start: m.start(),
            end: m.end(),
            variant,
            text: String::from_utf8_lossy(match_text).to_string(),
        });
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pattern_empty() {
        let pattern = build_pattern(&[]).unwrap();
        assert_eq!(pattern.variants.len(), 0);
    }

    #[test]
    fn test_build_pattern_single() {
        let variants = vec!["hello_world".to_string()];
        let pattern = build_pattern(&variants).unwrap();
        assert_eq!(pattern.variants.len(), 1);
        assert!(pattern.regex.is_match(b"hello_world"));
        assert!(!pattern.regex.is_match(b"hello"));
    }

    #[test]
    fn test_build_pattern_multiple() {
        let variants = vec![
            "old_name".to_string(),
            "oldName".to_string(),
            "OldName".to_string(),
        ];
        let pattern = build_pattern(&variants).unwrap();
        assert_eq!(pattern.variants.len(), 3);
        assert!(pattern.regex.is_match(b"old_name"));
        assert!(pattern.regex.is_match(b"oldName"));
        assert!(pattern.regex.is_match(b"OldName"));
    }

    #[test]
    fn test_longest_first_ordering() {
        let variants = vec![
            "foo".to_string(),
            "foobar".to_string(),
            "foobarbaz".to_string(),
        ];
        let pattern = build_pattern(&variants).unwrap();

        let text = b"foobarbaz";
        let m = pattern.regex.find(text).unwrap();
        assert_eq!(m.as_bytes(), b"foobarbaz");
    }

    #[test]
    fn test_is_boundary() {
        let text = b"hello_world test";

        assert!(is_boundary(text, 0, 11));

        assert!(is_boundary(text, 12, 16));

        assert!(!is_boundary(text, 1, 5));

        assert!(!is_boundary(text, 6, 11));
    }

    #[test]
    fn test_is_boundary_underscore() {
        let text = b"hello_world_test";

        assert!(is_boundary(text, 0, 16));

        assert!(!is_boundary(text, 0, 5));
        assert!(!is_boundary(text, 6, 11));
    }

    #[test]
    fn test_is_boundary_with_punctuation() {
        let text = b"call(hello_world);";

        assert!(is_boundary(text, 5, 16));

        assert!(!is_boundary(text, 5, 10));
    }

    #[test]
    fn test_identify_variant() {
        let variants = vec![
            "old_name".to_string(),
            "oldName".to_string(),
            "OldName".to_string(),
        ];
        let pattern = build_pattern(&variants).unwrap();

        assert_eq!(pattern.identify_variant(b"old_name"), Some("old_name"));
        assert_eq!(pattern.identify_variant(b"oldName"), Some("oldName"));
        assert_eq!(pattern.identify_variant(b"OldName"), Some("OldName"));
        assert_eq!(pattern.identify_variant(b"unknown"), None);
    }

    #[test]
    fn test_find_matches() {
        let variants = vec!["hello".to_string(), "world".to_string()];
        let pattern = build_pattern(&variants).unwrap();

        let content = b"hello world\nmore hello here";
        let matches = find_matches(&pattern, content, "test.txt");

        assert_eq!(matches.len(), 3);

        assert_eq!(matches[0].text, "hello");
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].column, 1);

        assert_eq!(matches[1].text, "world");
        assert_eq!(matches[1].line, 1);
        assert_eq!(matches[1].column, 7);

        assert_eq!(matches[2].text, "hello");
        assert_eq!(matches[2].line, 2);
        assert_eq!(matches[2].column, 6);
    }

    #[test]
    fn test_find_matches_respects_boundaries() {
        let variants = vec!["test".to_string()];
        let pattern = build_pattern(&variants).unwrap();

        let content = b"test testing attest test";
        let matches = find_matches(&pattern, content, "test.txt");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[1].start, 20);
    }

    #[test]
    fn test_special_chars_escaped() {
        let variants = vec!["foo.bar".to_string(), "foo[bar]".to_string()];
        let pattern = build_pattern(&variants).unwrap();

        assert!(pattern.regex.is_match(b"foo.bar"));
        assert!(pattern.regex.is_match(b"foo[bar]"));

        assert!(!pattern.regex.is_match(b"fooXbar"));
        assert!(!pattern.regex.is_match(b"foo_bar_"));
    }

    #[test]
    fn test_empty_content() {
        let variants = vec!["test".to_string()];
        let pattern = build_pattern(&variants).unwrap();

        let matches = find_matches(&pattern, b"", "empty.txt");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_multiline_positions() {
        let variants = vec!["foo".to_string()];
        let pattern = build_pattern(&variants).unwrap();

        let content = b"line1\nline2 foo\nfoo line3";
        let matches = find_matches(&pattern, content, "test.txt");

        assert_eq!(matches.len(), 2);

        assert_eq!(matches[0].line, 2);
        assert_eq!(matches[0].column, 7);

        assert_eq!(matches[1].line, 3);
        assert_eq!(matches[1].column, 1);
    }
}
