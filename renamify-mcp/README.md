# Renamify MCP Server

MCP (Model Context Protocol) server for Renamify - enabling AI agents to perform intelligent, case-aware renaming operations across codebases.

[![npm version](https://badge.fury.io/js/@renamify%2Fmcp-server.svg)](https://www.npmjs.com/package/@renamify/mcp-server)

Renamify understands different naming conventions and can rename identifiers and files in a single atomic operation. This MCP server makes it available to AI assistants like Claude, Cursor, and other MCP-compatible tools.

## Installation

### Prerequisites

1. **Renamify CLI** must be installed and available in your PATH:

   ```bash
   # Quick install (Linux/macOS)
   curl -fsSL https://docspring.github.io/renamify/install.sh | bash

   # Or download from GitHub releases
   # Or see: https://docspring.github.io/renamify/installation
   ```

2. **Node.js 18+** is required

### Using the MCP Server

The MCP server is designed to be used with AI assistants via npx (no installation needed):

```bash
npx @renamify/mcp-server
```

## Configuration

Add to your MCP client configuration (e.g., for Claude Desktop or Cursor):

```json
{
  "mcpServers": {
    "renamify": {
      "command": "npx",
      "args": ["-y", "@renamify/mcp-server"]
    }
  }
}
```

### Custom Binary Path

If `renamify` is not in your PATH or you want to use a specific binary, set the `RENAMIFY_PATH` environment variable:

```json
{
  "mcpServers": {
    "renamify": {
      "command": "npx",
      "args": ["-y", "@renamify/mcp-server"],
      "env": {
        "RENAMIFY_PATH": "/path/to/renamify"
      }
    }
  }
}
```

### Supported AI Assistants

- **Claude Desktop** - Anthropic's AI assistant
- **Cursor** - AI-powered code editor
- **Continue** - Open-source AI code assistant
- **Any MCP-compatible client** - Following the MCP specification

## Features

- **Case-Aware Transformations**: Automatically handles snake_case, camelCase, PascalCase, kebab-case, and more
- **File and Directory Renaming**: Renames files and directories that match your patterns
- **Safe Operations**: Plan-first workflow with atomic operations and full undo/redo support
- **AI-Optimized Output**: Summary format designed for AI agents to easily parse and understand

## Available Tools

### `renamify_plan`

Create a renaming plan to replace identifiers across a codebase with case-awareness.

**Parameters:**

- `old` (required): The old name/identifier to replace
- `new` (required): The new name/identifier to replace with
- `includes`: Array of glob patterns for files to include
- `excludes`: Array of glob patterns for files to exclude
- `styles`: Array of case styles to detect (snake, camel, pascal, kebab, etc.)
- `preview`: Output format - `table`, `diff`, `json`, or `summary` (default: summary)
- `dryRun`: If true, only preview without creating plan file
- `renameFiles`: Whether to rename files (default: true)
- `renameDirs`: Whether to rename directories (default: true)

**Example Usage (by AI agent):**

```
Tool: renamify_plan
Arguments: {
  "old": "user_name",
  "new": "customer_name",
  "includes": ["src/**/*.ts"],
  "excludes": ["node_modules/**"],
  "styles": ["snake", "camel", "pascal"],
  "preview": "summary"
}
```

### `renamify_apply`

Apply a renaming plan to make the actual changes.

**Parameters:**

- `planId`: Plan ID to apply (uses latest if not specified)
- `planPath`: Path to plan file
- `atomic`: Apply changes atomically (default: true)
- `commit`: Create a git commit after applying (default: false)

**Example Usage (by AI agent):**

```
Tool: renamify_apply
Arguments: {
  "atomic": true,
  "commit": true
}
```

### `renamify_undo`

Undo a previously applied renaming.

**Parameters:**

- `id` (required): History ID to undo

**Example Usage (by AI agent):**

```
Tool: renamify_undo
Arguments: {
  "id": "abc123def456"
}
```

### `renamify_redo`

Redo a previously undone renaming.

**Parameters:**

- `id` (required): History ID to redo

### `renamify_history`

Show renaming history.

**Parameters:**

- `limit`: Number of history entries to show (default: 10)

### `renamify_status`

Show current renamify status and pending plans.

### `renamify_preview`

Preview a plan without applying it.

**Parameters:**

- `planId`: Plan ID to preview
- `planPath`: Path to plan file to preview
- `format`: Preview format - `table`, `diff`, `json`, or `summary`

## Documentation

For comprehensive documentation, visit: https://docspring.github.io/renamify/mcp/

## AI Agent Usage Guide

**Note:** The examples below show how AI agents call MCP tools through the MCP protocol. These are not JavaScript function calls - they represent the tool name and arguments that the AI agent sends to the MCP server.

### Safe Renaming Workflow

1. **Plan First, Apply Later**

   ```
   Always create a plan first to review changes:
   1. Use renamify_plan to see what will change
   2. Review the summary carefully
   3. Use renamify_preview with different formats if needed
   4. Only then use renamify_apply
   ```

2. **Use Appropriate Preview Formats**

   - `summary`: Best for AI agents - simple, structured output
   - `diff`: See exact line-by-line changes
   - `table`: Human-readable tabular format
   - `json`: Full structured data for processing

3. **Safety Features**
   - Plans are atomic by default - all changes succeed or none
   - Full undo/redo support with history tracking
   - Dry run mode for testing without side effects
   - Git integration for automatic commits

### Best Practices for AI Agents

1. **Start with Dry Runs**
   Always test first by using the dry run option:

   ```
   Tool: renamify_plan
   Arguments: {
     "old": "oldName",
     "new": "newName",
     "dryRun": true,
     "preview": "summary"
   }
   ```

2. **Use Specific Includes**
   Target specific directories/files to avoid unintended changes:

   ```
   Tool: renamify_plan
   Arguments: {
     "old": "oldName",
     "new": "newName",
     "includes": ["src/**/*.ts", "lib/**/*.js"],
     "excludes": ["**/*.test.ts", "node_modules/**"]
   }
   ```

3. **Check Status Before Applying**
   Always check what's pending before applying changes:

   ```
   Tool: renamify_status
   Arguments: {}
   ```

   Then if everything looks good:

   ```
   Tool: renamify_apply
   Arguments: {}
   ```

4. **Handle Errors Gracefully**
   - Check if renamify CLI is available
   - Verify files haven't changed since planning
   - Use atomic=false only when debugging specific issues

### Common Scenarios

#### Rename a React Component

```
Tool: renamify_plan
Arguments: {
  "old": "UserProfile",
  "new": "CustomerProfile",
  "includes": ["src/**/*.tsx", "src/**/*.ts"],
  "styles": ["pascal", "camel"],
  "renameFiles": true
}
```

#### Update Database Schema Names

```
Tool: renamify_plan
Arguments: {
  "old": "user_accounts",
  "new": "customer_accounts",
  "includes": ["migrations/**/*.sql", "models/**/*.js"],
  "styles": ["snake"],
  "renameFiles": false
}
Note: renameFiles is false to preserve migration file names
```

#### Renaming API Endpoints

```
Tool: renamify_plan
Arguments: {
  "old": "get-user",
  "new": "get-customer",
  "includes": ["api/**/*.js", "tests/**/*.js"],
  "styles": ["kebab", "camel", "snake"]
}
```

## Development

### Building

```bash
npm run build
```

### Testing

```bash
npm test
```

### Running locally

```bash
npm run dev
```

## Troubleshooting

### "Renamify CLI is not available"

- Ensure `renamify` binary is in your PATH
- Test with: `renamify --version`

### "No matches found"

- Check your include/exclude patterns
- Verify the case styles match your codebase
- Try with fewer restrictions first

### "Conflicts detected"

- Files may have changed since plan creation
- Create a fresh plan with current file state
- Consider using `atomic: false` to skip problematic files

## License

MIT - See LICENSE file for details

## Contributing

Contributions are welcome! Please see the main Renamify repository for contribution guidelines.
