use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Detected style of an identifier or path segment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CoercionStyle")]
pub enum Style {
    /// `snake_case`
    Snake,
    /// kebab-case
    Kebab,
    /// camelCase
    Camel,
    /// `PascalCase`
    Pascal,
    /// `SCREAMING_SNAKE_CASE`
    ScreamingSnake,
    /// dot.separated
    Dot,
    /// Mixed or unknown style
    Mixed,
}

/// Token representing a word in an identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub word: String,
    pub is_acronym: bool,
}

/// Detect the predominant style of a string
pub fn detect_style(s: &str) -> Style {
    // For filenames, only consider the basename (without extension) for style detection
    // Only treat as file extension if the part after the last dot looks like an extension
    // (short, alphanumeric, and commonly used)
    let basename = if let Some(dot_pos) = s.rfind('.') {
        if dot_pos > 0 && dot_pos < s.len() - 1 {
            let extension = &s[dot_pos + 1..];
            // Common file extensions (not exhaustive, but covers most cases)
            let is_file_extension = extension.len() <= 6
                && extension.chars().all(char::is_alphanumeric)
                && matches!(
                    extension,
                    "rs" | "js"
                        | "ts"
                        | "py"
                        | "java"
                        | "cpp"
                        | "c"
                        | "h"
                        | "txt"
                        | "md"
                        | "json"
                        | "xml"
                        | "html"
                        | "css"
                        | "scss"
                        | "toml"
                        | "yml"
                        | "yaml"
                        | "exe"
                        | "dll"
                        | "so"
                        | "dylib"
                        | "a"
                        | "lib"
                        | "png"
                        | "jpg"
                        | "jpeg"
                        | "gif"
                        | "svg"
                        | "ico"
                        | "pdf"
                );

            if is_file_extension {
                &s[..dot_pos]
            } else {
                s
            }
        } else {
            s
        }
    } else {
        s
    };

    // Count separators in the basename only
    let mut hyphen_count = 0;
    let mut underscore_count = 0;
    let mut dot_count = 0;
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut case_transitions = 0;
    let mut prev_was_lower = false;
    let mut prev_was_upper = false;

    for ch in basename.chars() {
        match ch {
            '-' => hyphen_count += 1,
            '_' => underscore_count += 1,
            '.' => dot_count += 1,
            _ if ch.is_uppercase() => {
                has_uppercase = true;
                if prev_was_lower {
                    case_transitions += 1;
                }
                prev_was_upper = true;
                prev_was_lower = false;
            },
            _ if ch.is_lowercase() => {
                has_lowercase = true;
                if prev_was_upper {
                    case_transitions += 1;
                }
                prev_was_lower = true;
                prev_was_upper = false;
            },
            _ => {
                prev_was_lower = false;
                prev_was_upper = false;
            },
        }
    }

    // Determine style based on patterns
    if hyphen_count > 0 && underscore_count == 0 && dot_count == 0 {
        if has_uppercase && !has_lowercase {
            Style::Mixed // KEBAB-SCREAMING not a standard style
        } else {
            Style::Kebab
        }
    } else if underscore_count > 0 && hyphen_count == 0 && dot_count == 0 {
        if has_uppercase && !has_lowercase {
            Style::ScreamingSnake
        } else {
            Style::Snake
        }
    } else if dot_count > 0 && hyphen_count == 0 && underscore_count == 0 {
        Style::Dot
    } else if hyphen_count == 0 && underscore_count == 0 && dot_count == 0 {
        // No separators, check case pattern
        if case_transitions > 0 {
            // Check if it starts with uppercase
            if basename.chars().next().is_some_and(char::is_uppercase) {
                Style::Pascal
            } else {
                Style::Camel
            }
        } else if has_uppercase && !has_lowercase {
            Style::ScreamingSnake // All caps, treat as screaming snake without underscores
        } else {
            Style::Snake // All lowercase, treat as snake without underscores
        }
    } else {
        Style::Mixed
    }
}

