use crate::case_model::{detect_style, Style};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

// Global cache for cross-file analysis results
static CONTEXT_CACHE: LazyLock<Mutex<HashMap<String, CachedAnalysis>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug)]
struct CachedAnalysis {
    patterns: HashMap<String, HashMap<Style, usize>>,
}

/// Analyzes patterns across multiple files with the same extension
pub struct CrossFileContextAnalyzer {
    max_files_to_scan: usize,
    min_pattern_occurrences: usize,
}

impl Default for CrossFileContextAnalyzer {
    fn default() -> Self {
        Self {
            max_files_to_scan: 20,
            min_pattern_occurrences: 3,
        }
    }
}

impl CrossFileContextAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze cross-file context for a given pattern
    pub fn suggest_style(
        &self,
        project_root: &Path,
        include_hidden: bool,
        file_extension: &str,
        preceding_word: &str,
        possible_styles: &[Style],
    ) -> Option<Style> {
        // Create cache key
        let cache_key = format!("{}:{}", file_extension, preceding_word);

        // Check cache first
        if let Ok(cache) = CONTEXT_CACHE.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                    eprintln!(
                        "DEBUG CrossFileContextAnalyzer: Cache hit for {}",
                        cache_key
                    );
                }

                return self.pick_best_style_from_patterns(
                    &cached.patterns,
                    preceding_word,
                    possible_styles,
                );
            }
        }

        // Perform analysis
        let patterns = self.analyze_pattern_in_files(
            project_root,
            include_hidden,
            file_extension,
            preceding_word,
        );

        // Cache the result
        if let Ok(mut cache) = CONTEXT_CACHE.lock() {
            cache.insert(
                cache_key,
                CachedAnalysis {
                    patterns: patterns.clone(),
                },
            );
        }

        self.pick_best_style_from_patterns(&patterns, preceding_word, possible_styles)
    }

    /// Analyze how a pattern is used across files
    fn analyze_pattern_in_files(
        &self,
        project_root: &Path,
        include_hidden: bool,
        file_extension: &str,
        preceding_word: &str,
    ) -> HashMap<String, HashMap<Style, usize>> {
        let mut patterns = HashMap::new();
        let mut files_scanned = 0;

        // Find files with the same extension
        if let Ok(entries) =
            self.find_files_with_extension(project_root, include_hidden, file_extension)
        {
            for path in entries {
                if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                    eprintln!(
                        "DEBUG CrossFileContextAnalyzer: Checking file: {}",
                        path.display()
                    );
                }

                if files_scanned >= self.max_files_to_scan {
                    break;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    Self::extract_patterns_from_content(&content, preceding_word, &mut patterns);
                    files_scanned += 1;
                }
            }
        }

        patterns
    }

    /// Find all files with a specific extension in the project
    fn find_files_with_extension(
        &self,
        root: &Path,
        include_hidden: bool,
        extension: &str,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut files = Vec::new();
        self.find_files_recursive(root, include_hidden, extension, &mut files, 0)?;
        if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
            eprintln!(
                "DEBUG CrossFileContextAnalyzer: Found {} files with extension {}",
                files.len(),
                extension
            );
        }
        Ok(files)
    }

    fn find_files_recursive(
        &self,
        dir: &Path,
        include_hidden: bool,
        extension: &str,
        files: &mut Vec<PathBuf>,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        // Limit recursion depth
        if depth > 5 || files.len() >= self.max_files_to_scan {
            return Ok(());
        }

        if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
            eprintln!(
                "DEBUG CrossFileContextAnalyzer: Searching for files with extension {} in {}. Depth: {}, files.len(): {}",
                extension, dir.display(), depth, files.len());
        }

        // Skip common directories we don't want to scan
        // TODO: Use existing ripgrep library to filter paths like we do elsewhere - ignore .gitignore, etc.
        // e.g. use ignore::WalkBuilder;
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            if (!include_hidden && dir_name.starts_with('.'))
                || dir_name == "node_modules"
                || dir_name == "target"
                || dir_name == "build"
                || dir_name == "dist"
                || dir_name == "vendor"
            {
                if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                    eprintln!("DEBUG CrossFileContextAnalyzer: Skipping dir");
                }
                return Ok(());
            }
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                eprintln!("DEBUG CrossFileContextAnalyzer: entry: {}", path.display());
            }

            if path.is_dir() {
                self.find_files_recursive(&path, include_hidden, extension, files, depth + 1)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Extract patterns from file content
    fn extract_patterns_from_content(
        content: &str,
        preceding_word: &str,
        patterns: &mut HashMap<String, HashMap<Style, usize>>,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        for line in lines {
            if let Some(pos) = line.find(preceding_word) {
                let after_pos = pos + preceding_word.len();

                // Skip if not followed by whitespace
                if after_pos >= line.len() {
                    continue;
                }

                let rest = &line[after_pos..];
                if !rest.starts_with(|c: char| c.is_whitespace()) {
                    continue;
                }

                // Extract the identifier after the keyword
                let identifier = rest
                    .trim_start()
                    .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                    .next()
                    .unwrap_or("")
                    .to_string();

                if identifier.len() > 1 {
                    if let Some(style) = detect_style(&identifier) {
                        let pattern_key = format!("{} <identifier>", preceding_word);
                        patterns
                            .entry(pattern_key)
                            .or_default()
                            .entry(style)
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
            }
        }
    }

    /// Pick the best style from analyzed patterns
    fn pick_best_style_from_patterns(
        &self,
        patterns: &HashMap<String, HashMap<Style, usize>>,
        preceding_word: &str,
        possible_styles: &[Style],
    ) -> Option<Style> {
        let pattern_key = format!("{} <identifier>", preceding_word);

        if let Some(style_counts) = patterns.get(&pattern_key) {
            // Sort styles by frequency
            let mut sorted: Vec<(&Style, &usize)> = style_counts.iter().collect();
            sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

            // Find the most common style that's also possible
            for (style, count) in sorted {
                if *count >= self.min_pattern_occurrences && possible_styles.contains(style) {
                    return Some(*style);
                }
            }
        }

        None
    }

    /// Clear the cache (useful for testing)
    #[cfg(test)]
    pub fn clear_cache() {
        if let Ok(mut cache) = CONTEXT_CACHE.lock() {
            cache.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_cross_file_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create some test files
        let file1 = root.join("test1.js");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "const userId = 123;").unwrap();
        writeln!(f1, "const userName = 'John';").unwrap();
        writeln!(f1, "const userEmail = 'john@example.com';").unwrap();
        f1.sync_all().ok();

        let file2 = root.join("test2.js");
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "const productId = 456;").unwrap();
        writeln!(f2, "const productName = 'Widget';").unwrap();
        f2.sync_all().ok();

        if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
            eprintln!(
                "DEBUG CrossFileContextAnalyzer: Created test files: {}, {}",
                file1.display(),
                file1.display()
            );
        }

        let mut analyzer = CrossFileContextAnalyzer::new();
        analyzer.min_pattern_occurrences = 2; // Lower threshold for test
        CrossFileContextAnalyzer::clear_cache();

        // Test that it finds camelCase pattern after "const"
        let result =
            analyzer.suggest_style(root, true, "js", "const", &[Style::Camel, Style::Snake]);

        assert_eq!(result, Some(Style::Camel));
    }
}
