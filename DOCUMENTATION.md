# Refaktor Documentation

## Overview

Refaktor is a smart search & replace tool for code and files with case-aware transformations. It understands different naming conventions and can rename both file contents and the files themselves in a single operation.

## Core Features

### Case-Aware Transformations

Refaktor automatically detects and converts between different naming conventions:

- **snake_case** → `old_name` → `new_name`
- **kebab-case** → `old-name` → `new-name`
- **camelCase** → `oldName` → `newName`
- **PascalCase** → `OldName` → `NewName`
- **SCREAMING_SNAKE_CASE** → `OLD_NAME` → `NEW_NAME`
- **Title Case** → `Old Name` → `New Name`
- **Train-Case** → `Old-Name` → `New-Name`
- **dot.case** → `old.name` → `new.name`

When you provide a search and replace pattern, Refaktor will automatically apply the transformation to all detected case variants.

### Contextual Separator Coercion

Refaktor includes intelligent contextual separator coercion that adapts the replacement style based on the surrounding code context:

- **Context-aware replacement**: `refaktor_core::Engine` becomes `smart_search_and_replace_core::Engine` (snake_case context)
- **Path-aware**: `src/refaktor/main.rs` becomes `src/smart-search-and-replace/main.rs` (kebab-case for paths)
- **URL-aware**: `https://github.com/user/refaktor` becomes `https://github.com/user/smart-search-and-replace`
- **Module-aware**: `refaktor::core::apply()` becomes `smart_search_and_replace::core::apply()`

The coercion analyzes the immediate context around each match to determine the most appropriate separator style, making refactoring feel more natural and reducing manual corrections.

### File and Directory Renaming

In addition to replacing text within files, Refaktor can rename:

- Files that match the pattern
- Directories that match the pattern
- Both are renamed in dependency order (deepest directories first)

### Root Directory Renaming

**NEW**: Refaktor can now rename the project root directory itself when the directory name matches your pattern:

- **Default behavior**: Root directory renaming is disabled for safety
- **Enable with `--rename-root`**: Explicitly allow root directory renaming
- **Manual confirmation**: Always requires interactive confirmation
- **Next steps snippet**: Provides instructions for rebuilding/reloading after root rename

**Example:**
```bash
# This will offer to rename the project directory itself
cd /path/to/refaktor-project
refaktor rename refaktor smart_search_and_replace --rename-root

# Refaktor will show a "Next Steps" snippet like:
# Next Steps:
# 1. cd ../smart-search-and-replace-project  
# 2. cargo build --release  # if Rust project
# 3. Update any IDE project settings
```

**Safety considerations:**
- Always commit changes before root directory rename
- Update build scripts, IDE settings, and documentation paths
- Consider impact on CI/CD pipelines that reference the directory name

## Commands

### `plan` - Generate a Refactoring Plan

Creates a plan of all changes that will be made without modifying any files.

```bash
refaktor plan <OLD> <NEW> [OPTIONS]
```

**Options:**

- `--include <PATTERNS>` - Only process files matching these glob patterns (comma-separated)
- `--exclude <PATTERNS>` - Skip files matching these glob patterns (comma-separated)
- `--no-rename-files` - Don't rename matching files
- `--no-rename-dirs` - Don't rename matching directories
- `--styles <STYLES>` - Specify which case styles to use (comma-separated: snake,kebab,camel,pascal,screaming-snake,title,train,dot)
- `--preview-format <FORMAT>` - Output format: table (default), diff, json
- `--plan-out <PATH>` - Where to save the plan (default: .refaktor/plan.json)
- `--dry-run` - Only show preview, don't write plan file

### `apply` - Apply a Refactoring Plan

Executes a previously generated plan.

```bash
refaktor apply [OPTIONS]
```

**Options:**

- `--plan <PATH>` - Path to plan file (default: .refaktor/plan.json)
- `--atomic` - Apply changes atomically (default: true)
- `--commit` - Create a git commit after applying
- `--force-with-conflicts` - Apply even if conflicts are detected

