use crate::case_model::{generate_variant_map, Style};
use crate::compound_matcher::{find_compound_variants, CompoundMatch};
use crate::pattern::{build_pattern, is_boundary, Match, MatchPattern};
use crate::scanner::{MatchHunk, CoercionMode};
use anyhow::Result;
use bstr::ByteSlice;
use regex::bytes::Regex;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// Find all potential identifiers in the content using a broad regex pattern
fn find_all_identifiers(content: &[u8]) -> Vec<(usize, usize, String)> {
    let mut identifiers = Vec::new();
    
    // Pattern to match any identifier-like string
    // This matches snake_case, camelCase, PascalCase, kebab-case, etc.
    let pattern = r"\b[a-zA-Z_][a-zA-Z0-9_\-]*\b";
    let regex = Regex::new(pattern).unwrap();
    
    for m in regex.find_iter(content) {
        let identifier = String::from_utf8_lossy(m.as_bytes()).to_string();
        identifiers.push((m.start(), m.end(), identifier));
    }
    
    identifiers
}

/// Enhanced matching that finds both exact and compound matches
pub fn find_enhanced_matches(
    content: &[u8],
    file: &str,
    old: &str,
    new: &str,
    variant_map: &BTreeMap<String, String>,
    styles: &[Style],
) -> Vec<Match> {
    let mut all_matches = Vec::new();
    let mut processed_positions = HashSet::new();
    
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
            
            let line_number = content[..m.start()].iter().filter(|&&b| b == b'\n').count() + 1;
            
            let line_start = content[..m.start()]
                .iter()
                .rposition(|&b| b == b'\n')
                .map(|p| p + 1)
                .unwrap_or(0);
            
            let column = m.start() - line_start;
            
            // Mark this position as processed
            processed_positions.insert(m.start());
            
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
    
    // Second, find all identifiers and check for compound matches
    let identifiers = find_all_identifiers(content);
    
    for (start, end, identifier) in identifiers {
        // Skip if we already matched this position exactly
        if processed_positions.contains(&start) {
            continue;
        }
        
        // Check if this identifier contains our pattern as a compound
        let compound_matches = find_compound_variants(&identifier, old, new, styles);
        
        if !compound_matches.is_empty() {
            // We found a compound match!
            let compound = &compound_matches[0]; // Take the first match
            
            let line_number = content[..start].iter().filter(|&&b| b == b'\n').count() + 1;
            
            let line_start = content[..start]
                .iter()
                .rposition(|&b| b == b'\n')
                .map(|p| p + 1)
                .unwrap_or(0);
            
            let column = start - line_start;
            
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
    
    all_matches
}

/// Convert enhanced matches to MatchHunks with proper line context
pub fn enhanced_matches_to_hunks(
    matches: &[Match],
    content: &[u8],
    old: &str,
    new: &str,
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
        let (before, after) = if variant_map.contains_key(&m.variant) {
            // Exact match - use the variant map
            let new_variant = variant_map.get(&m.variant).unwrap_or(&m.variant);
            (m.variant.clone(), new_variant.clone())
        } else {
            // Compound match - the text field contains the replacement
            (m.variant.clone(), m.text.clone())
        };
        
        // Generate the full line with replacement
        let line_before = Some(line_string.clone());
        let line_after = if let Some(col) = line_string.find(&before) {
            let mut new_line = line_string.clone();
            new_line.replace_range(col..col + before.len(), &after);
            Some(new_line)
        } else {
            None
        };
        
        hunks.push(MatchHunk {
            file: path.to_path_buf(),
            line: m.line as u64,
            col: m.column as u32,
            variant: before.clone(),
            before,
            after,
            start: m.start,
            end: m.end,
            line_before,
            line_after,
            coercion_applied: None,
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
        let old = "preview_format";
        let new = "preview";
        
        let mut variant_map = BTreeMap::new();
        variant_map.insert("preview_format".to_string(), "preview".to_string());
        variant_map.insert("PreviewFormat".to_string(), "Preview".to_string());
        
        let styles = vec![Style::Snake, Style::Pascal];
        
        let matches = find_enhanced_matches(
            content,
            "test.rs",
            old,
            new,
            &variant_map,
            &styles,
        );
        
        // Should find both preview_format_arg and PreviewFormatArg
        assert_eq!(matches.len(), 2);
        
        // Check that we found the compound matches
        let variants: Vec<String> = matches.iter().map(|m| m.variant.clone()).collect();
        assert!(variants.contains(&"preview_format_arg".to_string()));
        assert!(variants.contains(&"PreviewFormatArg".to_string()));
    }
}