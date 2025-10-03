use super::{
    cross_file_context::CrossFileContextAnalyzer, file_context::FileContextAnalyzer,
    get_possible_styles, is_ambiguous, language_heuristics::LanguageHeuristics,
};
use crate::case_model::{detect_style, parse_to_tokens, to_style, Style};

/// Context for resolving ambiguity
#[derive(Debug, Clone, Default)]
pub struct AmbiguityContext {
    pub file_path: Option<std::path::PathBuf>,
    pub file_content: Option<String>,
    pub line_content: Option<String>,
    pub match_position: Option<usize>,
    pub project_root: Option<std::path::PathBuf>,
}

/// Result of ambiguity resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStyle {
    pub style: Style,
    pub confidence: ResolutionConfidence,
    pub method: ResolutionMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionMethod {
    NotAmbiguous,
    LanguageHeuristic,
    FileContext,
    CrossFileContext,
    ReplacementStringPreference,
    DefaultFallback,
}

/// Main ambiguity resolver that orchestrates all resolution strategies
pub struct AmbiguityResolver {
    file_analyzer: FileContextAnalyzer,
    cross_file_analyzer: CrossFileContextAnalyzer,
}

impl Default for AmbiguityResolver {
    fn default() -> Self {
        Self {
            file_analyzer: FileContextAnalyzer::new(),
            cross_file_analyzer: CrossFileContextAnalyzer::new(),
        }
    }
}