### `undo` - Undo a Previous Refactoring

Reverts a previously applied refactoring using backups.

```bash
refaktor undo <ID>
```

### `redo` - Redo a Previously Undone Refactoring

Re-applies a refactoring that was previously undone.

```bash
refaktor redo <ID>
```

### `status` - Show Current Status

Displays information about the current state of refactoring operations.

```bash
refaktor status
```

### `history` - Show Refactoring History

Lists all previous refactoring operations.

```bash
refaktor history [OPTIONS]
```

**Options:**

- `--limit <N>` - Show only the N most recent entries

### `rename` - Fast Path Plan and Apply

**NEW**: Combines planning and applying in a single command with confirmation prompts. Perfect for quick, interactive refactoring.

```bash
refaktor rename <OLD> <NEW> [OPTIONS]
```

**Options:**

- `--include <PATTERNS>` - Only process files matching these glob patterns
- `--exclude <PATTERNS>` - Skip files matching these glob patterns  
- `--no-rename-files` - Don't rename matching files
- `--no-rename-dirs` - Don't rename matching directories
- `--styles <STYLES>` - Specify which case styles to use (comma-separated)
- `--preview <FORMAT>` - Show preview before confirmation (table, diff, json)
- `--commit` - Create a git commit after applying changes
- `--large` - Acknowledge large changes (>500 files or >100 renames)
- `--force-with-conflicts` - Force apply even with conflicts
- `--confirm-collisions` - Confirm case-insensitive or collision renames
- `--rename-root` - Allow renaming the root project directory (requires confirmation)
- `--no-rename-root` - Never rename the root project directory

**Example:**
```bash
# Quick rename with table preview
refaktor rename old_name new_name --preview table

# Commit changes automatically
refaktor rename getUserName fetchUserProfile --commit
```

### `dry-run` - Preview Changes Without Saving

Alias for `plan --dry-run`. Shows what would be changed without creating a plan file.

```bash
refaktor dry-run <OLD> <NEW> [OPTIONS]
```

### `init` - Initialize Refaktor Ignore Settings

Adds `.refaktor/` to ignore files to prevent tracking of Refaktor's workspace.

```bash
refaktor init [OPTIONS]
```

**Options:**

- `--local` - Add to `.git/info/exclude` instead of `.gitignore`
- `--global` - Add to global git excludes file
- `--check` - Check if `.refaktor` is ignored (exit 0 if yes, 1 if no)
- `--configure-global` - Configure global excludes file if it doesn't exist (requires `--global`)

## File Filtering and Ignore Rules

### Respecting .gitignore (Default Behavior)

By default, Refaktor respects:

- `.gitignore` files at all levels
- `.git/info/exclude`
- Global git ignore rules
- `.rgignore` files (ripgrep-specific ignore files)
- Hidden files and directories are skipped

### Unrestricted Mode (-u/-uu/-uuu)

Refaktor supports ripgrep-style unrestricted flags to reduce filtering:

- `-u` (unrestricted level 1):

  - Disables `.gitignore` files
  - Still respects `.git/info/exclude` and global git ignore
  - Still skips hidden files

- `-uu` (unrestricted level 2):

  - Disables all ignore files
  - Shows hidden files and directories
  - Still skips binary files

- `-uuu` (unrestricted level 3):
  - Disables all ignore files
  - Shows hidden files and directories
  - Treats binary files as text

**Example:**

```bash
# Search including files normally ignored by .gitignore
refaktor plan old_name new_name -u

# Search including hidden files and all ignored files
refaktor plan old_name new_name -uu

# Search absolutely everything, including binary files
refaktor plan old_name new_name -uuu
```

### Include/Exclude Patterns

You can explicitly include or exclude files using glob patterns:

