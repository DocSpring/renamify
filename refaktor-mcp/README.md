# Refaktor MCP Server

MCP (Model Context Protocol) server for Refaktor - enabling AI agents to perform intelligent, case-aware refactoring operations across codebases.

[![npm version](https://badge.fury.io/js/@refaktor%2Fmcp-server.svg)](https://www.npmjs.com/package/@refaktor/mcp-server)

Refaktor understands different naming conventions and can rename identifiers and files in a single atomic operation. This MCP server makes it available to AI assistants like Claude, Cursor, and other MCP-compatible tools.

## Installation

### Prerequisites

1. **Refaktor CLI** must be installed and available in your PATH:
   ```bash
   # Quick install (Linux/macOS)
   curl -fsSL https://docspring.github.io/refaktor/install.sh | bash
   
   # Or download from GitHub releases
   # Or see: https://docspring.github.io/refaktor/installation
   ```

2. **Node.js 18+** is required

### Using the MCP Server

The MCP server is designed to be used with AI assistants via npx (no installation needed):

```bash
npx @refaktor/mcp-server
```

## Configuration

Add to your MCP client configuration (e.g., for Claude Desktop or Cursor):

```json
{
  "mcpServers": {
    "refaktor": {
      "command": "npx",
      "args": ["-y", "@refaktor/mcp-server"]
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

### `refaktor_plan`
Create a refactoring plan to replace identifiers across a codebase with case-awareness.

**Parameters:**
- `old` (required): The old name/identifier to replace
- `new` (required): The new name/identifier to replace with
- `includes`: Array of glob patterns for files to include
- `excludes`: Array of glob patterns for files to exclude
- `styles`: Array of case styles to detect (snake, camel, pascal, kebab, etc.)
- `previewFormat`: Output format - `table`, `diff`, `json`, or `summary` (default: summary)
- `dryRun`: If true, only preview without creating plan file
- `renameFiles`: Whether to rename files (default: true)
- `renameDirs`: Whether to rename directories (default: true)

**Example Usage (by AI agent):**
```
Tool: refaktor_plan
Arguments: {
  "old": "user_name",
  "new": "customer_name",
  "includes": ["src/**/*.ts"],
  "excludes": ["node_modules/**"],
  "styles": ["snake", "camel", "pascal"],
  "previewFormat": "summary"
}
```

### `refaktor_apply`
Apply a refactoring plan to make the actual changes.

**Parameters:**
- `planId`: Plan ID to apply (uses latest if not specified)
- `planPath`: Path to plan file
- `atomic`: Apply changes atomically (default: true)
- `commit`: Create a git commit after applying (default: false)

**Example Usage (by AI agent):**
```
Tool: refaktor_apply
Arguments: {
  "atomic": true,
  "commit": true
}
```

### `refaktor_undo`
Undo a previously applied refactoring.

**Parameters:**
- `id` (required): History ID to undo

**Example Usage (by AI agent):**
```
Tool: refaktor_undo
Arguments: {
  "id": "abc123def456"
}
```

### `refaktor_redo`
Redo a previously undone refactoring.

**Parameters:**
- `id` (required): History ID to redo

### `refaktor_history`
Show refactoring history.

**Parameters:**
- `limit`: Number of history entries to show (default: 10)

### `refaktor_status`
Show current refaktor status and pending plans.

### `refaktor_preview`
Preview a plan without applying it.

**Parameters:**
- `planId`: Plan ID to preview
- `planPath`: Path to plan file to preview
- `format`: Preview format - `table`, `diff`, `json`, or `summary`

## Documentation

For comprehensive documentation, visit: https://docspring.github.io/refaktor/mcp/

## AI Agent Usage Guide

**Note:** The examples below show how AI agents call MCP tools through the MCP protocol. These are not JavaScript function calls - they represent the tool name and arguments that the AI agent sends to the MCP server.

### Safe Refactoring Workflow

1. **Plan First, Apply Later**
   ```
   Always create a plan first to review changes:
   1. Use refaktor_plan to see what will change
   2. Review the summary carefully
   3. Use refaktor_preview with different formats if needed
   4. Only then use refaktor_apply
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
   Tool: refaktor_plan
   Arguments: {
     "old": "oldName",
     "new": "newName",
     "dryRun": true,
     "previewFormat": "summary"
   }
   ```

2. **Use Specific Includes**
   Target specific directories/files to avoid unintended changes:
   ```
   Tool: refaktor_plan
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
   Tool: refaktor_status
   Arguments: {}
   ```
   Then if everything looks good:
   ```
   Tool: refaktor_apply
   Arguments: {}
   ```

4. **Handle Errors Gracefully**
   - Check if refaktor CLI is available
   - Verify files haven't changed since planning
   - Use atomic=false only when debugging specific issues

### Common Scenarios

#### Rename a React Component
```
Tool: refaktor_plan
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
Tool: refaktor_plan
Arguments: {
  "old": "user_accounts",
  "new": "customer_accounts",
  "includes": ["migrations/**/*.sql", "models/**/*.js"],
  "styles": ["snake"],
  "renameFiles": false
}
Note: renameFiles is false to preserve migration file names
```

#### Refactor API Endpoints
```
Tool: refaktor_plan
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

### "Refaktor CLI is not available"
- Ensure `refaktor` binary is in your PATH
- Test with: `refaktor --version`

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

Contributions are welcome! Please see the main Refaktor repository for contribution guidelines.