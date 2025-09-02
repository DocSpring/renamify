import { describe, expect, test } from 'vitest';
import {
  applyReplacements,
  calculateReplacementPositions,
  escapeHtml,
  escapeRegExp,
  formatMergedMatchText,
  highlightReplaceTerm,
  highlightSearchTerm,
  insertReplacementPlaceholders,
  insertSearchPlaceholders,
  placeholdersToHtml,
} from './formatter';

const ADDED_LINE_REGEX = /<div class="diff-line added">\+ (.*?)<\/div>/;

describe('Formatter Test Suite', () => {
  describe('applyReplacements', () => {
    test('should apply single replacement', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 6, // 0-indexed
          variant: 'oldName',
          content: 'oldName',
          replace: 'newName',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
        },
      ];

      const result = applyReplacements('const oldName = 5;', matches);
      expect(result).toBe('const newName = 5;');
    });

    test('should apply multiple replacements', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 8, // 0-indexed
          variant: 'caseStyles',
          content: 'caseStyles',
          replace: 'braceStyles',
          start: 8,
          end: 18,
          line_before: '        caseStyles: data.caseStyles,',
        },
        {
          file: 'test.ts',
          line: 1,
          char_offset: 25, // 0-indexed
          variant: 'caseStyles',
          content: 'caseStyles',
          replace: 'braceStyles',
          start: 25,
          end: 35,
          line_before: '        caseStyles: data.caseStyles,',
        },
      ];

      const result = applyReplacements(
        '        caseStyles: data.caseStyles,',
        matches
      );
      expect(result).toBe('        braceStyles: data.braceStyles,');
    });
  });

  describe('insertSearchPlaceholders', () => {
    test('should insert placeholders for single match', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 6, // 0-indexed
          variant: 'oldName',
          content: 'oldName',
          replace: 'newName',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
        },
      ];

      const result = insertSearchPlaceholders('const oldName = 5;', matches);
      expect(result).toBe(
        'const \x00SEARCH_START\x00oldName\x00SEARCH_END\x00 = 5;'
      );
    });
  });

  describe('calculateReplacementPositions', () => {
    test('should calculate positions for single replacement', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 6, // 0-indexed
          variant: 'oldName',
          content: 'oldName',
          replace: 'newName',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
        },
      ];

      const positions = calculateReplacementPositions(matches);
      expect(positions).toEqual([
        { pos: 6, length: 7 }, // newName is 7 chars
      ]);
    });

    test('should calculate positions for multiple replacements with shifts', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 8, // 0-indexed
          variant: 'caseStyles',
          content: 'caseStyles', // 10 chars
          replace: 'braceStyles', // 11 chars
          start: 8,
          end: 18,
          line_before: '        caseStyles: data.caseStyles,',
        },
        {
          file: 'test.ts',
          line: 1,
          char_offset: 25, // 0-indexed
          variant: 'caseStyles',
          content: 'caseStyles', // 10 chars
          replace: 'braceStyles', // 11 chars
          start: 25,
          end: 35,
          line_before: '        caseStyles: data.caseStyles,',
        },
      ];

      const positions = calculateReplacementPositions(matches);
      expect(positions).toEqual([
        { pos: 8, length: 11 }, // First replacement at original position
        { pos: 26, length: 11 }, // Second replacement shifted by +1 (11-10)
      ]);
    });
  });

  describe('insertReplacementPlaceholders', () => {
    test('should insert placeholders at correct positions', () => {
      const text = '        braceStyles: data.braceStyles,';
      const positions = [
        { pos: 8, length: 11 },
        { pos: 26, length: 11 },
      ];

      const result = insertReplacementPlaceholders(text, positions);
      expect(result).toBe(
        '        \x00REPLACE_START\x00braceStyles\x00REPLACE_END\x00: data.\x00REPLACE_START\x00braceStyles\x00REPLACE_END\x00,'
      );
    });
  });

  describe('placeholdersToHtml', () => {
    test('should convert placeholders and escape HTML', () => {
      const text = '<div>\x00SEARCH_START\x00test\x00SEARCH_END\x00</div>';
      const result = placeholdersToHtml(text);
      expect(result).toBe(
        '&lt;div&gt;<span class="search-highlight">test</span>&lt;/div&gt;'
      );
    });
  });
  describe('formatMergedMatchText with HTML content', () => {
    test('should highlight caseStyles in HTML label correctly', () => {
      // When searching for 'case', renamify finds the full token 'caseStyles'
      const matches: MatchHunk[] = [
        {
          file: 'test.html',
          line: 1,
          char_offset: 12, // Position where "caseStyles" starts (0-indexed)
          variant: 'camelCase',
          content: 'caseStyles', // Full matched token
          replace: 'braceStyles', // Full replacement token
          start: 12,
          end: 22,
          line_before:
            '<label for="caseStyles">Case styles (<span id="checkedCount">',
        },
      ];

      const result = formatMergedMatchText(matches, 'case', 'brace');

      // Test the EXACT full output
      const expected =
        '<div class="match-content">' +
        '<div class="diff-line removed">- &lt;label for=&quot;<span class="search-highlight">caseStyles</span>&quot;&gt;Case styles (&lt;span id=&quot;checkedCount&quot;&gt;</div>' +
        '<div class="diff-line added">+ &lt;label for=&quot;<span class="replace-highlight">braceStyles</span>&quot;&gt;Case styles (&lt;span id=&quot;checkedCount&quot;&gt;</div>' +
        '</div>';

      expect(result).toBe(expected);
    });

    test('should handle replacement that contains search term', () => {
      // This is the exact problematic case from the VS Code inspector:
      // Two occurrences of "caseStyles" on the same line
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 226,
          char_offset: 8, // First "caseStyles" (0-indexed)
          variant: 'camelCase',
          content: 'caseStyles',
          replace: 'braceStyles',
          start: 8,
          end: 18,
          line_before: '        caseStyles: data.caseStyles,',
        },
        {
          file: 'test.ts',
          line: 226,
          char_offset: 25, // Second "caseStyles" after "data." (0-indexed)
          variant: 'camelCase',
          content: 'caseStyles',
          replace: 'braceStyles',
          start: 25,
          end: 35,
          line_before: '        caseStyles: data.caseStyles,',
        },
      ];

      const result = formatMergedMatchText(matches, 'case', 'brace');

      // Test the EXACT full output - both replacements should be correct
      const expected =
        '<div class="match-content">' +
        '<div class="diff-line removed">-         <span class="search-highlight">caseStyles</span>: data.<span class="search-highlight">caseStyles</span>,</div>' +
        '<div class="diff-line added">+         <span class="replace-highlight">braceStyles</span>: data.<span class="replace-highlight">braceStyles</span>,</div>' +
        '</div>';

      expect(result).toBe(expected);
    });

    test('should handle multiple matches in HTML content', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.html',
          line: 1,
          char_offset: 12, // Position of first "caseStyles" in class attribute (0-indexed)
          variant: 'camelCase',
          content: 'caseStyles',
          replace: 'braceStyles',
          start: 13,
          end: 23,
          line_before: '<div class="caseStyles" id="caseStylesContainer">',
        },
        {
          file: 'test.html',
          line: 1,
          char_offset: 28, // Position of "caseStylesContainer" in id attribute (0-indexed)
          variant: 'camelCase',
          content: 'caseStylesContainer',
          replace: 'braceStylesContainer',
          start: 29,
          end: 48,
          line_before: '<div class="caseStyles" id="caseStylesContainer">',
        },
      ];

      const result = formatMergedMatchText(matches, 'case', 'brace');

      // Test the EXACT full output
      const expected =
        '<div class="match-content">' +
        '<div class="diff-line removed">- &lt;div class=&quot;<span class="search-highlight">caseStyles</span>&quot; id=&quot;<span class="search-highlight">caseStylesContainer</span>&quot;&gt;</div>' +
        '<div class="diff-line added">+ &lt;div class=&quot;<span class="replace-highlight">braceStyles</span>&quot; id=&quot;<span class="replace-highlight">braceStylesContainer</span>&quot;&gt;</div>' +
        '</div>';

      expect(result).toBe(expected);
    });
  });

  describe('escapeHtml', () => {
    test('should escape HTML special characters', () => {
      expect(escapeHtml('Hello <world>')).toBe('Hello &lt;world&gt;');
      expect(escapeHtml('foo & bar')).toBe('foo &amp; bar');
      expect(escapeHtml('<script>alert("xss")</script>')).toBe(
        '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
      );
      expect(escapeHtml('She said "hello"')).toBe('She said &quot;hello&quot;');
      expect(escapeHtml("It's working")).toBe('It&#39;s working');
    });

    test('should handle empty strings', () => {
      expect(escapeHtml('')).toBe('');
    });

    test('should handle strings without special characters', () => {
      expect(escapeHtml('Hello World')).toBe('Hello World');
    });

    test('should escape multiple occurrences', () => {
      expect(escapeHtml('<<>>')).toBe('&lt;&lt;&gt;&gt;');
      expect(escapeHtml('&&&')).toBe('&amp;&amp;&amp;');
    });
  });

  describe('escapeRegExp', () => {
    test('should escape regex special characters', () => {
      // biome-ignore lint/suspicious/noTemplateCurlyInString: Testing literal string with special chars
      const testStr = '.*+?^${}()|[]\\'; // Testing regex special chars
      expect(
        escapeRegExp(testStr),
        '\\.\\*\\+\\?\\^\\$\\{\\}\\(\\)\\|\\[\\]\\\\'
      );
    });

    test('should handle strings without special characters', () => {
      expect(escapeRegExp('hello world')).toBe('hello world');
    });

    test('should escape common regex patterns', () => {
      expect(escapeRegExp('file.txt')).toBe('file\\.txt');
      expect(escapeRegExp('(foo|bar)')).toBe('\\(foo\\|bar\\)');
      expect(escapeRegExp('[a-z]+')).toBe('\\[a-z\\]\\+');
    });

    test('should handle empty strings', () => {
      expect(escapeRegExp('')).toBe('');
    });
  });

  describe('highlightSearchTerm', () => {
    test('should highlight search terms with span tags', () => {
      const result = highlightSearchTerm('hello world', 'world');
      expect(result).toBe('hello <span class="search-highlight">world</span>');
    });

    test('should highlight case-insensitively', () => {
      const result = highlightSearchTerm('Hello World', 'world');
      expect(result).toBe('Hello <span class="search-highlight">World</span>');
    });

    test('should highlight multiple occurrences', () => {
      const result = highlightSearchTerm('foo bar foo baz', 'foo');
      expect(
        result,
        '<span class="search-highlight">foo</span> bar <span class="search-highlight">foo</span> baz'
      );
    });

    test('should handle empty search term', () => {
      const result = highlightSearchTerm('hello world', '');
      expect(result).toBe('hello world');
    });

    test('should escape regex special characters in search term', () => {
      const result = highlightSearchTerm('file.txt', 'file.txt');
      expect(result).toBe('<span class="search-highlight">file.txt</span>');
    });

    test('should not match partial regex patterns', () => {
      const result = highlightSearchTerm('fileXtxt', 'file.txt');
      expect(result).toBe('fileXtxt'); // Should not match because . is escaped
    });
  });

  describe('highlightReplaceTerm', () => {
    test('should highlight replacement terms', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'oldName',
          content: 'oldName',
          replace: 'newName',
          start: 0,
          end: 7,
          line_before: 'const oldName = 5;',
          line_after: 'const newName = 5;',
        },
      ];
      const result = highlightReplaceTerm('const newName = 5;', matches);
      expect(
        result,
        'const <span class="replace-highlight">newName</span> = 5;'
      );
    });

    test('should highlight multiple different replacement terms', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'foo',
          content: 'foo',
          replace: 'bar',
          start: 0,
          end: 3,
          line_before: 'foo and hello',
          line_after: 'bar and baz',
        },
        {
          file: 'test.ts',
          line: 1,
          char_offset: 8,
          variant: 'hello',
          content: 'hello',
          replace: 'baz',
          start: 8,
          end: 13,
          line_before: 'foo and hello',
          line_after: 'bar and baz',
        },
      ];
      const result = highlightReplaceTerm('bar and baz code', matches);
      expect(
        result,
        '<span class="replace-highlight">bar</span> and <span class="replace-highlight">baz</span> code'
      );
    });

    test('should handle empty matches array', () => {
      const result = highlightReplaceTerm('hello world', []);
      expect(result).toBe('hello world');
    });

    test('should handle matches without replace field', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'foo',
          content: 'foo',
          replace: '',
          start: 0,
          end: 3,
          line_before: 'foo bar',
          line_after: 'foo bar',
        },
      ];
      const result = highlightReplaceTerm('foo bar', matches);
      expect(result).toBe('foo bar');
    });

    test('should handle null/undefined matches', () => {
      const result = highlightReplaceTerm('hello world', null as any);
      expect(result).toBe('hello world');
    });

    test('should highlight case-insensitively', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'foo',
          content: 'foo',
          replace: 'bar',
          start: 0,
          end: 3,
          line_before: 'foo',
          line_after: 'BAR',
        },
      ];
      const result = highlightReplaceTerm('Bar code', matches);
      expect(result).toBe('<span class="replace-highlight">Bar</span> code');
    });

    test('should handle multiple replacements with real Cargo.toml data', () => {
      const matches: MatchHunk[] = [
        {
          file: '/Users/ndbroadbent/code/renamify/Cargo.toml',
          line: 2,
          char_offset: 12,
          variant: 'renamify-core',
          content: 'renamify-core',
          replace: 'rename-thingy-core',
          start: 24,
          end: 37,
          line_before: 'members = ["renamify-core", "renamify-cli"]',
          line_after: 'members = ["rename-thingy-core", "renamify-cli"]',
        },
        {
          file: '/Users/ndbroadbent/code/renamify/Cargo.toml',
          line: 2,
          char_offset: 29,
          variant: 'renamify-cli',
          content: 'renamify-cli',
          replace: 'rename-thingy-cli',
          start: 41,
          end: 53,
          line_before: 'members = ["renamify-core", "renamify-cli"]',
          line_after: 'members = ["renamify-core", "rename-thingy-cli"]',
        },
      ];

      // Test with the CORRECT final text (what should be the result)
      const finalText = 'members = ["rename-thingy-core", "rename-thingy-cli"]';
      const result = highlightReplaceTerm(finalText, matches);

      // Should highlight both replacement terms in the final text
      const expected =
        'members = ["<span class="replace-highlight">rename-thingy-core</span>", "<span class="replace-highlight">rename-thingy-cli</span>"]';
      expect(result).toBe(expected);
    });

    test('should handle overlapping replacement terms', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'test',
          content: 'test',
          replace: 'testing',
          start: 0,
          end: 4,
          line_before: 'test code',
          line_after: 'testing code',
        },
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'code',
          content: 'code',
          replace: 'testing',
          start: 5,
          end: 9,
          line_before: 'test code',
          line_after: 'testing testing',
        },
      ];

      // Both replacements are 'testing' - should only highlight once per occurrence
      const result = highlightReplaceTerm('testing testing more', matches);
      const expected =
        '<span class="replace-highlight">testing</span> <span class="replace-highlight">testing</span> more';
      expect(result).toBe(expected);
    });

    test('should handle empty replacement terms', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'old',
          content: 'old',
          replace: '',
          start: 0,
          end: 3,
          line_before: 'old text',
          line_after: ' text',
        },
      ];

      const result = highlightReplaceTerm('some text here', matches);
      // Should not highlight anything since replace is empty
      expect(result).toBe('some text here');
    });
  });

  describe('formatMergedMatchText', () => {
    test('should format search-only mode', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 6, // 0-indexed position where "oldName" starts
          variant: 'oldName',
          content: 'oldName',
          replace: '',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
        },
      ];
      const result = formatMergedMatchText(matches, 'oldName', '');

      const expected =
        '<div class="match-content">' +
        '<div class="search-line">const <span class="search-highlight">oldName</span> = 5;</div>' +
        '</div>';

      expect(result).toBe(expected);
    });

    test('should format diff mode with replacements', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 6, // 0-indexed position where "oldName" starts
          variant: 'oldName',
          content: 'oldName',
          replace: 'newName',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
          line_after: 'const newName = 5;',
        },
      ];
      const result = formatMergedMatchText(matches, 'oldName', 'newName');

      const expected =
        '<div class="match-content">' +
        '<div class="diff-line removed">- const <span class="search-highlight">oldName</span> = 5;</div>' +
        '<div class="diff-line added">+ const <span class="replace-highlight">newName</span> = 5;</div>' +
        '</div>';

      expect(result).toBe(expected);
    });

    test('should handle multiple replacements on same line', () => {
      // ACTUAL data from renamify for Cargo.toml line 2
      const matches: MatchHunk[] = [
        {
          file: '/Users/ndbroadbent/code/renamify/Cargo.toml',
          line: 2,
          char_offset: 12, // 0-indexed position where "renamify-core" starts
          variant: 'renamify-core',
          content: 'renamify-core',
          replace: 'rename-thingy-core',
          start: 24, // File-relative position!
          end: 37,
          line_before: 'members = ["renamify-core", "renamify-cli"]',
          line_after: 'members = ["rename-thingy-core", "renamify-cli"]',
        },
        {
          file: '/Users/ndbroadbent/code/renamify/Cargo.toml',
          line: 2,
          char_offset: 29, // 0-indexed position where "renamify-cli" starts
          variant: 'renamify-cli',
          content: 'renamify-cli',
          replace: 'rename-thingy-cli',
          start: 41, // File-relative position!
          end: 53,
          line_before: 'members = ["renamify-core", "renamify-cli"]',
          line_after: 'members = ["renamify-core", "rename-thingy-cli"]',
        },
      ];
      const result = formatMergedMatchText(
        matches,
        'renamify',
        'rename_thingy'
      );

      // Check the HTML structure - should have removed and added lines
      expect(result.includes('diff-line removed'));
      expect(result.includes('diff-line added'));

      // Extract the added line content (the part after the + sign)
      const addedLineMatch = result.match(ADDED_LINE_REGEX);
      const addedLineContent = addedLineMatch
        ? addedLineMatch[1].replace(/<[^>]*>/g, '')
        : '';

      // Test both the plain text content AND the complete HTML with highlighting
      expect(addedLineContent).toBe(
        'members = [&quot;rename-thingy-core&quot;, &quot;rename-thingy-cli&quot;]'
      );

      // Test the complete HTML structure with highlighting
      const expectedHtml =
        '<div class="match-content">' +
        '<div class="diff-line removed">- members = [&quot;<span class="search-highlight">renamify-core</span>&quot;, &quot;<span class="search-highlight">renamify-cli</span>&quot;]</div>' +
        '<div class="diff-line added">+ members = [&quot;<span class="replace-highlight">rename-thingy-core</span>&quot;, &quot;<span class="replace-highlight">rename-thingy-cli</span>&quot;]</div>' +
        '</div>';

      expect(result).toBe(expectedHtml);
    });

    test('should escape HTML in text content', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 7, // 1-indexed position where "<script>" starts
          variant: '<script>',
          content: '<script>',
          replace: '<div>',
          start: 6,
          end: 14,
          line_before: 'const <script> = "xss";',
          line_after: 'const <div> = "xss";',
        },
      ];
      const result = formatMergedMatchText(matches, '<script>', '<div>');
      expect(result.includes('&lt;script&gt;'));
      expect(result.includes('&lt;div&gt;'));
      expect(result.includes('&quot;xss&quot;')); // Check quotes are escaped too
      expect(!result.includes('<script>'));
      expect(!result.includes('<div>'));
    });

    test('should use context when text is not available', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'oldValue',
          content: 'oldValue',
          replace: 'newValue',
          start: 0,
          end: 8,
          line_after: 'newValue',
        },
      ];
      const result = formatMergedMatchText(matches, 'oldValue', 'newValue');
      expect(result.includes('oldValue'));
    });

    test('should handle replaceTerm without actual replacements in matches', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 7, // 1-indexed position where "oldName" starts
          variant: 'oldName',
          content: 'oldName',
          replace: '',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
        },
      ];
      const result = formatMergedMatchText(matches, 'oldName', 'newName');
      // Should show search-only mode since no actual replacements
      expect(result.includes('search-line'));
      expect(!result.includes('diff-line'));
    });

    test('should wrap content in match-content div', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          char_offset: 1, // 1-indexed
          variant: 'test',
          content: 'test',
          replace: '',
          start: 0,
          end: 4,
          line_before: 'test',
        },
      ];
      const result = formatMergedMatchText(matches, 'test', '');
      expect(result.startsWith('<div class="match-content">'));
      expect(result.endsWith('</div>'));
    });
  });
});
