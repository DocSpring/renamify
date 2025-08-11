use crate::case_model::{generate_variant_map, Style};
use crate::pattern::{build_pattern, find_matches, Match};
use crate::rename::plan_renames;
use anyhow::Result;
use bstr::ByteSlice;
use content_inspector::ContentType;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOptions {
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub respect_gitignore: bool,  // Deprecated, use unrestricted_level instead
    pub unrestricted_level: u8,   // 0=default, 1=-u, 2=-uu, 3=-uuu
    pub styles: Option<Vec<Style>>,
    pub rename_files: bool,
    pub rename_dirs: bool,
    pub rename_root: bool,        // Allow renaming the root directory
    pub plan_out: PathBuf,
    pub coerce_separators: CoercionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CoercionMode {
    Auto,     // Default: automatically detect and apply
    Off,      // Disable coercion
    Force(crate::coercion::Style),  // Force a specific style
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
            respect_gitignore: true,  // For backward compatibility
            unrestricted_level: 0,    // Default: respect all ignore files
            styles: None,
            rename_files: true,
            rename_dirs: true,
            rename_root: false,       // Default: do not rename root directory
            plan_out: PathBuf::from(".refaktor/plan.json"),
            coerce_separators: CoercionMode::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchHunk {
    pub file: PathBuf,
    pub line: u64,
    pub col: u32,
    pub variant: String,
    pub before: String,  // The word/variant being replaced
    pub after: String,   // The replacement word/variant
    pub start: usize,
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_before: Option<String>,  // Full line context for diff preview
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_after: Option<String>,   // Full line with replacement for diff preview
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coercion_applied: Option<String>,  // Details about coercion if applied
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rename {
    pub from: PathBuf,
    pub to: PathBuf,
    pub kind: RenameKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coercion_applied: Option<String>,  // Details about coercion if applied
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenameKind {
    File,
    Dir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub files_scanned: usize,
    pub total_matches: usize,
    pub matches_by_variant: HashMap<String, usize>,
    pub files_with_matches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub created_at: String,
    pub old: String,
    pub new: String,
    pub styles: Vec<Style>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub matches: Vec<MatchHunk>,
    pub renames: Vec<Rename>,
    pub stats: Stats,
    pub version: String,
}

pub fn scan_repository(
    root: &Path,
    old: &str,
    new: &str,
    options: &PlanOptions,
) -> Result<Plan> {
    let variant_map = generate_variant_map(old, new, options.styles.as_deref());
    let variants: Vec<String> = variant_map.keys().cloned().collect();
    let pattern = build_pattern(&variants)?;

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
    let walker = crate::configure_walker(root, options).build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Skip non-files
        if !entry.file_type().map_or(false, |t| t.is_file()) {
            continue;
        }

        let path = entry.path();
        
        // Apply include/exclude filters (use relative path for matching)
        let relative_path = path.strip_prefix(root).unwrap_or(path);
        
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

            let file_matches = find_matches(&pattern, &content, path.to_str().unwrap_or(""));
            
            if !file_matches.is_empty() {
                let hunks = generate_hunks(&file_matches, &content, &variant_map, path, options);
                
                stats.files_with_matches += 1;
                stats.total_matches += hunks.len();
                
                for hunk in &hunks {
                    *stats.matches_by_variant
                        .entry(hunk.variant.clone())
                        .or_insert(0) += 1;
                }
                
                matches.extend(hunks);
            }
        }
    }

    let renames = if options.rename_files || options.rename_dirs {
        plan_renames(root, &variant_map, options)?
    } else {
        vec![]
    };

    let id = generate_plan_id(old, new, options);
    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    Ok(Plan {
        id,
        created_at,
        old: old.to_string(),
        new: new.to_string(),
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
        renames,
        stats,
        version: "1.0.0".to_string(),
    })
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn read_file_content(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    
    if metadata.len() > 50 * 1024 * 1024 {
        let mut content = Vec::new();
        use std::io::Read;
        std::fs::File::open(path)?.read_to_end(&mut content)?;
        Ok(content)
    } else {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(mmap.to_vec())
    }
}

fn is_binary(content: &[u8]) -> bool {
    matches!(
        content_inspector::inspect(content),
        ContentType::BINARY
    )
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

    for m in matches {
        let line_idx = m.line.saturating_sub(1);
        if line_idx >= lines.len() {
            continue;
        }

        let line = lines[line_idx];
        let line_string = String::from_utf8_lossy(line).to_string();
        
        let new_variant = variant_map.get(&m.variant).unwrap_or(&m.variant);
        
        // The before/after fields contain just the word being replaced (for apply)
        let before = m.variant.clone();
        let mut after = new_variant.clone();
        let mut coercion_applied = None;

        // Apply coercion if enabled
        if let CoercionMode::Auto = options.coerce_separators {
            // Find the match position within the line and extract context
            if let Some(match_pos) = line_string.find(&m.variant) {
                let context = extract_immediate_context(&line_string, match_pos, match_pos + m.variant.len());
                
                if let Some((_coerced, reason)) = crate::coercion::apply_coercion(&context, &m.variant, new_variant) {
                    if let Some(coerced_variant) = apply_coercion_to_variant(&context, &m.variant, new_variant) {
                        after = coerced_variant;
                        coercion_applied = Some(reason);
                    }
                }
            }
        }
        
        // For diff mode, we need the full line context
        let line_before = line_string.trim_end().to_string();
        
        // Create the after line by replacing the variant in the original line
        let line_after = if let Some(match_pos) = line_string.find(&m.variant) {
            let mut after_line = String::new();
            after_line.push_str(&line_string[..match_pos]);
            after_line.push_str(&after);
            after_line.push_str(&line_string[match_pos + m.variant.len()..]);
            after_line.trim_end().to_string()
        } else {
            // Fallback to original line if we can't find the match (shouldn't happen)
            line_before.clone()
        };

        hunks.push(MatchHunk {
            file: path.to_path_buf(),
            line: m.line as u64,
            col: m.column as u32,
            variant: m.variant.clone(),
            before,
            after,
            line_before: Some(line_before),
            line_after: Some(line_after),
            start: m.start,
            end: m.end,
            coercion_applied,
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
    let is_identifier_char = |c: char| -> bool {
        c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
    };
    
    // Characters that indicate we should stop extending context but might be part of a path/namespace
    let is_path_separator = |c: char| -> bool {
        c == '/' || c == '\\' || c == ':' || c == '@'
    };
    
    // Extend backwards to find the start of the identifier/path chain
    while context_start > 0 {
        let prev_char = chars[context_start - 1];
        if is_identifier_char(prev_char) || is_path_separator(prev_char) {
            context_start -= 1;
        } else {
            break;
        }
    }
    
    // Extend forwards to find the end of the identifier/path chain  
    while context_end < chars.len() {
        let next_char = chars[context_end];
        if is_identifier_char(next_char) || is_path_separator(next_char) {
            context_end += 1;
        } else {
            break;
        }
    }
    
    // Extract the context substring
    chars[context_start..context_end].iter().collect()
}

/// Apply coercion logic to just the variant (word-level), not the entire container
fn apply_coercion_to_variant(container: &str, _old_variant: &str, new_variant: &str) -> Option<String> {
    // Detect the container style
    let container_style = crate::coercion::detect_style(container);
    
    // If container has mixed or unknown style, no coercion
    if container_style == crate::coercion::Style::Mixed || container_style == crate::coercion::Style::Dot {
        return None;
    }
    
    // Apply the container's style to the new variant
    let new_tokens = crate::coercion::tokenize(new_variant);
    let coerced_variant = crate::coercion::render_tokens(&new_tokens, container_style);
    
    Some(coerced_variant)
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
        assert_eq!(opts.plan_out, PathBuf::from(".refaktor/plan.json"));
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
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let opts = PlanOptions::default();
        
        let plan = scan_repository(temp_dir.path(), "old", "new", &opts).unwrap();
        
        assert_eq!(plan.old, "old");
        assert_eq!(plan.new, "new");
        assert_eq!(plan.matches.len(), 0);
        assert_eq!(plan.renames.len(), 0);
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
        let hunks = generate_hunks(&matches, content, &variant_map, Path::new("test.txt"), &opts);
        
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].variant, "old_name");
        // The before/after fields contain just the words
        assert_eq!(hunks[0].before, "old_name");
        assert_eq!(hunks[0].after, "new_name");
        // The line context is in separate fields
        assert_eq!(hunks[0].line_before.as_ref().unwrap(), "old_name and oldName here");
        assert_eq!(hunks[0].line_after.as_ref().unwrap(), "new_name and oldName here");
        
        assert_eq!(hunks[1].variant, "oldName");
        assert_eq!(hunks[1].before, "oldName");
        assert_eq!(hunks[1].after, "newName");
        assert_eq!(hunks[1].line_before.as_ref().unwrap(), "old_name and oldName here");
        assert_eq!(hunks[1].line_after.as_ref().unwrap(), "old_name and newName here");
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
        assert_eq!(hunks[0].before, "old_name");
        assert_eq!(hunks[0].after, "new_name");
        assert_eq!(hunks[0].line_before.as_ref().unwrap(), "fn old_name() {");
        assert_eq!(hunks[0].line_after.as_ref().unwrap(), "fn new_name() {");
        
        // Second line replacement
        assert_eq!(hunks[1].variant, "oldName");
        assert_eq!(hunks[1].line, 2);
        assert_eq!(hunks[1].before, "oldName");
        assert_eq!(hunks[1].after, "newName");
        assert_eq!(hunks[1].line_before.as_ref().unwrap(), "    println!(\"oldName\");");
        assert_eq!(hunks[1].line_after.as_ref().unwrap(), "    println!(\"newName\");");
        
        // Third line replacement
        assert_eq!(hunks[2].variant, "old_name");
        assert_eq!(hunks[2].line, 3);
        assert_eq!(hunks[2].before, "old_name");
        assert_eq!(hunks[2].after, "new_name");
        assert_eq!(hunks[2].line_before.as_ref().unwrap(), "    old_name();");
        assert_eq!(hunks[2].line_after.as_ref().unwrap(), "    new_name();");
    }

    #[test]
    fn test_walk_directory() {
        use ignore::Walk;
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        let walker = Walk::new(temp_dir.path());
        let files: Vec<_> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map_or(false, |t| t.is_file()))
            .collect();
            
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path().file_name().unwrap(), "test.txt");
    }
    
    #[test]
    fn test_scan_with_matches() {
        // Create a simple test case
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "old_name and oldName here").unwrap();
        
        // Use non-parallel walk for testing
        use ignore::Walk;
        let walker = Walk::new(temp_dir.path());
        let mut file_count = 0;
        for entry in walker {
            if let Ok(e) = entry {
                if e.file_type().map_or(false, |t| t.is_file()) {
                    file_count += 1;
                }
            }
        }
        assert_eq!(file_count, 1, "Walker should find 1 file");
        
        // Now test with scan_repository
        let mut opts = PlanOptions::default();
        opts.respect_gitignore = false;
        
        let plan = scan_repository(temp_dir.path(), "old_name", "new_name", &opts).unwrap();
        
        // We expect 2 matches: "old_name" and "oldName"
        assert_eq!(plan.matches.len(), 2, "Expected 2 matches, found {}", plan.matches.len());
        assert_eq!(plan.stats.files_scanned, 1);
        assert_eq!(plan.stats.files_with_matches, 1);
    }

    #[test]
    fn test_rename_sorting() {
        let mut renames = vec![
            Rename {
                from: PathBuf::from("/a/b/file.txt"),
                to: PathBuf::from("/a/b/new.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
            Rename {
                from: PathBuf::from("/a/dir"),
                to: PathBuf::from("/a/new_dir"),
                kind: RenameKind::Dir,
                coercion_applied: None,
            },
            Rename {
                from: PathBuf::from("/a/b/c/deep.txt"),
                to: PathBuf::from("/a/b/c/new_deep.txt"),
                kind: RenameKind::File,
                coercion_applied: None,
            },
        ];
        
        renames.sort_by(|a, b| {
            let a_depth = a.from.components().count();
            let b_depth = b.from.components().count();
            
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
        assert_eq!(renames[1].from.components().count(), 5);
    }

    #[test]
    fn test_write_plan() {
        let temp_dir = TempDir::new().unwrap();
        let plan_path = temp_dir.path().join(".refaktor/plan.json");
        
        let plan = Plan {
            id: "test123".to_string(),
            created_at: "123456789".to_string(),
            old: "old".to_string(),
            new: "new".to_string(),
            styles: vec![Style::Snake],
            includes: vec![],
            excludes: vec![],
            matches: vec![],
            renames: vec![],
            stats: Stats {
                files_scanned: 1,
                total_matches: 0,
                matches_by_variant: HashMap::new(),
                files_with_matches: 0,
            },
            version: "1.0.0".to_string(),
        };
        
        write_plan(&plan, &plan_path).unwrap();
        assert!(plan_path.exists());
        
        let content = std::fs::read_to_string(&plan_path).unwrap();
        assert!(content.contains("\"id\": \"test123\""));
    }
}