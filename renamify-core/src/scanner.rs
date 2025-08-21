use crate::acronym::AcronymSet;
use crate::case_model::Style;
use crate::pattern::{build_pattern, Match};
use crate::rename::plan_renames;
use anyhow::Result;
use bstr::ByteSlice;
use content_inspector::ContentType;
use globset::{Glob, GlobSet, GlobSetBuilder};
use memmap2::Mmap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use ts_rs::TS;

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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlanOptions {
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub respect_gitignore: bool, // Deprecated, use unrestricted_level instead
    #[ts(type = "number")]
    pub unrestricted_level: u8, // 0=default, 1=-u, 2=-uu, 3=-uuu
    #[ts(optional)]
    pub styles: Option<Vec<Style>>,
    pub rename_files: bool,
    pub rename_dirs: bool,
    pub rename_root: bool, // Allow renaming the root directory
    #[ts(type = "string")]
    pub plan_out: PathBuf,
    pub coerce_separators: CoercionMode,
    pub exclude_match: Vec<String>, // Specific matches to exclude
    #[ts(optional)]
    pub exclude_matching_lines: Option<String>, // Regex to exclude lines matching this pattern
    pub no_acronyms: bool,          // Disable acronym detection
    pub include_acronyms: Vec<String>, // Additional acronyms to recognize
    pub exclude_acronyms: Vec<String>, // Default acronyms to exclude
    pub only_acronyms: Vec<String>, // Replace default list with these acronyms
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CoercionMode {
    Auto,                          // Default: automatically detect and apply
    Off,                           // Disable coercion
    Force(crate::coercion::Style), // Force a specific style
}

impl PlanOptions {
    /// Returns true if binary files should be treated as text (level 3/-uuu)
    pub fn binary_as_text(&self) -> bool {
        self.unrestricted_level >= 3
    }
}

impl Default for PlanOptions {
    fn default() -> Self {
        Self {
            includes: vec![],
            excludes: vec![],
            respect_gitignore: true, // For backward compatibility
            unrestricted_level: 0,   // Default: respect all ignore files
            styles: None,
            rename_files: true,
            rename_dirs: true,
            rename_root: false, // Default: do not rename root directory
            plan_out: PathBuf::from(".renamify/plan.json"),
            coerce_separators: CoercionMode::Auto,
            exclude_match: vec![],
            exclude_matching_lines: None,
            no_acronyms: false, // Default: enable acronym detection
            include_acronyms: vec![],
            exclude_acronyms: vec![],
            only_acronyms: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MatchHunk {
    #[ts(type = "string")]
    pub file: PathBuf,
    #[ts(type = "number")]
    pub line: u64,
    #[ts(type = "number")]
    pub col: u32,
    pub variant: String,
    pub content: String, // The word/variant being replaced
    #[serde(skip_serializing_if = "String::is_empty")]
    pub replace: String, // The replacement word/variant
    #[ts(type = "number")]
    pub start: usize,
    #[ts(type = "number")]
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub line_before: Option<String>, // Full line context for diff preview
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub line_after: Option<String>, // Full line with replacement for diff preview
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub coercion_applied: Option<String>, // Details about coercion if applied
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "string")]
    pub original_file: Option<PathBuf>, // Original file path before any renames
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "string")]
    pub renamed_file: Option<PathBuf>, // File path after renames (if different)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub patch_hash: Option<String>, // SHA256 hash of the patch file for this change
}

