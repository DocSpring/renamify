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
- Cross-platform support (Linux, macOS, Windows)

## Demo

Watch Refaktor rename itself to `smart_search_and_replace`, then use the renamed binary to change it back:

```bash
# Clone and build
git clone https://github.com/DocSpring/refaktor
cd refaktor
cargo build --release

# Use refaktor to rename itself to smart_search_and_replace
./target/release/refaktor rename refaktor smart_search_and_replace --preview table

# Rebuild with the new name
rm -rf target
cargo build --release

# Now use smart_search_and_replace to rename itself back to refaktor!
./target/release/smart_search_and_replace rename smart_search_and_replace refaktor --preview table

# Back to the original
rm -rf target
cargo build --release
./target/release/refaktor --help
```

This demonstrates Refaktor's power: it can rename entire projects including all code references, file names, and directory names - all while maintaining perfect consistency across different naming conventions.

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