/// Tokenize a string into words
pub fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut prev_was_lower = false;
    let mut prev_was_upper = false;
    let mut consecutive_upper = 0;

    for ch in s.chars() {
        match ch {
            '-' | '_' | '.' | ' ' => {
                // Separator - flush current word
                if !current_word.is_empty() {
                    let is_acronym =
                        current_word.chars().all(char::is_uppercase) && current_word.len() > 1;
                    tokens.push(Token {
                        word: current_word.to_lowercase(),
                        is_acronym,
                    });
                    current_word.clear();
                }
                consecutive_upper = 0;
                prev_was_lower = false;
                prev_was_upper = false;
            },
            _ if ch.is_uppercase() => {
                // Uppercase letter
                if prev_was_lower {
                    // Transition from lowercase to uppercase - new word
                    if !current_word.is_empty() {
                        tokens.push(Token {
                            word: current_word.to_lowercase(),
                            is_acronym: false,
                        });
                        current_word.clear();
                    }
                } else if prev_was_upper && consecutive_upper > 1 {
                    // Check if this might be the start of a new word after an acronym
                    // We look ahead to see if the next char is lowercase
                    // For now, just add to current word
                }
                current_word.push(ch);
                consecutive_upper += 1;
                prev_was_upper = true;
                prev_was_lower = false;
            },
            _ if ch.is_lowercase() => {
                // Lowercase letter
                if prev_was_upper && consecutive_upper > 1 {
                    // End of acronym, start of new word
                    // Move last uppercase to new word
                    let last_upper = current_word.pop().unwrap();
                    if !current_word.is_empty() {
                        tokens.push(Token {
                            word: current_word.to_lowercase(),
                            is_acronym: true,
                        });
                    }
                    current_word.clear();
                    current_word.push(last_upper);
                }
                current_word.push(ch);
                consecutive_upper = 0;
                prev_was_lower = true;
                prev_was_upper = false;
            },
            _ if ch.is_alphanumeric() => {
                current_word.push(ch);
                consecutive_upper = 0;
                prev_was_lower = false;
                prev_was_upper = false;
            },
            _ => {
                // Other character - treat as separator
                if !current_word.is_empty() {
                    let is_acronym =
                        current_word.chars().all(char::is_uppercase) && current_word.len() > 1;
                    tokens.push(Token {
                        word: current_word.to_lowercase(),
                        is_acronym,
                    });
                    current_word.clear();
                }
                consecutive_upper = 0;
                prev_was_lower = false;
                prev_was_upper = false;
            },
        }
    }

    // Flush remaining word
    if !current_word.is_empty() {
        let is_acronym = current_word.chars().all(char::is_uppercase) && current_word.len() > 1;
        tokens.push(Token {
            word: current_word.to_lowercase(),
            is_acronym,
        });
    }

    tokens
}

/// Render tokens in a specific style
pub fn render_tokens(tokens: &[Token], style: Style) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    match style {
        Style::Snake => tokens
            .iter()
            .map(|t| t.word.clone())
            .collect::<Vec<_>>()
            .join("_"),
        Style::Kebab => tokens
            .iter()
            .map(|t| t.word.clone())
            .collect::<Vec<_>>()
            .join("-"),
        Style::Camel => {
            let mut result = String::new();
            for (i, token) in tokens.iter().enumerate() {
                if i == 0 {
                    result.push_str(&token.word);
                } else {
                    result.push_str(&capitalize(&token.word));
                }
            }
            result
        },
        Style::Pascal => tokens
            .iter()
            .map(|t| capitalize(&t.word))
            .collect::<String>(),
        Style::ScreamingSnake => tokens
            .iter()
            .map(|t| t.word.to_uppercase())
            .collect::<Vec<_>>()
            .join("_"),
        Style::Dot => tokens
            .iter()
            .map(|t| t.word.clone())
            .collect::<Vec<_>>()
            .join("."),
        Style::Mixed => {
            // Default to snake case for mixed styles
            render_tokens(tokens, Style::Snake)
        },
    }
}

/// Apply contextual separator coercion
pub fn apply_coercion(
    container: &str,
    old_pattern: &str,
    new_pattern: &str,
) -> Option<(String, String)> {
    // If the container is exactly the same as the pattern, no meaningful coercion
    if container.to_lowercase() == old_pattern.to_lowercase() {
        return None;
    }

    // Check if the container contains the old pattern
    let container_lower = container.to_lowercase();
    let old_pattern_lower = old_pattern.to_lowercase();

    if !container_lower.contains(&old_pattern_lower) {
        return None;
    }

    // Detect the container style first
    let container_style = detect_style(container);
    let pattern_style = detect_style(old_pattern);

    // Special case: If container has a hyphen suffix after the pattern (e.g., FooBar-specific)
    // and the container has mixed/unknown style, treat this as a partial replacement
    // But only if the container doesn't have other separator types (underscore, dot)
    let is_partial_with_suffix = {
        if container_style == Style::Mixed
            && container.contains('-')
            && !container.contains('_')
            && !container.contains('.')
        {
            if let Some(pos) = container_lower.find(&old_pattern_lower) {
                let end_pos = pos + old_pattern_lower.len();
                end_pos < container.len() && container.chars().nth(end_pos) == Some('-')
            } else {
                false
            }
        } else {
            false
        }
    };

    if is_partial_with_suffix {
        // For partial patterns with hyphen suffixes in mixed-style containers,
        // preserve the original style of the pattern part and keep the suffix as-is
        let _old_tokens = tokenize(old_pattern);
        let new_tokens = tokenize(new_pattern);

        // Find the style of just the pattern part in the container
        if let Some(pos) = container_lower.find(&old_pattern_lower) {
            let pattern_part = &container[pos..pos + old_pattern.len()];
            let pattern_style = detect_style(pattern_part);

            // Don't coerce if the pattern part has mixed/unknown style
            if pattern_style == Style::Mixed || pattern_style == Style::Dot {
                return None;
            }

            // Render the new pattern in the detected style
            let coerced_new = render_tokens(&new_tokens, pattern_style);

            // Replace all occurrences (case-insensitive)
            let result = replace_case_insensitive(container, old_pattern, &coerced_new);

            return Some((
                result,
                format!("partial coercion to {:?} style", pattern_style),
            ));
        }
    }

    // If container has mixed or unknown style, no coercion
    // Also skip dot-case by default (risky for file extensions)
    if container_style == Style::Mixed || container_style == Style::Dot {
        return None;
    }

    // Prefer the style of the exact matched segment when it is well-defined.
    // This preserves embedded PascalCase segments inside larger camelCase identifiers
    // like getAdminDeployRequestsParams.
    let target_style = if pattern_style == Style::Pascal
        && matches!(container_style, Style::Camel | Style::Pascal)
    {
        Style::Pascal
    } else {
        container_style
    };

    // Tokenize the patterns
    let _old_tokens = tokenize(old_pattern);
    let new_tokens = tokenize(new_pattern);

    // Render the new pattern in the chosen style
    let coerced_new = render_tokens(&new_tokens, target_style);

    // Replace all occurrences (case-insensitive)
    let result = replace_case_insensitive(container, old_pattern, &coerced_new);

    // Return the coercion details
    Some((result, format!("coerced to {:?} style", target_style)))
}

