# Refaktor

Smart search & replace for code and files with case-aware transformations and built-in undo/redo.

- CLI tool
- Extension for VS Code and Cursor _(coming soon)_
- MCP server for AI integrations _(coming soon)_

## Features

- Case-aware transformations (snake_case, kebab-case, camelCase, PascalCase, UPPER_SNAKE_CASE)
- File and directory renaming
- Plan / apply workflow for safety during large refactorings
- Built-in undo/redo with history tracking (separate to git history)
  - Perform a large refactor safely without needing to commit anything first
- Respects ignore files (`.gitignore`, `.ignore`, `.rgignore`, `.rfignore`)
- Cross-platform support (Linux, macOS, Windows)

## Demo

Watch Refaktor rename itself to `renamed_refactoring_tool`, then use the renamed binary to change it back:

```bash
# Clone and build
git clone https://github.com/DocSpring/refaktor
cd refaktor
cargo build --release

# Use refaktor to rename itself to renamed_refactoring_tool
./target/release/refaktor rename refaktor renamed_refactoring_tool --preview table

# Rebuild with the new name
rm -rf target
cargo build --release

# Now use renamed_refactoring_tool to rename itself back to refaktor!
./target/release/renamed_refactoring_tool rename renamed_refactoring_tool refaktor --preview table

# Back to the original
rm -rf target
cargo build --release
./target/release/refaktor --help
```

This demonstrates Refaktor's power: it can rename entire projects including all code references, file names, and directory names - all while maintaining perfect consistency across different naming conventions.

## Ignore Files

Refaktor respects various ignore files to skip files and directories during scanning:

- `.gitignore` - Standard Git ignore patterns
- `.ignore` - Generic ignore file (like ripgrep)
- `.rgignore` - Ripgrep-specific ignore patterns
- `.rfignore` - Refaktor-specific ignore patterns

You can control how ignore files are handled using the `-u` flag:

- Default: Respects all ignore files and skips hidden files
- `-u`: Ignores `.gitignore` but respects other ignore files
- `-uu`: Ignores all ignore files and shows hidden files
- `-uuu`: Same as `-uu` plus treats binary files as text

The `.rfignore` file is useful when you want to exclude files specifically from refaktor operations without affecting Git or other tools.

## Build Status

[![CI](https://github.com/ndbroadbent/refaktor/actions/workflows/ci.yml/badge.svg)](https://github.com/ndbroadbent/refaktor/actions/workflows/ci.yml)

## Development

### Prerequisites

- Rust 1.80.1 or later
- cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Coverage

The `refaktor-core` library maintains at least 95% code coverage:

```bash
cargo install cargo-llvm-cov --version 0.6.15
cargo llvm-cov --package refaktor-core
```

## License

MIT License - Copyright (c) 2025 DocSpring

---

## Created By

<a href="https://docspring.com">
  <img src="https://docspring.com/assets/logo-text-1e09b5522ee8602e08f1e3c4851e1657b14bd49e2e633618c344b4dc23fcbf79.svg" alt="DocSpring Logo" width="200">
</a>

Refaktor was created by [DocSpring](https://docspring.com)

Fill, sign, and generate PDFs at scale