/// Helper function to check if a `PathBuf` is empty (for serde skip)
fn is_empty_path(p: &Path) -> bool {
    p.as_os_str().is_empty()
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Rename {
    #[ts(type = "string")]
    pub path: PathBuf,
    #[serde(skip_serializing_if = "is_empty_path")]
    #[ts(type = "string")]
    pub new_path: PathBuf,
    pub kind: RenameKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub coercion_applied: Option<String>, // Details about coercion if applied
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
pub enum RenameKind {
    File,
    Dir,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Stats {
    #[ts(type = "number")]
    pub files_scanned: usize,
    #[ts(type = "number")]
    pub total_matches: usize,
    #[ts(type = "Record<string, number>")]
    pub matches_by_variant: HashMap<String, usize>,
    #[ts(type = "number")]
    pub files_with_matches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Plan {
    pub id: String,
    pub created_at: String,
    pub search: String,
    pub replace: String,
    pub styles: Vec<Style>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub matches: Vec<MatchHunk>,
    pub paths: Vec<Rename>,
    pub stats: Stats,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Array<string>")]
    pub created_directories: Option<Vec<PathBuf>>, // Directories created during apply that should be removed on undo
}

/// Backward-compatible single-path scan (for tests)
pub fn scan_repository(root: &Path, old: &str, new: &str, options: &PlanOptions) -> Result<Plan> {
    scan_repository_multi(&[root.to_path_buf()], old, new, options)
}

/// Multi-path repository scan
pub fn scan_repository_multi(
    roots: &[PathBuf],
    search: &str,
    replace: &str,
    options: &PlanOptions,
) -> Result<Plan> {
    // Validate the exclude pattern if provided
    if let Some(ref pattern) = options.exclude_matching_lines {
        Regex::new(pattern).map_err(|e| {
            anyhow::anyhow!("Invalid regex pattern for --exclude-matching-lines: {}", e)
        })?;
    }

    // Build acronym set from options
    let acronym_set = build_acronym_set(options);
    let variant_map = generate_variant_map_with_acronyms(
        search,
        replace,
        options.styles.as_deref(),
        &acronym_set,
    );
    let variants: Vec<String> = variant_map.keys().cloned().collect();
    let _pattern = build_pattern(&variants)?;

    let include_globs = build_globset(&options.includes)?;
    let exclude_globs = build_globset(&options.excludes)?;

    let mut matches = Vec::new();
    let mut stats = Stats {
        files_scanned: 0,
        total_matches: 0,
        matches_by_variant: HashMap::new(),
        files_with_matches: 0,
    };

    // Use shared walker configuration
    let walker = crate::configure_walker(roots, options).build();

    for entry in walker {
        let Ok(entry) = entry else {
            continue;
        };

        // Skip non-files
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }

        let path = entry.path();

        // Apply include/exclude filters (use relative path for matching)
        // For multi-path, find the root that this path belongs to
        let relative_path = roots
            .iter()
            .find_map(|root| path.strip_prefix(root).ok())
            .unwrap_or(path);

        if let Some(ref includes) = include_globs {
            if !includes.is_match(relative_path) {
                continue;
            }
        }

        if let Some(ref excludes) = exclude_globs {
            if excludes.is_match(relative_path) {
                continue;
            }
        }

        if let Ok(content) = read_file_content(path) {
            stats.files_scanned += 1;

            // Skip binary files unless we're in -uuu mode
            if !options.binary_as_text() && is_binary(&content) {
                continue;
            }

            // Only use compound scanner (which also finds exact matches)
            // Debug: Check what styles are being used
            let actual_styles = options.styles.as_deref().unwrap_or(&[
                Style::Original, // Always include for exact string matching
                Style::Snake,
                Style::Kebab,
                Style::Camel,
                Style::Pascal,
                Style::ScreamingSnake,
                Style::Train, // Include Train-Case in default styles
            ]);

            if std::env::var("RENAMIFY_DEBUG_COMPOUND").is_ok() {
                eprintln!("SCANNER: Using styles: {:?}", actual_styles);
                eprintln!("SCANNER: options.styles = {:?}", options.styles);
            }

            let mut file_matches = if replace.is_empty() {
                // For search operations, use simple pattern matching without compound matches
                let variants: Vec<String> = variant_map.keys().cloned().collect();
                if let Ok(pattern) = build_pattern(&variants) {
                    crate::pattern::find_matches(&pattern, &content, path.to_str().unwrap_or(""))
                } else {
                    Vec::new()
                }
            } else {
                // For replace operations, use full compound matching
                crate::compound_scanner::find_enhanced_matches(
                    &content,
                    path.to_str().unwrap_or(""),
                    search,
                    replace,
                    &variant_map,
                    actual_styles,
                )
            };

            if !file_matches.is_empty() {
                // Sort by position to maintain order
                file_matches.sort_by_key(|m| (m.line, m.column));

                let hunks = generate_hunks(&file_matches, &content, &variant_map, path, options);

                stats.files_with_matches += 1;
                stats.total_matches += hunks.len();

                for hunk in &hunks {
                    *stats
                        .matches_by_variant
                        .entry(hunk.variant.clone())
                        .or_insert(0) += 1;
                }

                matches.extend(hunks);
            }
        }
    }

    let paths = if options.rename_files || options.rename_dirs {
        let mut all_renames = Vec::new();
        for root in roots {
            let mut root_renames = plan_renames(root, &variant_map, options)?;
            // For search mode (when new is empty), clear the new_path to empty PathBuf
            if replace.is_empty() {
                for rename in &mut root_renames {
                    rename.new_path = PathBuf::new();
                }
            }
            all_renames.append(&mut root_renames);
        }
        all_renames
    } else {
        vec![]
    };

    let id = generate_plan_id(search, replace, options);
    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    Ok(Plan {
        id,
        created_at,
        search: search.to_string(),
        replace: replace.to_string(),
        styles: options.styles.clone().unwrap_or_else(|| {
            vec![
                Style::Snake,
                Style::Kebab,
                Style::Camel,
                Style::Pascal,
                Style::ScreamingSnake,
            ]
        }),
        includes: options.includes.clone(),
        excludes: options.excludes.clone(),
        matches,
        paths,
        stats,
        version: "1.0.0".to_string(),
        created_directories: None,
    })
}

pub fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        // Add the pattern as-is
        builder.add(Glob::new(pattern)?);

        // If pattern looks like a directory (ends with / or no wildcards and no extension),
        // also add a pattern that matches everything under it
        if pattern.ends_with('/')
            || (!pattern.contains('*') && !pattern.contains('?') && !pattern.contains('.'))
        {
            let recursive_pattern = if pattern.ends_with('/') {
                format!("{}**", pattern)
            } else {
                format!("{}/**", pattern)
            };
            builder.add(Glob::new(&recursive_pattern)?);
        }
    }
    Ok(Some(builder.build()?))
}

fn read_file_content(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;

    let file = File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.len() > 50 * 1024 * 1024 {
        let mut content = Vec::new();
        std::fs::File::open(path)?.read_to_end(&mut content)?;
        Ok(content)
    } else {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(mmap.to_vec())
    }
}

fn is_binary(content: &[u8]) -> bool {
    matches!(content_inspector::inspect(content), ContentType::BINARY)
}

fn generate_hunks(
    matches: &[Match],
    content: &[u8],
    variant_map: &BTreeMap<String, String>,
    path: &Path,
    options: &PlanOptions,
) -> Vec<MatchHunk> {
    let lines: Vec<&[u8]> = content.lines_with_terminator().collect();
    let mut hunks = Vec::new();

    // Compile the exclude pattern if provided
    let exclude_line_regex = if let Some(ref pattern) = options.exclude_matching_lines {
        Regex::new(pattern).ok()
    } else {
        None
    };

    for m in matches {
        // Check if this match should be excluded
        if options.exclude_match.contains(&m.variant) || options.exclude_match.contains(&m.text) {
            continue;
        }

        let line_idx = m.line.saturating_sub(1);
        if line_idx >= lines.len() {
            continue;
        }

        let line = lines[line_idx];
        let line_string = String::from_utf8_lossy(line).to_string();

        // Check if this line should be excluded based on regex pattern
        if let Some(ref regex) = exclude_line_regex {
            if regex.is_match(&line_string) {
                continue;
            }
        }

        // Check if this is a compound match (text field contains replacement)
        // or an exact match (use variant map)
        let is_compound_match = !variant_map.contains_key(&m.variant);
        let (content, mut replace) = if is_compound_match {
            // Compound match - text field has the replacement
            (m.variant.clone(), m.text.clone())
        } else {
            // Exact match - use variant map
            let new_variant = variant_map.get(&m.variant).unwrap();
            (m.variant.clone(), new_variant.clone())
        };
        let mut coercion_applied = None;

        // Apply coercion if enabled
        if options.coerce_separators == CoercionMode::Auto {
            // Find the match position within the line and extract context
            if let Some(match_pos) = line_string.find(&content) {
                let identifier_context =
                    extract_immediate_context(&line_string, match_pos, match_pos + content.len());

                if is_compound_match {
                    // For compound matches, check if style coercion was already applied by the compound matcher
                    // by detecting if the replacement uses a consistent style that matches the context
                    if let Some(detected_style) =
                        detect_compound_coercion(&identifier_context, &content, &replace)
                    {
                        coercion_applied = Some(format!(
                            "Compound coercion applied: {} style",
                            style_name(detected_style)
                        ));
                    }
                } else {
                    // For exact matches, apply normal coercion
                    if let Some((_coerced, reason)) =
                        crate::coercion::apply_coercion(&identifier_context, &content, &replace)
                    {
                        if let Some(coerced_variant) =
                            apply_coercion_to_variant(&identifier_context, &content, &replace)
                        {
                            replace = coerced_variant;
                            coercion_applied = Some(reason);
                        }
                    }
                }
            }
        }

        // For diff mode, we need the full line context
        let line_before = line_string.clone();

        // Create the after line by replacing the variant in the original line
        // Use the column position from the match to ensure we replace the right occurrence
        let match_col = m.column;
        let line_after =
            if match_col < line_string.len() && line_string[match_col..].starts_with(&content) {
                let mut after_line = String::new();
                after_line.push_str(&line_string[..match_col]);
                after_line.push_str(&replace);
                after_line.push_str(&line_string[match_col + content.len()..]);
                after_line
            } else {
                // Fallback: try to find the match in the line
                if let Some(match_pos) = line_string.find(&content) {
                    let mut after_line = String::new();
                    after_line.push_str(&line_string[..match_pos]);
                    after_line.push_str(&replace);
                    after_line.push_str(&line_string[match_pos + content.len()..]);
                    after_line
                } else {
                    // Could not find the match in the line - this shouldn't happen
                    line_before.clone()
                }
            };

        hunks.push(MatchHunk {
            file: normalize_path(path),
            line: m.line as u64,
            col: u32::try_from(m.column).unwrap_or(u32::MAX),
            variant: m.variant.clone(),
            content,
            replace,
            line_before: Some(line_before),
            line_after: Some(line_after),
            start: m.start,
            end: m.end,
            coercion_applied,
            original_file: None,
            renamed_file: None,
            patch_hash: None,
        });
    }

    hunks
}

/// Extract the immediate context around a match to make better coercion decisions
fn extract_immediate_context(line: &str, match_start: usize, match_end: usize) -> String {
    let chars: Vec<char> = line.chars().collect();

    // Convert byte positions to character indices
    let char_start = line[..match_start].chars().count();
    let char_end = line[..match_end].chars().count();

    let mut context_start = char_start;
    let mut context_end = char_end;

    // Characters that are part of identifiers or compound names
    let is_identifier_char = |c: char| -> bool { c.is_alphanumeric() || c == '_' || c == '-' };

    // Extend backwards to find the start of the immediate identifier
    // Stop at any separator character
    while context_start > 0 {
        let prev_char = chars[context_start - 1];
        if is_identifier_char(prev_char) {
            context_start -= 1;
        } else {
            // Stop at any separator or other character
            break;
        }
    }

    // Extend forwards to find the end of the immediate identifier
    // Stop at any separator character
    while context_end < chars.len() {
        let next_char = chars[context_end];
        if is_identifier_char(next_char) {
            context_end += 1;
        } else {
            // Stop at any separator or other character
            break;
        }
    }

    // Extract the context substring
    chars[context_start..context_end].iter().collect()
}

/// Apply coercion logic to just the variant (word-level), not the entire container
fn apply_coercion_to_variant(
    container: &str,
    _old_variant: &str,
    new_variant: &str,
) -> Option<String> {
    // Detect the container style
    let container_style = crate::coercion::detect_style(container);

    // If container has mixed or unknown style, no coercion
    if container_style == crate::coercion::Style::Mixed
        || container_style == crate::coercion::Style::Dot
    {
        return None;
    }

    // Apply the container's style to the new variant
    let new_tokens = crate::coercion::tokenize(new_variant);
    let coerced_variant = crate::coercion::render_tokens(&new_tokens, container_style);

    Some(coerced_variant)
}

/// Detect if compound coercion was applied by checking if the replacement style matches the context style
fn detect_compound_coercion(
    context: &str,
    _match: &str,
    after: &str,
) -> Option<crate::coercion::Style> {
    let context_style = crate::coercion::detect_style(context);
    let after_style = crate::coercion::detect_style(after);

    // If the context has a clear style and the replacement matches that style,
    // then compound coercion was likely applied
    match context_style {
        crate::coercion::Style::Snake if after_style == crate::coercion::Style::Snake => {
            Some(context_style)
        },
        crate::coercion::Style::Kebab if after_style == crate::coercion::Style::Kebab => {
            Some(context_style)
        },
        crate::coercion::Style::Pascal if after_style == crate::coercion::Style::Pascal => {
            Some(context_style)
        },
        crate::coercion::Style::Camel if after_style == crate::coercion::Style::Camel => {
            Some(context_style)
        },
        crate::coercion::Style::ScreamingSnake
            if after_style == crate::coercion::Style::ScreamingSnake =>
        {
            Some(context_style)
        },
        _ => None,
    }
}

/// Convert a coercion style to a human-readable name
fn style_name(style: crate::coercion::Style) -> &'static str {
    match style {
        crate::coercion::Style::Snake => "Snake",
        crate::coercion::Style::Kebab => "Kebab",
        crate::coercion::Style::Pascal => "Pascal",
        crate::coercion::Style::Camel => "Camel",
        crate::coercion::Style::ScreamingSnake => "ScreamingSnake",
        crate::coercion::Style::Mixed => "Mixed",
        crate::coercion::Style::Dot => "Dot",
    }
}

