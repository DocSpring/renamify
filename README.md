# Renamify

Smart search & replace for code and files with case-aware transformations and built-in undo/redo.

- CLI tool
- Extension for VS Code and Cursor _(coming soon)_
- MCP server for AI integrations _(coming soon)_

## Help Wanted

- If you're a Windows user, we'd love your help to get everything working on Windows!
- MCP Server and VS Code + Cursor extensions are on our roadmap.
  - If you're interested in helping us build these, please open an issue and let us know! (So we don't duplicate the effort.)
- Contribute tests to help us get to >= 95% test coverage
- Contribute fixes or improvements to the documentation. We got AI to write a lot of it and AI really likes making stuff up.
- Please try out Renamify and let us know if you have any feedback. Feel free to open issues:
  - Found a bug?
  - Not enough options?
  - Too many options?
  - Missing a feature?

## Features

- Case-aware transformations (snake_case, kebab-case, camelCase, PascalCase, UPPER_SNAKE_CASE)
- File and directory renaming
- Plan / apply workflow for safety during large renamings
- Built-in undo/redo with history tracking (separate to git history)
  - Perform a large rename safely without needing to commit anything first
- Respects ignore files (`.gitignore`, `.ignore`, `.rgignore`, `.rnignore`)
- Cross-platform support (Linux, macOS, Windows)

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

### Debugging

For debugging tokenization issues, you can set the `DEBUG_TOKENIZE` environment variable to see detailed output of how Renamify parses identifiers into tokens:

```bash
DEBUG_TOKENIZE=1 cargo test test_name
```

This is particularly useful when troubleshooting issues with case conversions or acronym handling.

### Code Coverage

The `renamify-core` library maintains at least 85% code coverage:

```bash
cargo install cargo-llvm-cov --version 0.6.15
cargo llvm-cov --package renamify-core
```

### Contributing

- Open an issue if you find a bug or have a feature request
- If you open a PR, ensure your changes have solid test coverage
- Run `cargo fmt` and `cargo clippy` to ensure your code is formatted correctly and passes linting

## License

MIT License - Copyright (c) 2025 DocSpring

## Build Status

[![CI](https://github.com/DocSpring/renamify/actions/workflows/ci.yml/badge.svg)](https://github.com/DocSpring/renamify/actions/workflows/ci.yml)

[![E2E Tests](https://github.com/DocSpring/renamify/actions/workflows/e2e.yml/badge.svg)](https://github.com/DocSpring/renamify/actions/workflows/e2e.yml)

---

## Created By

<a href="https://docspring.com">
  <img src="https://docspring.com/assets/logo-text-1e09b5522ee8602e08f1e3c4851e1657b14bd49e2e633618c344b4dc23fcbf79.svg" alt="DocSpring Logo" width="200">
</a>

Renamify was created by [DocSpring](https://docspring.com)

Fill, sign, and generate PDFs at scale