```bash
# Only process Rust source files
refaktor plan old_name new_name --include "**/*.rs"

# Exclude test files
refaktor plan old_name new_name --exclude "**/*test*"

# Multiple patterns (comma-separated)
refaktor plan old_name new_name --include "src/**/*.rs,lib/**/*.rs" --exclude "target/**"
```

## Safety Features

### Atomic Operations

By default, all file modifications are atomic:

- Changes are written to temporary files first
- Files are atomically renamed into place
- Parent directories are synced (on Unix systems)
- If any operation fails, all changes are rolled back

### Backup and Restore

- Before applying changes, Refaktor creates backups of all affected files
- Backups are stored in `.refaktor/backups/<plan-id>/`
- Each backup includes checksums for integrity verification
- The `undo` command can restore from backups at any time

### Conflict Detection

Refaktor detects several types of conflicts:

1. **Multiple-to-One Conflicts**: When multiple files would be renamed to the same destination
2. **Case-Insensitive Filesystem Conflicts**: When renaming would conflict on case-insensitive filesystems
3. **Windows Reserved Names**: Prevents creating files with Windows reserved names (CON, PRN, AUX, etc.)

### History Tracking

- All operations are logged in `.refaktor/history.json`
- Each entry includes:
  - Unique ID and timestamp
  - Original and new patterns
  - All affected files with checksums
  - All rename operations performed
  - Links to backup locations

## Platform Considerations

### Case-Insensitive Filesystems

On macOS and Windows (typically case-insensitive), Refaktor:

- Detects the filesystem type automatically
- Uses two-step renames for case-only changes (e.g., `oldName` → `temp` → `OldName`)
- Warns about potential conflicts

### Windows Compatibility

- Handles Windows path length limitations
- Prevents creation of reserved filenames
- Supports both forward and backward slashes

### Symbolic Links

- By default, symbolic links are not followed
- Symlink files can be renamed if their names match the pattern
- Symlink targets are never modified

## Auto-Initialization

Refaktor automatically prompts to ignore the `.refaktor/` directory on first use. This ensures your workspace files aren't accidentally committed to version control.

### Interactive Prompt

When running commands that create `.refaktor/` for the first time, you'll see:

```
Refaktor uses .refaktor/ for plans, backups, and history.
Ignore it now?
  [Y] Repo .gitignore   [l] Local .git/info/exclude   [g] Global excludesfile   [n] No
Choice (Y/l/g/n): 
```

- **Y (default)**: Add to `.gitignore` in the current directory
- **l**: Add to `.git/info/exclude` (repository-specific, not committed)
- **g**: Add to global git excludes file
- **n**: Skip initialization

### Command-Line Flags

Control auto-initialization behavior with these flags:

- `--auto-init=MODE` - Automatically initialize with specified mode (repo|local|global)
- `--no-auto-init` - Disable automatic initialization
- `-y, --yes` - Assume yes for all prompts (uses repo mode for auto-init)

**Examples:**

```bash
# Always add to .gitignore without prompting
refaktor --auto-init=repo plan old new

# Prevent any auto-initialization
refaktor --no-auto-init plan old new

# Use -y for non-interactive environments
refaktor -y plan old new
```

### CI/CD Usage

For CI/CD pipelines:

```bash
# Check if .refaktor is properly ignored
refaktor init --check || exit 1

# Auto-initialize in CI without prompts
refaktor --auto-init=repo plan old new
```

Non-TTY environments (like CI) will never show prompts unless `--auto-init` is explicitly set.

## Configuration

### Project-Level Settings

Refaktor looks for configuration in `.refaktor/config.toml`:

```toml
# Example configuration (not yet implemented)
[defaults]
respect_gitignore = true
atomic = true
create_backups = true

[patterns]
# Custom ignore patterns
ignore = ["vendor/**", "node_modules/**"]
```

### Environment Variables

- `NO_COLOR` - Disable colored output (respects the NO_COLOR standard)
- `REFAKTOR_YES` - Same as `-y` flag (assume yes for prompts)

