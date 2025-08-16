# Renamify Documentation

## Overview

Renamify is a smart search & replace tool for code and files with case-aware transformations. It understands different naming conventions and can rename both file contents and the files themselves in a single operation.

## Core Features

### Case-Aware Transformations

Renamify automatically detects and converts between different naming conventions:

- **snake_case** → `old_name` → `new_name`
- **kebab-case** → `old-name` → `new-name`
- **camelCase** → `oldName` → `newName`
- **PascalCase** → `OldName` → `NewName`
- **SCREAMING_SNAKE_CASE** → `OLD_NAME` → `NEW_NAME`
- **Title Case** → `Old Name` → `New Name`
- **Train-Case** → `Old-Name` → `New-Name`
- **dot.case** → `old.name` → `new.name`

When you provide a search and replace pattern, Renamify will automatically apply the transformation to all detected case variants.

### Contextual Separator Coercion

Renamify includes intelligent contextual separator coercion that adapts the replacement style based on the surrounding code context:

- **Context-aware replacement**: `renamify_core::Engine` becomes `renamed_renaming_tool_core::Engine` (snake_case context)
- **Path-aware**: `src/renamify/main.rs` becomes `src/renamed-renaming-tool/main.rs` (kebab-case for paths)
- **URL-aware**: `https://github.com/user/renamify` becomes `https://github.com/user/renamed-renaming-tool`
- **Module-aware**: `renamify::core::apply()` becomes `renamed_renaming_tool::core::apply()`

The coercion analyzes the immediate context around each match to determine the most appropriate separator style, making renaming feel more natural and reducing manual corrections.

### File and Directory Renaming

In addition to replacing text within files, Renamify can rename:

- Files that match the pattern
- Directories that match the pattern
- Both are renamed in dependency order (deepest directories first)

### Root Directory Renaming

**NEW**: Renamify can now rename the project root directory itself when the directory name matches your pattern:

- **Default behavior**: Root directory renaming is disabled for safety
- **Enable with `--rename-root`**: Explicitly allow root directory renaming
- **Manual confirmation**: Always requires interactive confirmation
- **Next steps snippet**: Provides instructions for rebuilding/reloading after root rename

**Example:**

```bash
# This will offer to rename the project directory itself
cd /path/to/renamify-project
renamify rename renamify renamed_renaming_tool --rename-root

# Renamify will show a "Next Steps" snippet like:
# Next Steps:
# 1. cd ../renamed-renaming-tool-project
# 2. cargo build --release  # if Rust project
# 3. Update any IDE project settings
```

**Safety considerations:**

- Always commit changes before root directory rename
- Update build scripts, IDE settings, and documentation paths
- Consider impact on CI/CD pipelines that reference the directory name

## Commands

### `plan` - Generate a Renaming Plan

Creates a plan of all changes that will be made without modifying any files.

```bash
renamify plan <OLD> <NEW> [OPTIONS]
```

**Options:**

- `--include <PATTERNS>` - Only process files matching these glob patterns (comma-separated)
- `--exclude <PATTERNS>` - Skip files matching these glob patterns (comma-separated)
- `--no-rename-files` - Don't rename matching files
- `--no-rename-dirs` - Don't rename matching directories
- `--exclude-styles <STYLES>` - Exclude specific case styles from the default set (comma-separated: snake,kebab,camel,pascal,screaming-snake)
- `--include-styles <STYLES>` - Add additional case styles to the active set (comma-separated: title,train,dot)
- `--only-styles <STYLES>` - Use only these case styles, ignoring defaults (comma-separated: any combination)
- `--exclude-match <PATTERNS>` - Skip specific matches (e.g., compound words to ignore)
- `--preview <FORMAT>` - Output format: table (default), diff, json
- `--plan-out <PATH>` - Where to save the plan (default: .renamify/plan.json)
- `--dry-run` - Only show preview, don't write plan file

### `apply` - Apply a Renaming Plan

Executes a previously generated plan.

```bash
renamify apply [OPTIONS]
```

**Options:**

- `--plan <PATH>` - Path to plan file (default: .renamify/plan.json)
- `--atomic` - Apply changes atomically (default: true)
- `--commit` - Create a git commit after applying
- `--force-with-conflicts` - Apply even if conflicts are detected

### `undo` - Undo a Previous Renaming

Reverts a previously applied renaming using backups.

```bash
renamify undo <ID>
```

### `redo` - Redo a Previously Undone Renaming

Re-applies a renaming that was previously undone.

```bash
renamify redo <ID>
```

### `status` - Show Current Status

Displays information about the current state of renaming operations.

