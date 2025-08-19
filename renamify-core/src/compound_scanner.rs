use crate::case_model::{generate_variant_map, Style};
use crate::compound_matcher::{find_compound_variants, CompoundMatch};
use crate::pattern::{build_pattern, is_boundary, is_compound_boundary, Match, MatchPattern};
use crate::scanner::{CoercionMode, MatchHunk};
use anyhow::Result;
use bstr::ByteSlice;
use regex::bytes::Regex;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// Normalize a path by removing Windows long path prefix if present
fn normalize_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("\\\\?\\") {
            PathBuf::from(&path_str[4..])
        } else {
            path.to_path_buf()
        }
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

/// Check if a string has any recognizable case style
fn has_recognizable_case_style(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check for camelCase/PascalCase
    if is_camel_or_pascal_case(s) {
        return true;
    }

    // Check for SCREAMING_SNAKE_CASE
    if s.contains('_') && s.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
        return true;
    }

    // Check for snake_case
    if s.contains('_')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return true;
    }

    // Check for kebab-case
    if s.contains('-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return true;
    }

    false
}

/// Check if a string looks like camelCase or `PascalCase`
fn is_camel_or_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must have at least one lowercase and one uppercase letter
    let has_lower = s.bytes().any(|b| b.is_ascii_lowercase());
    let has_upper = s.bytes().any(|b| b.is_ascii_uppercase());

    // Must not contain underscores, hyphens, or spaces (pure camel/pascal)
    let is_pure = !s.contains(['_', '-', ' ']);

    has_lower && has_upper && is_pure
}

/// Find all potential identifiers in the content using a broad regex pattern
fn find_all_identifiers(content: &[u8]) -> Vec<(usize, usize, String)> {
    let mut identifiers = Vec::new();

    // Pattern to match identifier-like strings, including dots in some contexts
    // This is tricky: we want to split on dots for things like obj.prop but keep
    // dots for mixed-style identifiers like config.max_value
    let pattern = r"\b[a-zA-Z_][a-zA-Z0-9_\-\.]*\b";
    let regex = Regex::new(pattern).unwrap();

    for m in regex.find_iter(content) {
        let identifier = String::from_utf8_lossy(m.as_bytes()).to_string();

        // Debug: print what identifiers are being found
        if std::env::var("RENAMIFY_DEBUG_IDENTIFIERS").is_ok() {
            println!(
                "Found identifier: '{}' at {}-{}",
                identifier,
                m.start(),
                m.end()
            );
        }

        // Split on dots for expressions like obj.method, process.env.VARIABLE, etc.
        // Always split on dots to check each part independently for compound matching
        if identifier.contains('.') {
            // Split on dots for things like obj.method or this.property
            let parts: Vec<&str> = identifier.split('.').collect();
            let mut current_pos = m.start();

            for (i, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    identifiers.push((current_pos, current_pos + part.len(), (*part).to_string()));
                }
                current_pos += part.len() + 1; // +1 for the dot

                // If there are more parts, we've consumed a dot
                if i < parts.len() - 1 && current_pos <= m.end() {
                    // The dot is at current_pos - 1, move past it
                    // current_pos is already at the right position for the next part
                }
            }
        } else {
            // Keep as single identifier (including dots for mixed-style names)
            identifiers.push((m.start(), m.end(), identifier));
        }
    }

    identifiers
}