## Examples

### Basic Rename

Rename a function across your entire codebase:

```bash
# Generate a plan
refaktor plan getUserName fetchUserProfile

# Review the plan (shows table of changes)
cat .refaktor/plan.json

# Apply the changes
refaktor apply
```

### Rename with Specific Styles

Only transform specific naming conventions:

```bash
# Only handle snake_case and camelCase
refaktor plan old_name new_name --styles snake,camel

# Include the new case styles
refaktor plan old_name new_name --styles snake,kebab,title,train,dot
```

### Rename in Specific Directories

```bash
# Only rename in src/ directory
refaktor plan old_name new_name --include "src/**"

# Exclude tests
refaktor plan old_name new_name --exclude "**/*test*,**/*spec*"
```

### Preview Without Creating Plan

```bash
# Just see what would change
refaktor dry-run old_name new_name

# With diff output
refaktor dry-run old_name new_name --preview-format diff
```

### Fast Interactive Renaming

```bash
# Use the new rename command for quick interactive refactoring
refaktor rename old_name new_name --preview table

# Rename with automatic git commit
refaktor rename getUserName fetchUserProfile --commit --preview diff
```

### Root Directory Projects

```bash
# Rename the entire project directory (requires confirmation)
refaktor rename myproject awesome_project --rename-root --commit

# Or explicitly prevent root directory renaming
refaktor rename myproject awesome_project --no-rename-root
```

### Undo/Redo Operations

```bash
# See history
refaktor history

# Undo the last operation
refaktor undo <id-from-history>

# Redo if you change your mind
refaktor redo <id-from-history>
```

## Best Practices

1. **Use `rename` for quick refactoring** - The new fast-path command with preview and confirmation
2. **Always run `plan` first for large changes** - Review changes before applying them
3. **Use `--dry-run` for exploration** - See what would change without commitment
4. **Commit before large refactors** - Makes it easy to revert if needed
5. **Use specific includes** - Limit scope when working in large codebases  
6. **Check history after apply** - Verify the operation was recorded
7. **Use `--preview` with rename** - Always see what changes before confirming
8. **Be careful with `--rename-root`** - Only use when you want to rename the project directory

## Troubleshooting

### Changes Not Being Found

- Check if files are ignored: Run with `-u` flag to include gitignored files
- Verify file encoding: Refaktor works with UTF-8 and ASCII files
- Check your include/exclude patterns

### Permission Errors

- Ensure you have write permissions for all affected files
- On Windows, close any programs that might have files open

### Undo Fails

- Check that backup files still exist in `.refaktor/backups/`
- Verify file permissions haven't changed
- Ensure no manual changes were made after the refactoring

## Integration

### Git Workflows

```bash
# Commit before refactoring
git add -A && git commit -m "Before refactoring"

# Perform refactoring with auto-commit
refaktor plan old_name new_name
refaktor apply --commit

# Or manually commit after
refaktor apply
git add -A && git commit -m "Refactor: old_name -> new_name"
```

### CI/CD Pipelines

```bash
# Validate no uncommitted refactoring plans
test ! -f .refaktor/plan.json || exit 1

# Or apply pending refactorings automatically
if [ -f .refaktor/plan.json ]; then
  refaktor apply
fi
```

## Appendix

### Supported File Types

Refaktor works with any text file. Binary files are automatically detected and skipped (unless using `-uuu`).

### Performance Considerations

- Uses memory-mapped files for large file reading
- Parallel directory traversal for scanning
- Atomic writes may be slower but ensure safety

### Limitations

- Maximum file size: Limited by available memory for pattern matching
- Path length: OS-dependent (260 chars on Windows without long path support)
- Number of files: No hard limit, but plans with >10,000 files may be slow

---

For more information, see:

- [GitHub Repository](https://github.com/DocSpring/refaktor)
- [Issue Tracker](https://github.com/DocSpring/refaktor/issues)
