/* biome-ignore-all lint/suspicious/noControlCharactersInRegex: Using control chars as safe placeholders */
/* biome-ignore-all lint/style/useTemplate: String concatenation is clearer for building HTML */
/* biome-ignore-all lint/correctness/noUnusedFunctionParameters: Parameters kept for API consistency */

export function escapeHtml(text: string): string {
  if (!text) {
    return '';
  }
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function highlightSearchTerm(text: string, searchTerm: string): string {
  if (!searchTerm) {
    return text;
  }
  const searchRegex = new RegExp(escapeRegExp(searchTerm), 'gi');
  return text.replace(
    searchRegex,
    (m) => `<span class="search-highlight">${m}</span>`
  );
}

export function highlightReplaceTerm(
  text: string,
  matches: MatchHunk[]
): string {
  if (!matches) {
    return text;
  }

  let highlightedText = text;

  // Collect all unique replacement terms from the matches
  const replacementTerms = new Set<string>();
  for (const match of matches) {
    if (match.replace) {
      replacementTerms.add(match.replace);
    }
  }

  // Highlight each replacement term
  for (const replaceTerm of replacementTerms) {
    const replaceRegex = new RegExp(escapeRegExp(replaceTerm), 'gi');
    highlightedText = highlightedText.replace(
      replaceRegex,
      (m) => `<span class="replace-highlight">${m}</span>`
    );
  }

  return highlightedText;
}

// Apply text replacements to create the final text
export function applyReplacements(
  originalText: string,
  matches: MatchHunk[]
): string {
  let finalText = originalText;

  // Sort matches by column position
  const sortedMatches = [...matches].sort(
    (a, b) => a.char_offset - b.char_offset
  );

  // Apply replacements from right to left to avoid position shifts
  for (let i = sortedMatches.length - 1; i >= 0; i--) {
    const match = sortedMatches[i];
    if (match.replace && match.content) {
      // col is 0-indexed from Rust
      const lineStart = match.char_offset;
      const lineEnd = lineStart + match.content.length;

      const before = finalText.substring(0, lineStart);
      const after = finalText.substring(lineEnd);
      finalText = before + match.replace + after;
    }
  }

  return finalText;
}

// Insert placeholders for search highlights
export function insertSearchPlaceholders(
  text: string,
  matches: MatchHunk[]
): string {
  let result = text;

  // Sort matches by position (right to left) to avoid position shifts when inserting
  const sortedMatches = [...matches].sort(
    (a, b) => b.char_offset - a.char_offset
  );

  for (const match of sortedMatches) {
    if (match.content) {
      // char_offset is 0-indexed from Rust
      const lineStart = match.char_offset;
      const lineEnd = lineStart + match.content.length;

      const before = result.substring(0, lineStart);
      const matchContent = result.substring(lineStart, lineEnd);
      const after = result.substring(lineEnd);

      result =
        before +
        '\x00SEARCH_START\x00' +
        matchContent +
        '\x00SEARCH_END\x00' +
        after;
    }
  }

  return result;
}

// Calculate replacement positions in the final text
export function calculateReplacementPositions(
  matches: MatchHunk[]
): Array<{ pos: number; length: number }> {
  const replacements: Array<{ pos: number; length: number }> = [];

  // Sort matches by column position
  const sortedMatches = [...matches].sort(
    (a, b) => a.char_offset - b.char_offset
  );

  let cumulativeShift = 0;
  for (const match of sortedMatches) {
    if (match.replace && match.content) {
      const originalPos = match.char_offset; // char_offset is 0-indexed
      const finalPos = originalPos + cumulativeShift;

      replacements.push({
        pos: finalPos,
        length: match.replace.length,
      });

      // Update shift for next replacement
      const lengthDiff = match.replace.length - match.content.length;
      cumulativeShift += lengthDiff;
    }
  }

  return replacements;
}

// Insert placeholders for replacement highlights
export function insertReplacementPlaceholders(
  text: string,
  positions: Array<{ pos: number; length: number }>
): string {
  let result = text;

  // Insert from right to left to avoid position shifts
  for (let i = positions.length - 1; i >= 0; i--) {
    const repl = positions[i];
    const before = result.substring(0, repl.pos);
    const content = result.substring(repl.pos, repl.pos + repl.length);
    const after = result.substring(repl.pos + repl.length);

    result =
      before +
      '\x00REPLACE_START\x00' +
      content +
      '\x00REPLACE_END\x00' +
      after;
  }

  return result;
}

// Convert placeholders to HTML spans
export function placeholdersToHtml(text: string): string {
  let result = escapeHtml(text);

  result = result
    .replace(/\x00SEARCH_START\x00/g, '<span class="search-highlight">')
    .replace(/\x00SEARCH_END\x00/g, '</span>')
    .replace(/\x00REPLACE_START\x00/g, '<span class="replace-highlight">')
    .replace(/\x00REPLACE_END\x00/g, '</span>');

  return result;
}

export function formatMergedMatchText(
  matches: MatchHunk[],
  searchTerm: string,
  replaceTerm: string
): string {
  // Use the first match for the original line text (should be the full line)
  const originalText = matches[0].line_before || matches[0].content || '';

  let formatted = '<div class="match-content">';

  if (replaceTerm && matches.some((m) => m.replace)) {
    // Diff mode
    const finalText = applyReplacements(originalText, matches);

    // For the removed line: highlight the search terms
    const originalWithPlaceholders = insertSearchPlaceholders(
      originalText,
      matches
    );
    const highlightedOriginal = placeholdersToHtml(originalWithPlaceholders);

    // For the added line: highlight the replacements
    const replacementPositions = calculateReplacementPositions(matches);
    const finalWithPlaceholders = insertReplacementPlaceholders(
      finalText,
      replacementPositions
    );
    const highlightedFinal = placeholdersToHtml(finalWithPlaceholders);

    formatted +=
      '<div class="diff-line removed">- ' + highlightedOriginal + '</div>';
    formatted +=
      '<div class="diff-line added">+ ' + highlightedFinal + '</div>';
  } else {
    // Search only mode
    const textWithPlaceholders = insertSearchPlaceholders(
      originalText,
      matches
    );
    const highlightedText = placeholdersToHtml(textWithPlaceholders);

    formatted += '<div class="search-line">' + highlightedText + '</div>';
  }

  formatted += '</div>';
  return formatted;
}