fn generate_plan_id(old: &str, new: &str, options: &PlanOptions) -> String {
    let mut hasher = Sha256::new();
    hasher.update(old.as_bytes());
    hasher.update(new.as_bytes());
    hasher.update(format!("{:?}", options).as_bytes());
    hasher.update(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
            .as_bytes(),
    );
    format!("{:x}", hasher.finalize())[..16].to_string()
}

pub fn write_plan(plan: &Plan, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, plan)?;
    Ok(())
}

/// Build an `AcronymSet` from `PlanOptions`
fn build_acronym_set(options: &PlanOptions) -> AcronymSet {
    if options.no_acronyms {
        // Disable acronym detection
        AcronymSet::disabled()
    } else if !options.only_acronyms.is_empty() {
        // Replace default list with custom acronyms
        AcronymSet::from_list(&options.only_acronyms)
    } else {
        // Start with default set
        let mut set = AcronymSet::default();

        // Add custom acronyms
        for acronym in &options.include_acronyms {
            set.add(acronym);
        }

        // Remove excluded acronyms
        for acronym in &options.exclude_acronyms {
            set.remove(acronym);
        }

        set
    }
}

/// Generate variant map with custom acronym configuration
fn generate_variant_map_with_acronyms(
    search: &str,
    replace: &str,
    styles: Option<&[Style]>,
    acronym_set: &AcronymSet,
) -> std::collections::BTreeMap<String, String> {
    // Use the acronym-aware tokenization
    let old_tokens = crate::case_model::parse_to_tokens_with_acronyms(search, acronym_set);
    let new_tokens = crate::case_model::parse_to_tokens_with_acronyms(replace, acronym_set);

    let default_styles = [
        Style::Original, // Always include the exact original string
        Style::Snake,
        Style::Kebab,
        Style::Camel,
        Style::Pascal,
        Style::ScreamingSnake,
        Style::Train, // Include Train-Case for patterns like Renamify-Core-Engine
        Style::ScreamingTrain, // Include ScreamingTrain for patterns like RENAMIFY-DEBUG
    ];
    let styles = styles.unwrap_or(&default_styles);

    let mut map = std::collections::BTreeMap::new();

    // Process styles in order to prioritize Original style
    for style in styles {
        if *style == Style::Original {
            // Add the original pattern directly
            map.insert(search.to_string(), replace.to_string());
        } else {
            let search_variant = crate::case_model::to_style(&old_tokens, *style);
            let replace_variant = crate::case_model::to_style(&new_tokens, *style);

            // Only add if not already in map (Original takes priority)
            map.entry(search_variant).or_insert(replace_variant);
        }
    }

    // Removed automatic case variants - they were causing incorrect matches
    // All variants should come from the explicit style system only

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plan_options_default() {
        let opts = PlanOptions::default();
        assert!(opts.respect_gitignore);
        assert!(opts.rename_files);
        assert!(opts.rename_dirs);
        assert_eq!(opts.plan_out, PathBuf::from(".renamify/plan.json"));
    }

    #[test]
    fn test_is_binary() {
        assert!(!is_binary(b"hello world"));
        assert!(is_binary(&[0x00, 0x01, 0x02, 0x03]));
        assert!(!is_binary(&[0xFF, 0xFE, 0xFD]));
    }

    #[test]
    fn test_generate_plan_id() {
        let opts = PlanOptions::default();
        let id1 = generate_plan_id("old", "new", &opts);
        std::thread::sleep(std::time::Duration::from_secs(1));
        let id2 = generate_plan_id("old", "new", &opts);

        assert_eq!(id1.len(), 16);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_build_globset_empty() {
        let result = build_globset(&[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_globset_patterns() {
        let patterns = vec!["*.rs".to_string(), "src/**".to_string()];
        let result = build_globset(&patterns).unwrap();
        assert!(result.is_some());

        let globset = result.unwrap();
        assert!(globset.is_match("test.rs"));
        assert!(globset.is_match("src/main.rs"));
        assert!(!globset.is_match("test.txt"));
    }

    #[test]
    fn test_build_globset_directory_exclusion() {
        // Test that directory patterns like "docs" automatically match subdirectories
        let patterns = vec!["docs".to_string()];
        let result = build_globset(&patterns).unwrap();
        assert!(result.is_some());

        let globset = result.unwrap();
        // Should match the directory itself
        assert!(globset.is_match("docs"));
        // Should match files and subdirectories within it
        assert!(globset.is_match("docs/README.md"));
        assert!(globset.is_match("docs/src/assets/file.png"));
        assert!(globset.is_match("docs/deep/nested/path.txt"));
        // Should not match other directories
        assert!(!globset.is_match("src/docs.rs"));
        assert!(!globset.is_match("other/file.md"));
    }

    #[test]
    fn test_build_globset_directory_with_slash() {
        // Test that "docs/" also works
        let patterns = vec!["docs/".to_string()];
        let result = build_globset(&patterns).unwrap();
        assert!(result.is_some());

        let globset = result.unwrap();
        assert!(globset.is_match("docs/"));
        assert!(globset.is_match("docs/README.md"));
        assert!(globset.is_match("docs/src/assets/file.png"));
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let opts = PlanOptions::default();

        let plan = scan_repository(temp_dir.path(), "old", "new", &opts).unwrap();

        assert_eq!(plan.search, "old");
        assert_eq!(plan.replace, "new");
        assert_eq!(plan.matches.len(), 0);
        assert_eq!(plan.paths.len(), 0);
        assert_eq!(plan.stats.files_scanned, 0);
    }

    #[test]
    fn test_generate_hunks() {
        use crate::pattern::Match;

        let content = b"old_name and oldName here";
        let mut variant_map = BTreeMap::new();
        variant_map.insert("old_name".to_string(), "new_name".to_string());
        variant_map.insert("oldName".to_string(), "newName".to_string());

        let matches = vec![
            Match {
                file: "test.txt".to_string(),
                line: 1,
                column: 1,
                start: 0,
                end: 8,
                variant: "old_name".to_string(),
                text: "old_name".to_string(),
            },
            Match {
                file: "test.txt".to_string(),
                line: 1,
                column: 14,
                start: 13,
                end: 20,
                variant: "oldName".to_string(),
                text: "oldName".to_string(),
            },
        ];

        let opts = PlanOptions::default();
        let hunks = generate_hunks(
            &matches,
            content,
            &variant_map,
            Path::new("test.txt"),
            &opts,
        );

        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].variant, "old_name");
        // The before/after fields contain just the words
        assert_eq!(hunks[0].content, "old_name");
        assert_eq!(hunks[0].replace, "new_name");
        // The line context is in separate fields
        assert_eq!(
            hunks[0].line_before.as_ref().unwrap(),
            "old_name and oldName here"
        );
        assert_eq!(
            hunks[0].line_after.as_ref().unwrap(),
            "new_name and oldName here"
        );

        assert_eq!(hunks[1].variant, "oldName");
        assert_eq!(hunks[1].content, "oldName");
        assert_eq!(hunks[1].replace, "newName");
        assert_eq!(
            hunks[1].line_before.as_ref().unwrap(),
            "old_name and oldName here"
        );
        assert_eq!(
            hunks[1].line_after.as_ref().unwrap(),
            "old_name and newName here"
        );
    }

    #[test]
    fn test_generate_hunks_multiline() {
        use crate::pattern::Match;

        let content = b"fn old_name() {\n    println!(\"oldName\");\n    old_name();\n}\n";
        let mut variant_map = BTreeMap::new();
        variant_map.insert("old_name".to_string(), "new_name".to_string());
        variant_map.insert("oldName".to_string(), "newName".to_string());

        let matches = vec![
            Match {
                file: "test.rs".to_string(),
                line: 1,
                column: 4,
                start: 3,
                end: 11,
                variant: "old_name".to_string(),
                text: "old_name".to_string(),
            },
            Match {
                file: "test.rs".to_string(),
                line: 2,
                column: 14,
                start: 29,
                end: 36,
                variant: "oldName".to_string(),
                text: "oldName".to_string(),
            },
            Match {
                file: "test.rs".to_string(),
                line: 3,
                column: 5,
                start: 44,
                end: 52,
                variant: "old_name".to_string(),
                text: "old_name".to_string(),
            },
        ];

        let opts = PlanOptions::default();
        let hunks = generate_hunks(&matches, content, &variant_map, Path::new("test.rs"), &opts);

        assert_eq!(hunks.len(), 3);

        // First line replacement
        assert_eq!(hunks[0].variant, "old_name");
        assert_eq!(hunks[0].line, 1);
        assert_eq!(hunks[0].content, "old_name");
        assert_eq!(hunks[0].replace, "new_name");
        assert_eq!(hunks[0].line_before.as_ref().unwrap(), "fn old_name() {\n");
        assert_eq!(hunks[0].line_after.as_ref().unwrap(), "fn new_name() {\n");

        // Second line replacement
        assert_eq!(hunks[1].variant, "oldName");
        assert_eq!(hunks[1].line, 2);
        assert_eq!(hunks[1].content, "oldName");
        assert_eq!(hunks[1].replace, "newName");
        assert_eq!(
            hunks[1].line_before.as_ref().unwrap(),
            "    println!(\"oldName\");\n"
        );
        assert_eq!(
            hunks[1].line_after.as_ref().unwrap(),
            "    println!(\"newName\");\n"
        );

        // Third line replacement
        assert_eq!(hunks[2].variant, "old_name");
        assert_eq!(hunks[2].line, 3);
        assert_eq!(hunks[2].content, "old_name");
        assert_eq!(hunks[2].replace, "new_name");
        assert_eq!(hunks[2].line_before.as_ref().unwrap(), "    old_name();\n");
        assert_eq!(hunks[2].line_after.as_ref().unwrap(), "    new_name();\n");
    }

    #[test]
    fn test_walk_directory() {
        use ignore::Walk;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let walker = Walk::new(temp_dir.path());
        let files: Vec<_> = walker
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
            .collect();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path().file_name().unwrap(), "test.txt");
    }

    #[test]
    fn test_scan_with_matches() {
        use ignore::Walk;

        // Create a simple test case
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "old_name and oldName here").unwrap();

        // Use non-parallel walk for testing
        let walker = Walk::new(temp_dir.path());
        let mut file_count = 0;
        for e in walker.flatten() {
            if e.file_type().is_some_and(|t| t.is_file()) {
                file_count += 1;
            }
        }
        assert_eq!(file_count, 1, "Walker should find 1 file");

        // Now test with scan_repository
        let opts = PlanOptions {
            respect_gitignore: false,
            ..Default::default()
        };

        let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();

        // We expect 2 matches: "old_name" and "oldName"
        assert_eq!(
            plan.matches.len(),
            2,
            "Expected 2 matches, found {}",
            plan.matches.len()
        );
        assert_eq!(plan.stats.files_scanned, 1);
        assert_eq!(plan.stats.files_with_matches, 1);
    }

    #[test]
    fn test_rename_sorting() {
        let mut renames = vec![
            Rename {
                path: PathBuf::from("/a/b/file.txt"),
                new_path: PathBuf::from("/a/b/new.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
            Rename {
                path: PathBuf::from("/a/dir"),
                new_path: PathBuf::from("/a/new_dir"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            },
            Rename {
                path: PathBuf::from("/a/b/c/deep.txt"),
                new_path: PathBuf::from("/a/b/c/new_deep.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
        ];

        renames.sort_by(|a, b| {
            let a_depth = a.path.components().count();
            let b_depth = b.path.components().count();

            match (
                matches!(a.kind, RenameKind::Dir),
                matches!(b.kind, RenameKind::Dir),
            ) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b_depth.cmp(&a_depth),
            }
        });

        assert!(matches!(renames[0].kind, RenameKind::Dir));
        assert_eq!(renames[1].path.components().count(), 5);
    }

    #[test]
    fn test_write_plan() {
        let temp_dir = TempDir::new().unwrap();
        let plan_path = temp_dir.path().join(".renamify/plan.json");

        let plan = Plan {
            id: "test123".to_string(),
            created_at: "123456789".to_string(),
            search: "old".to_string(),
            replace: "new".to_string(),
            styles: vec![Style::Snake],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            paths: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
            created_directories: None,
        };

        write_plan(&plan, &plan_path).unwrap();
        assert!(plan_path.exists());

        let content = std::fs::read_to_string(&plan_path).unwrap();
        assert!(content.contains("\"id\": \"test123\""));
    }
}
