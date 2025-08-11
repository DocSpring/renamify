# Refaktor

Smart search & replace for code and files with case-aware transformations and built-in undo/redo.

## Demo: Refaktor Refactoring Itself! 🤯

Watch Refaktor rename itself to `smart_search_and_replace`, then use the renamed binary to change it back:

```bash
# Clone and build
git clone https://github.com/DocSpring/refaktor
cd refaktor
cargo build --release

# Use refaktor to rename itself to smart_search_and_replace
./target/release/refaktor plan refaktor smart_search_and_replace
./target/release/refaktor apply

# Rebuild with the new name
rm -rf target
cargo build --release

# Now use smart_search_and_replace to rename itself back to refaktor!
./target/release/smart_search_and_replace plan smart_search_and_replace refaktor
./target/release/smart_search_and_replace apply

# Back to the original
rm -rf target
cargo build --release
./target/release/refaktor --help
```

This demonstrates Refaktor's power: it can rename entire projects including all code references, file names, directory names, and even the binary itself - all while maintaining perfect consistency across different naming conventions!

## Features

- Case-aware transformations (snake_case, kebab-case, camelCase, PascalCase, UPPER_SNAKE_CASE)
- File and directory renaming
- Built-in undo/redo with history tracking
- Cross-platform support (Linux, macOS, Windows)
- VS Code extension
- MCP server for AI integrations

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