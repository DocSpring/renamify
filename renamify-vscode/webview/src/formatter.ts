export function escapeHtml(text: string): string {
  if (!text) {
    return '';
  }
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
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

export function formatMergedMatchText(
  matches: MatchHunk[],
  searchTerm: string,
  replaceTerm: string
): string {
  // Use the first match for the original line text (should be the full line)
  const originalText = matches[0].line_before || matches[0].content || '';

  let formatted = '<div class="match-content">';

  if (replaceTerm && matches.some((m) => m.replace)) {
    // Merge all replacements by applying them in order
    let finalText = originalText;

    // Sort matches by column position to apply replacements in the correct order
    const sortedMatches = [...matches].sort((a, b) => a.col - b.col);

    // Apply replacements from right to left to avoid position shifts
    for (let i = sortedMatches.length - 1; i >= 0; i--) {
      const match = sortedMatches[i];
      if (match.replace && match.content) {
        // Use line-relative positions from col field
        const lineStart = match.col; // col is already 0-indexed for our purposes
        const lineEnd = lineStart + match.content.length;

        const before = finalText.substring(0, lineStart);
        const after = finalText.substring(lineEnd);
        finalText = before + match.replace + after;
      }
    }

    // Show diff format: - original, + final result
    formatted +=
      '<div class="diff-line removed">- ' +
      highlightSearchTerm(escapeHtml(originalText), searchTerm) +
      '</div>';
    formatted +=
      '<div class="diff-line added">+ ' +
      highlightReplaceTerm(escapeHtml(finalText), matches) +
      '</div>';
  } else {
    // Search only mode - just highlight all search terms
    formatted +=
      '<div class="search-line">' +
      highlightSearchTerm(escapeHtml(originalText), searchTerm) +
      '</div>';
  }

  formatted += '</div>';
  return formatted;
}