```bash
renamify status
```

### `history` - Show Renaming History

Lists all previous renaming operations.

```bash
renamify history [OPTIONS]
```

**Options:**

- `--limit <N>` - Show only the N most recent entries

### `rename` - Fast Path Plan and Apply

**NEW**: Combines planning and applying in a single command with confirmation prompts. Perfect for quick, interactive renaming.

```bash
renamify rename <OLD> <NEW> [OPTIONS]
```

**Options:**

- `--include <PATTERNS>` - Only process files matching these glob patterns
- `--exclude <PATTERNS>` - Skip files matching these glob patterns
- `--no-rename-files` - Don't rename matching files
- `--no-rename-dirs` - Don't rename matching directories
- `--exclude-styles <STYLES>` - Exclude specific case styles from the default set (comma-separated)
- `--include-styles <STYLES>` - Add additional case styles to the active set (comma-separated)
- `--only-styles <STYLES>` - Use only these case styles, ignoring defaults (comma-separated)
- `--exclude-match <PATTERNS>` - Skip specific matches (e.g., compound words to ignore)
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
renamify rename old_name new_name --preview table

# Commit changes automatically
renamify rename getUserName fetchUserProfile --commit
```

### `dry-run` - Preview Changes Without Saving

Alias for `plan --dry-run`. Shows what would be changed without creating a plan file.

```bash
renamify dry-run <OLD> <NEW> [OPTIONS]
```

### `init` - Initialize Renamify Ignore Settings

Adds `.renamify/` to ignore files to prevent tracking of Renamify's workspace.

```bash
renamify init [OPTIONS]
```

**Options:**

- `--local` - Add to `.git/info/exclude` instead of `.gitignore`
- `--global` - Add to global git excludes file
- `--check` - Check if `.renamify` is ignored (exit 0 if yes, 1 if no)
- `--configure-global` - Configure global excludes file if it doesn't exist (requires `--global`)

## File Filtering and Ignore Rules

### Respecting .gitignore (Default Behavior)

By default, Renamify respects:

- `.gitignore` files at all levels
- `.git/info/exclude`
- Global git ignore rules
- `.rgignore` files (ripgrep-specific ignore files)
- Hidden files and directories are skipped

### Unrestricted Mode (-u/-uu/-uuu)

Renamify supports ripgrep-style unrestricted flags to reduce filtering:

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
renamify plan old_name new_name -u

# Search including hidden files and all ignored files
renamify plan old_name new_name -uu

# Search absolutely everything, including binary files
renamify plan old_name new_name -uuu
```

### Include/Exclude Patterns

You can explicitly include or exclude files using glob patterns:

```bash
# Only process Rust source files
renamify plan old_name new_name --include "**/*.rs"

# Exclude test files
renamify plan old_name new_name --exclude "**/*test*"

# Multiple patterns (comma-separated)
renamify plan old_name new_name --include "src/**/*.rs,lib/**/*.rs" --exclude "target/**"
```

## Safety Features

### Atomic Operations

By default, all file modifications are atomic:

- Changes are written to temporary files first
- Files are atomically renamed into place
- Parent directories are synced (on Unix systems)
- If any operation fails, all changes are rolled back

### Backup and Restore

- Before applying changes, Renamify creates backups of all affected files
- Backups are stored in `.renamify/backups/<plan-id>/`
- Each backup includes checksums for integrity verification
- The `undo` command can restore from backups at any time

### Conflict Detection

Renamify detects several types of conflicts:

1. **Multiple-to-One Conflicts**: When multiple files would be renamed to the same destination
2. **Case-Insensitive Filesystem Conflicts**: When renaming would conflict on case-insensitive filesystems
3. **Windows Reserved Names**: Prevents creating files with Windows reserved names (CON, PRN, AUX, etc.)

### History Tracking

- All operations are logged in `.renamify/history.json`
- Each entry includes:
  - Unique ID and timestamp
  - Original and new patterns
  - All affected files with checksums
  - All rename operations performed
  - Links to backup locations

## Platform Considerations

### Case-Insensitive Filesystems

On macOS and Windows (typically case-insensitive), Renamify:

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

Renamify automatically prompts to ignore the `.renamify/` directory on first use. This ensures your workspace files aren't accidentally committed to version control.

### Interactive Prompt

When running commands that create `.renamify/` for the first time, you'll see:

```
Renamify uses .renamify/ for plans, backups, and history.
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
renamify --auto-init=repo plan old new

# Prevent any auto-initialization
renamify --no-auto-init plan old new

# Use -y for non-interactive environments
renamify -y plan old new
```

### CI/CD Usage

