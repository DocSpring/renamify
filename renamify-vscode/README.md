# Renamify VS Code Extension

Smart case-aware search and replace with file renaming capabilities for VS Code.

## Features

- **Smart Case Conversion**: Automatically detects and converts between different case styles (snake_case, kebab-case, camelCase, PascalCase, SCREAMING_SNAKE_CASE, Train-Case, Title Case, dot.case)
- **Atomic Operations**: All changes are applied atomically with full undo/redo support
- **File & Directory Renaming**: Rename files and directories along with content updates
- **Advanced Filtering**:
  - Include/exclude files with glob patterns
  - Filter out matches on lines matching a regex pattern (e.g., comments)
  - Respect .gitignore and other ignore files
- **Rich Preview**: See all changes before applying with syntax highlighting
- **History**: Track and revert previous operations

## Requirements

- VS Code 1.85.0 or higher
- Renamify CLI installed (or available in project's `target/debug/` for development)

## Installation

### From VSIX

1. Download the `.vsix` file
2. Open VS Code
3. Go to Extensions view (Ctrl+Shift+X)
4. Click the "..." menu and select "Install from VSIX..."
5. Select the downloaded file

### Development

```bash
# Install dependencies
cd renamify-vscode
pnpm install

# Compile TypeScript
pnpm run compile

# Run tests
pnpm test

# Package extension
pnpm install -g @vscode/vsce
vsce package
```

## Usage

1. Click the Renamify icon in the Activity Bar (left sidebar)
2. Enter your search and replacement terms
3. Configure options:
   - Select case styles to match
   - Set file include/exclude patterns
   - Add regex to exclude matching lines (e.g., `^\\s*//` for comments)
4. Click "Search" to preview changes
5. Review results with red strikethrough for removals and green for additions
6. Click "Apply" to execute the changes

## Configuration

- `renamify.cliPath`: Path to renamify CLI binary (auto-detected by default)
- `renamify.respectGitignore`: Honor .gitignore files (default: true)
- `renamify.showContextLines`: Number of context lines in results (default: 2)
- `renamify.autoSaveBeforeApply`: Auto-save files before applying (default: true)
- `renamify.confirmBeforeApply`: Show confirmation dialog (default: true)

## Commands

- `Renamify: Open Search Panel` - Open the Renamify search view
- `Renamify: Create Plan` - Create a replacement plan
- `Renamify: Apply Changes` - Apply pending changes
- `Renamify: Undo Last Operation` - Undo the last operation
- `Renamify: Show History` - View operation history
- `Renamify: Clear Results` - Clear search results

## License

Copyright (c) DocSpring. All rights reserved.
