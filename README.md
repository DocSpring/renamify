<picture><source media="(prefers-color-scheme: dark)" srcset="docs/src/assets/logo-white-wordmark.svg"><source media="(prefers-color-scheme: light)" srcset="docs/src/assets/logo-wordmark.svg"><img alt="Renamify logo" src="docs/src/assets/logo-white-wordmark.svg" height="32" align="absmiddle"></picture>

Smart search & replace for code and filenames with case-aware transformations and built-in undo/redo.

[![CI](https://github.com/DocSpring/renamify/actions/workflows/ci.yml/badge.svg)](https://github.com/DocSpring/renamify/actions/workflows/ci.yml)
[![E2E](https://github.com/DocSpring/renamify/actions/workflows/e2e.yml/badge.svg)](https://github.com/DocSpring/renamify/actions/workflows/e2e.yml)
[![MCP](https://github.com/DocSpring/renamify/actions/workflows/mcp.yml/badge.svg)](https://github.com/DocSpring/renamify/actions/workflows/mcp.yml)
[![VS Code](https://github.com/DocSpring/renamify/actions/workflows/vscode.yml/badge.svg)](https://github.com/DocSpring/renamify/actions/workflows/vscode.yml)
[![Docs](https://github.com/DocSpring/renamify/actions/workflows/docs.yml/badge.svg)](https://docspring.github.io/renamify/)
[![codecov](https://codecov.io/gh/DocSpring/renamify/graph/badge.svg)](https://codecov.io/gh/DocSpring/renamify)

- **CLI tool** - Cross-platform with full Windows, macOS, and Linux support<br>
- **MCP server** - AI integrations for Claude, Cursor, and other MCP-compatible tools<br>
- **VS Code + Cursor extension** - Instant renaming without leaving your editor<br>
- **Written in Rust** - High performance and memory safety with >90% code coverage<br>
- **Documentation** - Learn more at [docspring.github.io/renamify](https://docspring.github.io/renamify/)

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://docspring.github.io/renamify/install.sh | sh
```

### From Source

```bash
# Using Cargo
cargo install --git https://github.com/DocSpring/renamify

# Or clone and build
git clone https://github.com/DocSpring/renamify
cd renamify
cargo build --release
# Binary will be at ./target/release/renamify
```

### MCP Server

```bash
npx @renamify/mcp-server
```

See [MCP Server documentation](https://docspring.github.io/renamify/mcp/overview/) for Claude and Cursor setup.

## Features

- **Smart Case Conversion**: Automatically detects and converts between different case styles (snake_case, kebab-case, camelCase, PascalCase, SCREAMING_SNAKE_CASE, Train-Case, Title Case, dot.case, lowercase, UPPERCASE)
- **Safe by default**: Plan → Review → Apply workflow prevents accidents
- **Built-in undo/redo**: Full history tracking separate from git. No need to git stash or commit with `--no-verify` before renaming.
- **File and directory renaming**: Rename everything in one atomic operation
- **Respects ignore files**: Works with `.gitignore`, `.ignore`, `.rgignore`, `.rnignore`
- **Cross-platform**: Full support for Linux, macOS, and Windows
- **Line filtering**: Exclude matches on lines matching a regex pattern (e.g., skip comments, TODOs)

## Quick Examples

```bash
# Preview changes before applying
renamify plan old_name new_name

# Rename with automatic approval
renamify rename old_name new_name --yes

# Undo the last rename
renamify undo

# See what would change with different preview formats
renamify plan myProject betterName --preview table
renamify plan myProject betterName --preview diff

# Exclude matches in comments and TODO lines
renamify plan old_name new_name --exclude-matching-lines '^//'
renamify plan old_name new_name --exclude-matching-lines '(TODO|FIXME)'

# Skip renaming files and directories
renamify plan old_name new_name --no-rename-paths

# Ignore ambiguous/mixed-case identifiers
renamify plan old_name new_name --ignore-ambiguous
```

## Demo

Watch Renamify rename itself to `renamed_renaming_tool`, then use the renamed binary to change it back:

```bash
# Clone and build
git clone https://github.com/DocSpring/renamify
cd renamify
cargo build --release

# Use renamify to rename itself to renamed_renaming_tool
./target/release/renamify rename renamify renamed_renaming_tool --preview table

# Rebuild with the new name
rm -rf target
cargo build --release

# Now use renamed_renaming_tool to rename itself back to renamify!
./target/release/renamed_renaming_tool rename renamed_renaming_tool renamify --preview table

# Back to the original
rm -rf target
cargo build --release
./target/release/renamify --help
```

This demonstrates Renamify's power: it can rename entire projects including all code references, file names, and directory names - all while maintaining perfect consistency across different naming conventions.

## Ignore Files

Renamify respects various ignore files to skip files and directories during scanning:

- `.gitignore` - Standard Git ignore patterns
- `.ignore` - Generic ignore file (like ripgrep)
- `.rgignore` - Ripgrep-specific ignore patterns
- `.rnignore` - Renamify-specific ignore patterns

You can control how ignore files are handled using the `-u` flag:

- Default: Respects all ignore files and skips hidden files
- `-u`: Ignores `.gitignore` but respects other ignore files
- `-uu`: Ignores all ignore files and shows hidden files
- `-uuu`: Same as `-uu` plus treats binary files as text

The `.rnignore` file is useful when you want to exclude files specifically from renamify operations without affecting Git or other tools.

## Versioning Policy

Renamify follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backward-compatible functionality additions
- **PATCH** version for backward-compatible bug fixes

### Component Compatibility

The MCP server and VS Code extension depend on the CLI and must maintain version compatibility:

- **Major version**: Must match the CLI major version exactly
- **Minor version**: Must be ≤ CLI minor version (can use older features, not newer ones)
- **Patch version**: Independent, can be any value

#### Examples

- CLI `2.3.1` → MCP `2.0.5` ✅ (major matches, minor 0 ≤ 3)
- CLI `2.3.1` → MCP `2.3.9` ✅ (major matches, minor 3 ≤ 3)
- CLI `2.3.1` → MCP `2.4.0` ❌ (minor 4 > 3, might use unavailable features)
- CLI `2.3.1` → MCP `3.0.0` ❌ (major version mismatch)

### Version Check

The CLI provides version information via:

```bash
renamify version --output json
# {"name":"renamify","version":"0.1.0"}
```

Both the MCP server and VS Code extension verify CLI compatibility on startup and will fail gracefully if the CLI version doesn't meet requirements.

## Development

### Quick Setup

```bash
# Install all development dependencies (Rust, tools, etc.)
./scripts/dev-setup.sh
```

### Prerequisites

- Rust 1.80.1 or later (installed by dev-setup.sh)
- cargo (included with Rust)

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Debugging

For debugging tokenization issues, you can set the `DEBUG_TOKENIZE` environment variable to see detailed output of how Renamify parses identifiers into tokens:

```bash
DEBUG_TOKENIZE=1 cargo test test_name
```

This is particularly useful when troubleshooting issues with case conversions or acronym handling.

### Code Coverage

The `renamify-core` library maintains comprehensive test coverage:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --package renamify-core --html
# Open target/llvm-cov/html/index.html to view the report
```

### Linting & Formatting

We use automated checks to maintain code quality:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features

# Check shell scripts
./scripts/shellcheck.sh

# Or let git hooks handle it automatically
git commit  # lefthook will run checks pre-commit
```

### Contributing

- **Documentation**: Help improve our docs, add more examples, fix any inaccuracies
- **Bug Reports**: Found an issue? Please [report it](https://github.com/DocSpring/renamify/issues)!
- PRs welcome! All checks must pass (formatting, linting, tests)
- We use [lefthook](https://github.com/evilmartians/lefthook) for git hooks

## License

MIT License - Copyright (c) 2025 DocSpring

---

## Created By

<a href="https://docspring.com">
  <img src="https://docspring.com/assets/logo-text-1e09b5522ee8602e08f1e3c4851e1657b14bd49e2e633618c344b4dc23fcbf79.svg" alt="DocSpring Logo" width="200">
</a>

Renamify was created by [DocSpring](https://docspring.com)

Fill, sign, and generate PDFs at scale