/// Replace all occurrences of pattern with replacement (case-insensitive)
fn replace_case_insensitive(text: &str, pattern: &str, replacement: &str) -> String {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    let mut result = String::new();
    let mut last_end = 0;

    while let Some(start) = text_lower[last_end..].find(&pattern_lower) {
        let absolute_start = last_end + start;
        let absolute_end = absolute_start + pattern.len();

        // Add the part before the match
        result.push_str(&text[last_end..absolute_start]);

        // Add the replacement
        result.push_str(replacement);

        last_end = absolute_end;
    }

    // Add the remaining part
    result.push_str(&text[last_end..]);

    result
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_style() {
        assert_eq!(detect_style("snake_case"), Style::Snake);
        assert_eq!(detect_style("kebab-case"), Style::Kebab);
        assert_eq!(detect_style("camelCase"), Style::Camel);
        assert_eq!(detect_style("PascalCase"), Style::Pascal);
        assert_eq!(detect_style("SCREAMING_SNAKE"), Style::ScreamingSnake);
        assert_eq!(detect_style("dot.separated"), Style::Dot);
        assert_eq!(detect_style("mixed-style_here"), Style::Mixed);
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("foo-bar");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].word, "foo");
        assert_eq!(tokens[1].word, "bar");

        let tokens = tokenize("getUserName");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].word, "get");
        assert_eq!(tokens[1].word, "user");
        assert_eq!(tokens[2].word, "name");

        let tokens = tokenize("HTTPSConnection");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].word, "https");
        assert!(tokens[0].is_acronym);
        assert_eq!(tokens[1].word, "connection");
    }

    #[test]
    fn test_render_tokens() {
        let tokens = vec![
            Token {
                word: "renamed".to_string(),
                is_acronym: false,
            },
            Token {
                word: "renaming".to_string(),
                is_acronym: false,
            },
            Token {
                word: "tool".to_string(),
                is_acronym: false,
            },
        ];

        assert_eq!(
            render_tokens(&tokens, Style::Snake),
            "renamed_renaming_tool"
        );
        assert_eq!(
            render_tokens(&tokens, Style::Kebab),
            "renamed-renaming-tool"
        );
        assert_eq!(render_tokens(&tokens, Style::Camel), "renamedRenamingTool");
        assert_eq!(render_tokens(&tokens, Style::Pascal), "RenamedRenamingTool");
        assert_eq!(
            render_tokens(&tokens, Style::ScreamingSnake),
            "RENAMED_RENAMING_TOOL"
        );
        assert_eq!(render_tokens(&tokens, Style::Dot), "renamed.renaming.tool");
    }

    #[test]
    fn test_apply_coercion() {
        // Test kebab-case container
        let result = apply_coercion("lorem-ipsum", "lorem", "dolor");
        assert_eq!(
            result,
            Some((
                "dolor-ipsum".to_string(),
                "coerced to Kebab style".to_string()
            ))
        );

        // Test snake_case container
        let result = apply_coercion("lorem_ipsum", "lorem", "dolor");
        assert_eq!(
            result,
            Some((
                "dolor_ipsum".to_string(),
                "coerced to Snake style".to_string()
            ))
        );

        // Test PascalCase container
        let result = apply_coercion("LoremIpsum", "Lorem", "Dolor");
        assert_eq!(
            result,
            Some((
                "DolorIpsum".to_string(),
                "coerced to Pascal style".to_string()
            ))
        );
    }
}