For CI/CD pipelines:

```bash
# Check if .renamify is properly ignored
renamify init --check || exit 1

# Auto-initialize in CI without prompts
renamify --auto-init=repo plan old new
```

Non-TTY environments (like CI) will never show prompts unless `--auto-init` is explicitly set.

## Configuration

### Project-Level Settings

Renamify looks for configuration in `.renamify/config.toml`:

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
- `RENAMIFY_YES` - Same as `-y` flag (assume yes for prompts)

## Examples

### Basic Rename

Rename a function across your entire codebase:

```bash
# Generate a plan
renamify plan getUserName fetchUserProfile

# Review the plan (shows table of changes)
cat .renamify/plan.json

# Apply the changes
renamify apply
```

### Rename with Specific Styles

Only transform specific naming conventions:

```bash
# Only handle snake_case and camelCase
renamify plan old_name new_name --styles snake,camel

# Include the new case styles
renamify plan old_name new_name --styles snake,kebab,title,train,dot
```

### Rename in Specific Directories

```bash
# Only rename in src/ directory
renamify plan old_name new_name --include "src/**"

# Exclude tests
renamify plan old_name new_name --exclude "**/*test*,**/*spec*"
```

### Preview Without Creating Plan

```bash
# Just see what would change
renamify dry-run old_name new_name

# With diff output
renamify dry-run old_name new_name --preview diff
```

### Fast Interactive Renaming

```bash
# Use the new rename command for quick interactive renaming
renamify rename old_name new_name --preview table

# Rename with automatic git commit
renamify rename getUserName fetchUserProfile --commit --preview diff
```

### Excluding Specific Matches

Use `--exclude-match` to skip specific compound words or identifiers:

```bash
# Skip specific compound words that shouldn't be changed
renamify plan foo bar --exclude-match bazFooQux,FooService

# Useful when a pattern accidentally matches unintended identifiers
renamify rename config settings --exclude-match ConfigurationManager
```

### Root Directory Projects

```bash
# Rename the entire project directory (requires confirmation)
renamify rename myproject awesome_project --rename-root --commit

# Or explicitly prevent root directory renaming
renamify rename myproject awesome_project --no-rename-root
```

### Undo/Redo Operations

```bash
# See history
renamify history

# Undo the last operation
renamify undo <id-from-history>

# Redo if you change your mind
renamify redo <id-from-history>
```

## Best Practices

1. **Use `rename` for quick renaming** - The new fast-path command with preview and confirmation
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
- Verify file encoding: Renamify works with UTF-8 and ASCII files
- Check your include/exclude patterns

### Permission Errors

- Ensure you have write permissions for all affected files
- On Windows, close any programs that might have files open

### Undo Fails

- Check that backup files still exist in `.renamify/backups/`
- Verify file permissions haven't changed
- Ensure no manual changes were made after the renaming

## Integration

### Git Workflows

```bash
# Commit before renaming
git add -A && git commit -m "Before renaming"

# Perform renaming with auto-commit
renamify plan old_name new_name
renamify apply --commit

# Or manually commit after
renamify apply
git add -A && git commit -m "Refactor: old_name -> new_name"
```

### CI/CD Pipelines

```bash
# Validate no uncommitted renaming plans
test ! -f .renamify/plan.json || exit 1

# Or apply pending refactorings automatically
if [ -f .renamify/plan.json ]; then
  renamify apply
fi
```

## Appendix

### Supported File Types

Renamify works with any text file. Binary files are automatically detected and skipped (unless using `-uuu`).

### Performance Considerations

- Uses memory-mapped files for large file reading
- Parallel directory traversal for scanning
- Atomic writes may be slower but ensure safety

### Concurrent Process Protection

Renamify uses a lock file mechanism to prevent concurrent operations:

- Lock file is created in `.renamify/renamify.lock` during plan/apply/rename operations
- Contains process ID and timestamp for tracking
- Automatically cleaned up when operations complete
- Stale locks (older than 5 minutes) are automatically removed
- If a lock exists from a crashed process, it can be manually removed:
  ```bash
  rm .renamify/renamify.lock
  ```

### Limitations

- Maximum file size: Limited by available memory for pattern matching
- Path length: OS-dependent (260 chars on Windows without long path support)
- Number of files: No hard limit, but plans with >10,000 files may be slow

### Why "Renamify"?

- Looks cool
- Pronounced the same as "refactor"
- The name of the tool explains exactly what it does

---

For more information, see:

- [GitHub Repository](https://github.com/DocSpring/renamify)
- [Issue Tracker](https://github.com/DocSpring/renamify/issues)
