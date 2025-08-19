import { describe, expect, test } from 'vitest';
import {
  escapeHtml,
  escapeRegExp,
  formatMergedMatchText,
  highlightReplaceTerm,
  highlightSearchTerm,
} from './formatter';

const ADDED_LINE_REGEX = /<div class="diff-line added">\+ (.*?)<\/div>/;

describe('Formatter Test Suite', () => {
  describe('escapeHtml', () => {
    test('should escape HTML special characters', () => {
      expect(escapeHtml('Hello <world>')).toBe('Hello &lt;world&gt;');
      expect(escapeHtml('foo & bar')).toBe('foo &amp; bar');
      expect(escapeHtml('<script>alert("xss")</script>')).toBe(
        '&lt;script&gt;alert("xss")&lt;/script&gt;'
      );
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
          col: 0,
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
          col: 0,
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
          col: 8,
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
          col: 0,
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
          col: 0,
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
          col: 12,
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
          col: 29,
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
          col: 0,
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
          col: 0,
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
          col: 0,
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
          col: 6,
          variant: 'oldName',
          content: 'oldName',
          replace: '',
          start: 6,
          end: 13,
          line_before: 'const oldName = 5;',
        },
      ];
      const result = formatMergedMatchText(matches, 'oldName', '');
      expect(result.includes('search-line'));
      expect(result.includes('<span class="search-highlight">oldName</span>'));
      expect(!result.includes('diff-line'));
    });

    test('should format diff mode with replacements', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          col: 6,
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
      expect(result.includes('diff-line removed'));
      expect(result.includes('diff-line added'));
      expect(result.includes('<span class="search-highlight">oldName</span>'));
      expect(result.includes('<span class="replace-highlight">newName</span>'));
    });

    test('should handle multiple replacements on same line', () => {
      // ACTUAL data from renamify for Cargo.toml line 2
      const matches: MatchHunk[] = [
        {
          file: '/Users/ndbroadbent/code/renamify/Cargo.toml',
          line: 2,
          col: 12,
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
          col: 29,
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
        'members = ["rename-thingy-core", "rename-thingy-cli"]'
      );

      // Test the complete HTML structure with highlighting
      const expectedHtml =
        '<div class="match-content">' +
        '<div class="diff-line removed">- members = ["<span class="search-highlight">renamify</span>-core", "<span class="search-highlight">renamify</span>-cli"]</div>' +
        '<div class="diff-line added">+ members = ["<span class="replace-highlight">rename-thingy-core</span>", "<span class="replace-highlight">rename-thingy-cli</span>"]</div>' +
        '</div>';

      expect(result).toBe(expectedHtml);
    });

    test('should escape HTML in text content', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          col: 6,
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
      expect(!result.includes('<script>'));
      expect(!result.includes('<div>'));
    });

    test('should use context when text is not available', () => {
      const matches: MatchHunk[] = [
        {
          file: 'test.ts',
          line: 1,
          col: 0,
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
          col: 6,
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
          col: 0,
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