/// Enhanced matching that finds both exact and compound matches
pub fn find_enhanced_matches(
    content: &[u8],
    file: &str,
    search: &str,
    replace: &str,
    variant_map: &BTreeMap<String, String>,
    styles: &[Style],
) -> Vec<Match> {
    let mut all_matches = Vec::new();
    let mut processed_ranges = Vec::new(); // Track (start, end) ranges that were exactly matched

    // First, find exact matches using the existing pattern approach
    let variants: Vec<String> = variant_map.keys().cloned().collect();
    if let Ok(pattern) = build_pattern(&variants) {
        for m in pattern.regex.find_iter(content) {
            if !is_boundary(content, m.start(), m.end()) {
                continue;
            }

            let match_text = m.as_bytes();
            let variant = pattern
                .identify_variant(match_text)
                .unwrap_or_default()
                .to_string();

            #[allow(clippy::naive_bytecount)]
            let line_number = content[..m.start()].iter().filter(|&&b| b == b'\n').count() + 1;

            let line_start = content[..m.start()]
                .iter()
                .rposition(|&b| b == b'\n')
                .map_or(0, |p| p + 1);

            let column = m.start() - line_start;

            // Mark this range as processed
            processed_ranges.push((m.start(), m.end()));

            all_matches.push(Match {
                file: file.to_string(),
                line: line_number,
                column,
                start: m.start(),
                end: m.end(),
                variant: variant.clone(),
                text: String::from_utf8_lossy(match_text).to_string(),
            });
        }
    }

    // Second, if Original style is enabled, do simple string replacement for remaining instances
    // Note: Original style does NOT use boundary checking - it matches exact strings anywhere
    if styles.contains(&Style::Original) {
        // Find the original exact mapping in the variant map
        if let Some(replacement) = variant_map.get(search) {
            let search_pattern = search;
            let replace_pattern = replacement;
            let pattern_bytes = search_pattern.as_bytes();
            let mut search_start = 0;

            while let Some(pos) = content[search_start..]
                .windows(pattern_bytes.len())
                .position(|window| window == pattern_bytes)
            {
                let actual_pos = search_start + pos;
                let end_pos = actual_pos + pattern_bytes.len();

                // Skip if this position was already processed
                let already_processed = processed_ranges.iter().any(|(proc_start, proc_end)| {
                    // Check for overlap
                    actual_pos < *proc_end && end_pos > *proc_start
                });

                // For Original style, use a more permissive boundary check
                // Allow matching when followed by underscore if it's not a simple identifier continuation
                let passes_boundary =
                    if actual_pos > 0 && content[actual_pos - 1].is_ascii_alphanumeric() {
                        // If preceded by alphanumeric, skip this match (it's in the middle of a word)
                        false
                    } else if end_pos < content.len() {
                        let next_char = content[end_pos];
                        // Allow if followed by:
                        // - Non-alphanumeric and not underscore (normal boundary)
                        // - Underscore followed by non-alphanumeric (like _{, _[, etc.)
                        if !next_char.is_ascii_alphanumeric() && next_char != b'_' {
                            true // Normal word boundary
                        } else if next_char == b'_' && end_pos + 1 < content.len() {
                            // Check what comes after the underscore
                            let after_underscore = content[end_pos + 1];
                            // Allow if underscore is followed by non-identifier characters
                            !after_underscore.is_ascii_alphanumeric() && after_underscore != b'_'
                        } else {
                            false
                        }
                    } else {
                        true // At end of content
                    };

                if !already_processed && passes_boundary {
                    #[allow(clippy::naive_bytecount)]
                    let line_number = content[..actual_pos]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count()
                        + 1;

                    let line_start = content[..actual_pos]
                        .iter()
                        .rposition(|&b| b == b'\n')
                        .map_or(0, |p| p + 1);

                    let column = actual_pos - line_start;

                    // Mark this range as processed
                    processed_ranges.push((actual_pos, end_pos));

                    all_matches.push(Match {
                        file: file.to_string(),
                        line: line_number,
                        column,
                        start: actual_pos,
                        end: end_pos,
                        variant: search_pattern.to_string(),
                        text: replace_pattern.clone(),
                    });
                }

                search_start = actual_pos + 1; // Move past this match
            }
        }
    }

    // Third, find all identifiers and check for compound matches
    let identifiers = find_all_identifiers(content);

    for (start, end, identifier) in identifiers {
        // Skip if this identifier was already matched exactly or if it's completely contained within a processed range
        let should_skip = processed_ranges.iter().any(|(proc_start, proc_end)| {
            // Skip if exact match (same start and end)
            (*proc_start == start && *proc_end == end) ||
                // Skip if identifier is completely contained within processed range
                (*proc_start <= start && *proc_end >= end)
        });

        if should_skip {
            continue;
        }

        // Check if this identifier contains our pattern as a compound
        let compound_matches = find_compound_variants(&identifier, search, replace, styles);

        if !compound_matches.is_empty() {
            // We found a compound match!
            let compound = &compound_matches[0]; // Take the first match

            #[allow(clippy::naive_bytecount)]
            let line_number = content[..start].iter().filter(|&&b| b == b'\n').count() + 1;

            let line_start = content[..start]
                .iter()
                .rposition(|&b| b == b'\n')
                .map_or(0, |p| p + 1);

            let column = start - line_start;

            // Add the compound match
            all_matches.push(Match {
                file: file.to_string(),
                line: line_number,
                column,
                start,
                end,
                variant: compound.full_identifier.clone(),
                text: compound.replacement.clone(),
            });
        }
    }

    // Sort matches by position
    all_matches.sort_by_key(|m| (m.line, m.column));

    // Remove overlapping matches, prioritizing longer matches
    let mut final_matches = Vec::new();
    let mut i = 0;
    while i < all_matches.len() {
        let current = &all_matches[i];
        let mut best_match = current;
        let mut j = i + 1;

        // Find all matches that overlap with current
        while j < all_matches.len() {
            let candidate = &all_matches[j];

            // Check if they overlap
            let overlaps = current.start < candidate.end && current.end > candidate.start;

            if !overlaps {
                break; // Since sorted by position, no more overlaps
            }

            // If they overlap, choose the longer match
            let current_len = best_match.end - best_match.start;
            let candidate_len = candidate.end - candidate.start;

            if candidate_len > current_len {
                best_match = candidate;
            }

            j += 1;
        }

        final_matches.push(best_match.clone());
        i = j; // Skip all overlapping matches
    }

    final_matches
}