impl AmbiguityResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve ambiguity for a matched text
    pub fn resolve(
        &self,
        matched_text: &str,
        replacement_text: &str,
        context: &AmbiguityContext,
    ) -> ResolvedStyle {
        self.resolve_with_styles(matched_text, replacement_text, context, None)
    }

    /// Resolution method with pre-calculated replacement styles for performance
    pub fn resolve_with_styles(
        &self,
        matched_text: &str,
        replacement_text: &str,
        context: &AmbiguityContext,
        replacement_possible_styles: Option<&[Style]>,
    ) -> ResolvedStyle {
        if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
            eprintln!(
                "=== Resolving ambiguity for '{}' -> '{}' ===",
                matched_text, replacement_text
            );
            eprintln!("  File: {:?}", context.file_path);
            eprintln!("  Content: {:?}", context.line_content);
        }

        // First check if it's even ambiguous
        if !is_ambiguous(matched_text) {
            if let Some(style) = detect_style(matched_text) {
                if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                    eprintln!("  -> Not ambiguous, detected style: {:?}", style);
                }
                return ResolvedStyle {
                    style,
                    confidence: ResolutionConfidence::High,
                    method: ResolutionMethod::NotAmbiguous,
                };
            }
        }

        // Get possible styles for the matched text
        let possible_styles = get_possible_styles(matched_text);

        // CRITICAL: Filter styles by case constraints
        // All-uppercase text (e.g., "TESTWORD") can ONLY match uppercase styles
        // (ScreamingSnake, ScreamingTrain, UpperFlat, UpperSentence), NEVER Title/Pascal/Camel
        let constrained_styles = crate::case_constraints::filter_compatible_styles(
            matched_text,
            &possible_styles,
        );

        if constrained_styles.is_empty() {
            // Shouldn't happen, but handle gracefully
            return Self::default_fallback(
                &possible_styles,
                replacement_text,
                replacement_possible_styles,
            );
        }

        if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
            eprintln!("  Possible styles (before constraints): {:?}", possible_styles);
            eprintln!("  Constrained styles (after filtering): {:?}", constrained_styles);
        }

        // Level 1: Language-specific heuristics
        if let Some(resolved) =
            Self::try_language_heuristics(matched_text, context, &constrained_styles)
        {
            if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                eprintln!("  -> Resolved by language heuristics: {:?}", resolved.style);
            }
            return resolved;
        }

        // Level 2: File context analysis
        if let Some(resolved) = self.try_file_context(matched_text, context, &constrained_styles) {
            if std::env::var("RENAMIFY_DEBUG_AMBIGUITY").is_ok() {
                eprintln!("  -> Resolved by file context: {:?}", resolved.style);
            }
            return resolved;
        }

        // Level 3: Cross-file context analysis
        if let Some(resolved) = self.try_cross_file_context(matched_text, context, &constrained_styles)
        {
            return resolved;
        }

        // Level 4: Replacement string preference (ultimate fallback)
        Self::try_replacement_preference(
            &constrained_styles,
            replacement_text,
            replacement_possible_styles,
        )
    }

    /// Try language-specific heuristics
    fn try_language_heuristics(
        _matched_text: &str,
        context: &AmbiguityContext,
        possible_styles: &[Style],
    ) -> Option<ResolvedStyle> {
        let file_path = context.file_path.as_ref()?;
        let line = context.line_content.as_ref()?;
        let match_pos = context.match_position?;

        // Extract preceding context
        let preceding = if match_pos > 0 {
            &line[..match_pos]
        } else {
            ""
        };

        if let Some(style) =
            LanguageHeuristics::suggest_style(file_path, preceding, possible_styles)
        {
            return Some(ResolvedStyle {
                style,
                confidence: ResolutionConfidence::High,
                method: ResolutionMethod::LanguageHeuristic,
            });
        }

        None
    }

    /// Try file context analysis
    fn try_file_context(
        &self,
        _matched_text: &str,
        context: &AmbiguityContext,
        possible_styles: &[Style],
    ) -> Option<ResolvedStyle> {
        let file_content = context.file_content.as_ref()?;

        if let Some(style) = self
            .file_analyzer
            .suggest_style(file_content, possible_styles)
        {
            // Get confidence from file analysis
            let stats = self.file_analyzer.analyze(file_content);
            let confidence = match stats.confidence {
                super::file_context::ConfidenceLevel::High => ResolutionConfidence::High,
                super::file_context::ConfidenceLevel::Medium => ResolutionConfidence::Medium,
                _ => ResolutionConfidence::Low,
            };

            return Some(ResolvedStyle {
                style,
                confidence,
                method: ResolutionMethod::FileContext,
            });
        }

        None
    }

    /// Try cross-file context analysis
    fn try_cross_file_context(
        &self,
        _matched_text: &str,
        context: &AmbiguityContext,
        possible_styles: &[Style],
    ) -> Option<ResolvedStyle> {
        let project_root = context.project_root.as_ref()?;
        let file_path = context.file_path.as_ref()?;
        let line = context.line_content.as_ref()?;
        let match_pos = context.match_position?;

        let extension = file_path.extension()?.to_str()?;

        // Extract preceding word
        let preceding = if match_pos > 0 {
            &line[..match_pos]
        } else {
            ""
        };

        // Find the last word before the match
        let preceding_word = preceding.split_whitespace().last().unwrap_or("");

        if !preceding_word.is_empty() {
            if let Some(style) = self.cross_file_analyzer.suggest_style(
                project_root,
                false,
                extension,
                preceding_word,
                possible_styles,
            ) {
                return Some(ResolvedStyle {
                    style,
                    confidence: ResolutionConfidence::Medium,
                    method: ResolutionMethod::CrossFileContext,
                });
            }
        }

        None
    }

    /// Try replacement string preference
    fn try_replacement_preference(
        possible_styles: &[Style],
        replacement_text: &str,
        replacement_possible_styles: Option<&[Style]>,
    ) -> ResolvedStyle {
        // Determine the style of the replacement string
        if let Some(replacement_style) = detect_style(replacement_text) {
            // If the replacement style is one of the possible styles, use it
            if possible_styles.contains(&replacement_style) {
                return ResolvedStyle {
                    style: replacement_style,
                    confidence: ResolutionConfidence::Medium,
                    method: ResolutionMethod::ReplacementStringPreference,
                };
            }
        }

        // Otherwise use default fallback
        Self::default_fallback(
            possible_styles,
            replacement_text,
            replacement_possible_styles,
        )
    }

    /// Default fallback when all else fails
    fn default_fallback(
        possible_styles: &[Style],
        replacement_text: &str,
        replacement_possible_styles: Option<&[Style]>,
    ) -> ResolvedStyle {
        // Define default precedence order at the top
        const DEFAULT_PRECEDENCE: &[Style] = &[
            Style::Snake,
            Style::Camel,
            Style::Pascal,
            Style::Kebab,
            Style::ScreamingSnake,
            Style::Train,
            Style::ScreamingTrain,
            Style::Title,
            Style::Dot,
        ];

        // Use pre-calculated styles if provided, otherwise calculate them
        let calculated_styles;
        let replacement_styles = if let Some(styles) = replacement_possible_styles {
            styles
        } else {
            calculated_styles = get_possible_styles(replacement_text);
            &calculated_styles
        };

        // Find styles that are possible for both the matched text and replacement
        let common_styles: Vec<Style> = possible_styles
            .iter()
            .filter(|s| replacement_styles.contains(s))
            .copied()
            .collect();

        // If there's exactly one common style, that's our answer
        if common_styles.len() == 1 {
            return ResolvedStyle {
                style: common_styles[0],
                confidence: ResolutionConfidence::High,
                method: ResolutionMethod::DefaultFallback,
            };
        }

        // If there are multiple common styles, prefer the one from the replacement
        // Check if replacement has a definitive (non-ambiguous) style
        if !common_styles.is_empty() {
            if let Some(replacement_style) = detect_style(replacement_text) {
                if common_styles.contains(&replacement_style) {
                    return ResolvedStyle {
                        style: replacement_style,
                        confidence: ResolutionConfidence::Medium,
                        method: ResolutionMethod::DefaultFallback,
                    };
                }
            }
            // Otherwise use the first common style from precedence order
            for &style in DEFAULT_PRECEDENCE {
                if common_styles.contains(&style) {
                    return ResolvedStyle {
                        style,
                        confidence: ResolutionConfidence::Medium,
                        method: ResolutionMethod::DefaultFallback,
                    };
                }
            }
        }

        // No common styles - fall back to precedence order from possible_styles
        // Find the first style in our precedence order that's possible
        for &style in DEFAULT_PRECEDENCE {
            if possible_styles.contains(&style) {
                return ResolvedStyle {
                    style,
                    confidence: ResolutionConfidence::Low,
                    method: ResolutionMethod::DefaultFallback,
                };
            }
        }

        // Last resort: just use the first possible style
        if let Some(&style) = possible_styles.first() {
            return ResolvedStyle {
                style,
                confidence: ResolutionConfidence::Low,
                method: ResolutionMethod::DefaultFallback,
            };
        }

        // Ultra last resort (shouldn't happen)
        ResolvedStyle {
            style: Style::LowerFlat,
            confidence: ResolutionConfidence::Low,
            method: ResolutionMethod::DefaultFallback,
        }
    }

    /// Convert ambiguous text to resolved style
    pub fn apply_resolution(
        _matched_text: &str,
        replacement_text: &str,
        resolved_style: &ResolvedStyle,
    ) -> String {
        // Parse the replacement into tokens
        let tokens = parse_to_tokens(replacement_text);

        // Apply the resolved style
        to_style(&tokens, resolved_style.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_not_ambiguous() {
        let resolver = AmbiguityResolver::new();
        let context = AmbiguityContext::default();

        let result = resolver.resolve("user_id", "account_id", &context);
        assert_eq!(result.method, ResolutionMethod::NotAmbiguous);
        assert_eq!(result.style, Style::Snake);
    }

    #[test]
    fn test_replacement_preference() {
        let resolver = AmbiguityResolver::new();
        let context = AmbiguityContext::default();

        // "api" is ambiguous (could be snake, camel, kebab, or lower)
        // Replacement is snake_case
        let result = resolver.resolve("api", "application_interface", &context);
        assert_eq!(result.style, Style::Snake);
        assert_eq!(result.method, ResolutionMethod::ReplacementStringPreference);
    }

    #[test]
    fn test_language_heuristic_ruby() {
        let resolver = AmbiguityResolver::new();
        let context = AmbiguityContext {
            file_path: Some(PathBuf::from("test.rb")),
            line_content: Some("class API".to_string()),
            match_position: Some(6), // Position after "class "
            ..Default::default()
        };

        // "API" is a known acronym, so it CAN match PascalCase
        // In Ruby, "class API" is valid - API is a PascalCase class name
        // The language heuristic should recognize this and return Pascal
        let result = resolver.resolve("API", "Interface", &context);
        assert_eq!(result.style, Style::Pascal);
        assert_eq!(result.method, ResolutionMethod::LanguageHeuristic);
    }

    #[test]
    fn test_apply_resolution() {
        let _resolver = AmbiguityResolver::new();

        let resolved = ResolvedStyle {
            style: Style::Camel,
            confidence: ResolutionConfidence::High,
            method: ResolutionMethod::ReplacementStringPreference,
        };

        let result = AmbiguityResolver::apply_resolution(
            "api",
            "application_programming_interface",
            &resolved,
        );

        assert_eq!(result, "applicationProgrammingInterface");
    }

    #[test]
    fn test_impossible_replacement_style() {
        let resolver = AmbiguityResolver::new();
        let context = AmbiguityContext::default();

        // "api" starts with lowercase, can't be PascalCase
        // But replacement is PascalCase
        let result = resolver.resolve("api", "ApplicationInterface", &context);

        // Should fall back to a possible style
        assert!(matches!(result.method, ResolutionMethod::DefaultFallback));
        assert!(result.style != Style::Pascal); // Can't be Pascal since "api" starts lowercase
    }

    #[test]
    fn test_uppercase_must_stay_uppercase() {
        let resolver = AmbiguityResolver::new();

        // Context: Title Case comment
        let context = AmbiguityContext {
            file_path: Some(PathBuf::from("test.rs")),
            line_content: Some("// Testword Core Engine".to_string()),
            match_position: Some(3),
            ..Default::default()
        };

        // "TESTWORD" is ambiguous (could be UpperFlat, UpperSentence, ScreamingSnake)
        // But it's all uppercase, so resolution should ONLY pick uppercase styles
        let result = resolver.resolve("TESTWORD", "module", &context);

        // The resolved style must be an uppercase style
        assert!(
            matches!(
                result.style,
                Style::ScreamingSnake | Style::UpperFlat | Style::UpperSentence | Style::ScreamingTrain
            ),
            "All uppercase matched text must resolve to uppercase style, got: {:?}",
            result.style
        );
    }

    #[test]
    fn test_uppercase_in_uppercase_context() {
        let resolver = AmbiguityResolver::new();

        // Context: All uppercase sentence
        let context = AmbiguityContext {
            file_path: Some(PathBuf::from("test.rs")),
            line_content: Some("// TESTWORD CORE ENGINE".to_string()),
            match_position: Some(3),
            ..Default::default()
        };

        let result = resolver.resolve("TESTWORD", "module", &context);

        // Should resolve to an uppercase style
        assert!(
            matches!(
                result.style,
                Style::ScreamingSnake | Style::UpperFlat | Style::UpperSentence | Style::ScreamingTrain
            ),
            "Uppercase in uppercase context should stay uppercase, got: {:?}",
            result.style
        );
    }

    #[test]
    fn test_lowercase_must_stay_lowercase() {
        let resolver = AmbiguityResolver::new();

        // Context: PascalCase
        let context = AmbiguityContext {
            file_path: Some(PathBuf::from("test.rs")),
            line_content: Some("TestwordCoreEngine".to_string()),
            match_position: Some(0),
            ..Default::default()
        };

        // "testword" is ambiguous but all lowercase
        // Even though context is PascalCase, we can't make "testword" into "Testword"
        // if it was matched as lowercase
        let result = resolver.resolve("testword", "Module", &context);

        // Should resolve to a lowercase style (snake, kebab, lower_flat, lower_sentence)
        // NOT Pascal, Camel, or Title
        assert!(
            matches!(
                result.style,
                Style::Snake | Style::Kebab | Style::LowerFlat | Style::LowerSentence | Style::Dot
            ),
            "All lowercase matched text must resolve to lowercase style, got: {:?}",
            result.style
        );
    }

    #[test]
    fn test_mixed_case_flexible() {
        let resolver = AmbiguityResolver::new();

        // Context: snake_case
        let context = AmbiguityContext {
            file_path: Some(PathBuf::from("test.rs")),
            line_content: Some("testword_core_engine".to_string()),
            match_position: Some(0),
            ..Default::default()
        };

        // "Testword" starts with uppercase, so it can only match styles that allow
        // first letter uppercase: Pascal, Train, Title, UpperFlat, etc.
        // It CANNOT match Snake (requires all lowercase)
        let result = resolver.resolve("Testword", "module", &context);

        // Should resolve to Pascal (starts uppercase, no consecutive uppercase)
        // NOT Snake because "Testword" has an uppercase letter
        assert_eq!(result.style, Style::Pascal);
    }
}
