# Refaktor

Smart search & replace for code and files with case-aware transformations and built-in undo/redo.

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

The `refaktor-core` library maintains 100% code coverage:

```bash
cargo install cargo-llvm-cov --version 0.6.15
cargo llvm-cov --package refaktor-core
```

## License

MIT OR Apache-2.0 (dual licensed)