/// Convert enhanced matches to `MatchHunks` with proper line context
pub fn enhanced_matches_to_hunks(
    matches: &[Match],
    content: &[u8],
    search: &str,
    replace: &str,
    variant_map: &BTreeMap<String, String>,
    path: &Path,
    styles: &[Style],
    coerce_mode: CoercionMode,
) -> Vec<MatchHunk> {
    let lines: Vec<&[u8]> = content.lines_with_terminator().collect();
    let mut hunks = Vec::new();

    for m in matches {
        let line_idx = m.line.saturating_sub(1);
        if line_idx >= lines.len() {
            continue;
        }

        let line = lines[line_idx];
        let line_string = String::from_utf8_lossy(line).to_string();

        // Determine if this is a compound match or exact match
        let (content, replace) = if variant_map.contains_key(&m.variant) {
            // Exact match - use the variant map
            let new_variant = variant_map.get(&m.variant).unwrap_or(&m.variant);
            (m.variant.clone(), new_variant.clone())
        } else {
            // Compound match - the text field contains the replacement
            (m.variant.clone(), m.text.clone())
        };

        // Generate the full line with replacement
        let line_before = Some(line_string.clone());
        let line_after = if let Some(col) = line_string.find(&content) {
            let mut new_line = line_string.clone();
            new_line.replace_range(col..col + content.len(), &replace);
            Some(new_line)
        } else {
            None
        };

        hunks.push(MatchHunk {
            file: normalize_path(path),
            line: m.line as u64,
            col: u32::try_from(m.column).unwrap_or(u32::MAX),
            variant: content.clone(),
            content,
            replace,
            start: m.start,
            end: m.end,
            line_before,
            line_after,
            coercion_applied: None,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        });
    }

    hunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_identifiers() {
        let content = b"let preview_format_arg = PreviewFormatArg::new();";
        let identifiers = find_all_identifiers(content);

        // Should find: let, preview_format_arg, PreviewFormatArg, new
        assert!(identifiers.len() >= 4);

        let names: Vec<String> = identifiers.iter().map(|(_, _, id)| id.clone()).collect();
        assert!(names.contains(&"preview_format_arg".to_string()));
        assert!(names.contains(&"PreviewFormatArg".to_string()));
    }

    #[test]
    fn test_enhanced_matching_finds_compounds() {
        let content = b"let preview_format_arg = PreviewFormatArg::new();";
        let search = "preview_format";
        let replace = "preview";

        let mut variant_map = BTreeMap::new();
        variant_map.insert("preview_format".to_string(), "preview".to_string());
        variant_map.insert("PreviewFormat".to_string(), "Preview".to_string());

        let styles = vec![Style::Snake, Style::Pascal];

        let matches =
            find_enhanced_matches(content, "test.rs", search, replace, &variant_map, &styles);

        // Should find both preview_format_arg and PreviewFormatArg
        assert_eq!(matches.len(), 2);

        // Check that we found the compound matches
        let variants: Vec<String> = matches.iter().map(|m| m.variant.clone()).collect();
        assert!(variants.contains(&"preview_format_arg".to_string()));
        assert!(variants.contains(&"PreviewFormatArg".to_string()));
    }
}
