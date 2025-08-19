Research:

Here’s what exists today:

**VS Code extensions**

- There's a _Preserve Case_ feature for regex replacements—modifiers like `\u`, `\U`, `\l`, `\L` let you adjust case in replace patterns [YouTube+14Visual Studio Code+14Stack Overflow+14](https://code.visualstudio.com/docs/editing/codebasics?utm_source=chatgpt.com).
- The “AB” icon toggle preserves case during find/replace [GitHub+3YouTube+3Visual Studio Code+3](https://www.youtube.com/watch?v=JEESpXjbv38&utm_source=chatgpt.com).
  But nothing matches across different casing styles (snake, kebab, camel, Pascal) automatically, nor does it handle file/directory renaming.

**CLI/file tools**

- `stdrename` renames files/directories across naming conventions like camelCase, snake_case, kebab-case, PascalCase, SCREAMING_SNAKE_CASE, Title Case, Train‑Case, etc. It supports recursive renaming, respects `.gitignore`, and handles directories [Visual Studio Code+15GitHub+15GitHub+15](https://github.com/Gadiguibou/stdrename?utm_source=chatgpt.com).
- `GPRename` batch-renames files & directories with options including case changes and regex replacement—but it's not aware of multiple case variants, only transforms to uppercase, lowercase, or first-letter capitalization [Wikipedia](https://en.wikipedia.org/wiki/GPRename?utm_source=chatgpt.com).

**Language-specific utilities**

- The NPM `change-case` library converts individual strings across numerous case formats [Stack Overflow+4npm+4GitHub+4](https://www.npmjs.com/package/change-case?utm_source=chatgpt.com).
- `camel-snake-pep8` is a Python tool to refactor variable names between camelCase and snake_case following PEP‑8, interactively with diffs [PyPI](https://pypi.org/project/camel-snake-pep8/?utm_source=chatgpt.com).

**Summary**

| Feature                                       | Exists Now? | Tool(s)                 |
| --------------------------------------------- | ----------- | ----------------------- |
| Multi-case-aware search/replace               | No          | —                       |
| Rename files/directories with case conversion | Partial     | `stdrename`, `GPRename` |
| Built-in undo/redo history                    | No          | —                       |
| VS Code + MCP/generic CLI bridge              | No          | —                       |

Nothing combines **case-aware content replacement**, **cross-case variant generation**, **file/directory renaming**, **undo/redo with history**, and ships as both **VS Code extension**, **CLI**, and **MCP server**. Your idea seems unique.

> does this sound like a useful tool to build? and even if it already exists, raising developer awareness and promoting our tool could still be worthwhile?

###### ChatGPT said:

Yes – it sounds both **useful**and **marketable**.

**Why it’s valuable**

- Current tools are fragmented: content replacement, case conversion, and file/directory renaming are split across different utilities.
- There’s no “all-in-one” developer-friendly solution that handles **cross-case refactors**, **ripgrep-fast search**, **rename across code and filesystem**, and **undo/redo outside of git**.
- Developers doing big renames (especially cross-language) hit exactly this pain point.
- MCP support means it’s future-proof for AI coding agents.

**Even if it already exists**

- Many devs don’t know these features _could_ exist, so awareness alone could drive adoption.
- If an existing tool is niche, poorly branded, or lacking in UX polish, a **well-promoted, well-integrated**alternative can still dominate.
- VS Code marketplace discoverability is low without promotion—good naming, branding, and content can make yours the “go-to” option.

**Positioning opportunity**

- Brand it clearly around “safe renaming” or “smart rename” rather than just “search/replace.”
- Highlight speed (ripgrep), case-intelligence, and filesystem awareness in one package.
- If you own both the **developer experience**and **awareness campaign**, you can displace incumbents.

If you want, I can sketch **a positioning & promotion plan**so that even if there’s a similar tool, yours becomes the one developers think of first.
