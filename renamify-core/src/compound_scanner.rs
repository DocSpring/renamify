use crate::case_model::Style;
use crate::compound_matcher::find_compound_variants;
use crate::pattern::{build_pattern, is_boundary, Match};
use crate::scanner::{CoercionMode, MatchHunk};
use bstr::ByteSlice;
use regex::bytes::Regex;
use std::collections::BTreeMap;
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

/// Find all potential identifiers in the content using a broad regex pattern
fn find_all_identifiers(content: &[u8], styles: &[Style]) -> Vec<(usize, usize, String)> {
    let mut identifiers = Vec::new();

    // Pattern to match identifier-like strings, including dots in some contexts
    // This is tricky: we want to split on dots for things like obj.prop but keep
    // dots for mixed-style identifiers like config.max_value
    // For Title style, we need to include spaces to capture "Title Case" patterns
    let pattern = if styles.len() == 1 && styles[0] == Style::Title {
        // Special pattern for Title style that includes spaces
        r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b"
    } else {
        r"\b[a-zA-Z_][a-zA-Z0-9_\-\.]*\b"
    };
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

        // Only split on dots if dot style is NOT in the selected styles
        // When dot style is selected, keep dot-separated identifiers intact
        let should_split_on_dots = !styles.contains(&Style::Dot);

        if identifier.contains('.') && should_split_on_dots {
            // Split on dots for things like obj.method or this.property
            // But NOT when we're specifically looking for dot.case style
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
            // Keep as single identifier (including dots)
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
    // Skip this for Original-only mode, as it will be handled in the second pass with strict boundaries
    // Also skip for single-style mode with single-word search (we want compound matches only in that case)
    let is_single_word_search = !search.contains('_')
        && !search.contains('-')
        && !search.contains('.')
        && !search.contains(' ');
    let is_single_style_search = styles.len() == 1;
    let skip_exact_match = is_single_word_search && is_single_style_search;

    if !skip_exact_match {
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
    }

    // Third, find all identifiers and check for compound matches
    {
        let identifiers = find_all_identifiers(content, styles);

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
    }

    // Sort matches by position
    all_matches.sort_by_key(|m| (m.line, m.column));

    // Remove overlapping matches, prioritizing longer matches
    // Use a proper deduplication approach that checks against all previously selected matches
    let mut final_matches = Vec::new();

    for candidate in all_matches {
        // Check if this candidate overlaps with any already selected match
        let overlaps_with_selected = final_matches.iter().any(|selected: &Match| {
            candidate.start < selected.end && candidate.end > selected.start
        });

        if overlaps_with_selected {
            // Find the overlapping match and see if we should replace it
            let mut should_replace = false;
            let mut replace_idx = None;

            for (idx, selected) in final_matches.iter().enumerate() {
                if candidate.start < selected.end && candidate.end > selected.start {
                    // They overlap - choose the longer one
                    let candidate_len = candidate.end - candidate.start;
                    let selected_len = selected.end - selected.start;

                    if candidate_len > selected_len {
                        should_replace = true;
                        replace_idx = Some(idx);
                        break;
                    }
                }
            }

            if should_replace {
                // Replace the shorter match with this longer one
                final_matches[replace_idx.unwrap()] = candidate;
            } else {
                // Skip this candidate (it's shorter than the overlapping match)
            }
        } else {
            // No overlap, add it
            final_matches.push(candidate);
        }
    }

    final_matches
}

/// Convert enhanced matches to `MatchHunks` with proper line context
pub fn enhanced_matches_to_hunks(
    matches: &[Match],
    content: &[u8],
    _search: &str,
    _replace: &str,
    variant_map: &BTreeMap<String, String>,
    path: &Path,
    _styles: &[Style],
    _coerce_mode: CoercionMode,
) -> Vec<MatchHunk> {
    let lines: Vec<&[u8]> = content.lines_with_terminator().collect();
    let mut hunks = Vec::new();

    // Sort matches by position and deduplicate overlapping ones
    let mut sorted_matches = matches.to_vec();
    sorted_matches.sort_by_key(|m| (m.start, m.end));

    // Remove overlapping matches, keeping the longest/most specific match
    let mut deduplicated_matches = Vec::new();
    for m in sorted_matches {
        // Check if this match overlaps with any existing match
        let overlaps = deduplicated_matches.iter().any(|existing: &Match| {
            // Two matches overlap if one starts before the other ends
            m.start < existing.end && m.end > existing.start
        });

        if overlaps {
            // Find the overlapping match
            if let Some(existing_idx) = deduplicated_matches
                .iter()
                .position(|existing| m.start < existing.end && m.end > existing.start)
            {
                let existing = &deduplicated_matches[existing_idx];

                // Keep the longer match (more specific)
                let m_length = m.end - m.start;
                let existing_length = existing.end - existing.start;

                if m_length > existing_length {
                    // Replace existing with the longer match
                    deduplicated_matches[existing_idx] = m;
                }
                // If existing is longer or same length, keep existing (do nothing)
            }
        } else {
            deduplicated_matches.push(m);
        }
    }

    for m in deduplicated_matches {
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
        // Use default styles for test
        let styles = vec![Style::Snake, Style::Pascal];
        let identifiers = find_all_identifiers(content, &styles);

        // Should find: let, preview_format_arg, PreviewFormatArg, new
        assert!(identifiers.len() >= 4);

        let names: Vec<String> = identifiers.iter().map(|(_, _, id)| id.clone()).collect();
        assert!(names.contains(&"preview_format_arg".to_string()));
        assert!(names.contains(&"PreviewFormatArg".to_string()));
    }

    #[test]
    fn test_find_all_identifiers_dot_style() {
        let content = b"test.case use.case brief.case obj.method";

        // When looking for dot style only, keep dot-separated identifiers intact
        let dot_styles = vec![Style::Dot];
        let identifiers = find_all_identifiers(content, &dot_styles);
        let names: Vec<String> = identifiers.iter().map(|(_, _, id)| id.clone()).collect();
        assert!(names.contains(&"test.case".to_string()));
        assert!(names.contains(&"use.case".to_string()));
        assert!(names.contains(&"brief.case".to_string()));
        assert!(names.contains(&"obj.method".to_string()));

        // When using other styles, split on dots
        let other_styles = vec![Style::Snake, Style::Camel];
        let identifiers = find_all_identifiers(content, &other_styles);
        let names: Vec<String> = identifiers.iter().map(|(_, _, id)| id.clone()).collect();
        // Should split into individual parts
        assert!(names.contains(&"test".to_string()));
        assert!(names.contains(&"case".to_string()));
        assert!(names.contains(&"use".to_string()));
        assert!(names.contains(&"brief".to_string()));
        assert!(names.contains(&"obj".to_string()));
        assert!(names.contains(&"method".to_string()));
        // Should NOT contain the full dot-separated identifiers
        assert!(!names.contains(&"test.case".to_string()));
        assert!(!names.contains(&"use.case".to_string()));
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
