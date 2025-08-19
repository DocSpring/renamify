#!/usr/bin/env node

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { RenamifyService } from './renamify-service.js';
import { RenamifyTools } from './tools.js';

export function createServer(
  renamifyService?: RenamifyService,
  tools?: RenamifyTools
) {
  const server = new McpServer({
    name: 'renamify-mcp',
    version: '0.1.0',
  });

  const service = renamifyService || new RenamifyService();
  const toolsInstance = tools || new RenamifyTools(service);

  // Register tools using the new API - inputSchema needs Zod schema properties, not the full schema
  server.registerTool(
    'renamify_plan',
    {
      title: 'Create Renaming Plan',
      description:
        'Create a renaming plan to replace identifiers across a codebase with case-awareness',
      inputSchema: {
        search: z.string().describe('The name/identifier to search for'),
        replace: z.string().describe('The name/identifier to replace with'),
        paths: z
          .array(z.string())
          .optional()
          .describe(
            'Paths to search (files or directories). Defaults to current directory'
          ),
        includes: z
          .array(z.string())
          .optional()
          .describe('Glob patterns for files to include'),
        excludes: z
          .array(z.string())
          .optional()
          .describe('Glob patterns for files to exclude'),
        styles: z
          .array(z.string())
          .optional()
          .describe(
            'Case styles to detect and transform (snake, kebab, camel, pascal, screaming-snake, title, train, dot)'
          ),
        preview: z
          .enum(['table', 'diff', 'summary'])
          .optional()
          .default('summary')
          .describe('Output format for the plan'),
        dryRun: z
          .boolean()
          .optional()
          .default(false)
          .describe('If true, only preview without creating plan file'),
        renameFiles: z
          .boolean()
          .optional()
          .default(true)
          .describe('Whether to rename files'),
        renameDirs: z
          .boolean()
          .optional()
          .default(true)
          .describe('Whether to rename directories'),
      },
    },
    async (params) => {
      const result = await toolsInstance.plan(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_search',
    {
      title: 'Search for Identifiers',
      description:
        'Search for identifiers across a codebase without making replacements',
      inputSchema: {
        search: z.string().describe('The name/identifier to search for'),
        paths: z
          .array(z.string())
          .optional()
          .describe(
            'Paths to search (files or directories). Defaults to current directory'
          ),
        includes: z
          .array(z.string())
          .optional()
          .describe('Glob patterns for files to include'),
        excludes: z
          .array(z.string())
          .optional()
          .describe('Glob patterns for files to exclude'),
        styles: z
          .array(z.string())
          .optional()
          .describe(
            'Case styles to detect and transform (snake, kebab, camel, pascal, screaming-snake, title, train, dot)'
          ),
        preview: z
          .enum(['table', 'matches', 'summary'])
          .optional()
          .default('summary')
          .describe('Output format for the search results'),
        dryRun: z
          .boolean()
          .optional()
          .default(false)
          .describe('If true, only preview without creating result file'),
        renameFiles: z
          .boolean()
          .optional()
          .default(true)
          .describe('Whether to include file renames in search'),
        renameDirs: z
          .boolean()
          .optional()
          .default(true)
          .describe('Whether to include directory renames in search'),
      },
    },
    async (params) => {
      const result = await toolsInstance.search(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_apply',
    {
      title: 'Apply Renaming Plan',
      description: 'Apply a renaming plan to make the actual changes',
      inputSchema: {
        planId: z
          .string()
          .optional()
          .describe('Plan ID to apply (uses latest if not specified)'),
        planPath: z.string().optional().describe('Path to plan file'),
        atomic: z
          .boolean()
          .optional()
          .default(true)
          .describe('Apply changes atomically'),
        commit: z
          .boolean()
          .optional()
          .default(false)
          .describe('Create a git commit after applying'),
      },
    },
    async (params) => {
      const result = await toolsInstance.apply(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_undo',
    {
      title: 'Undo Renaming',
      description: 'Undo a previously applied renaming',
      inputSchema: {
        id: z
          .string()
          .optional()
          .describe(
            'History ID to undo (use "latest" for most recent non-revert entry, defaults to "latest")'
          ),
      },
    },
    async (params) => {
      const result = await toolsInstance.undo(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_redo',
    {
      title: 'Redo Renaming',
      description: 'Redo a previously undone renaming',
      inputSchema: {
        id: z
          .string()
          .optional()
          .describe(
            'History ID to redo (use "latest" for most recent reverted entry, defaults to "latest")'
          ),
      },
    },
    async (params) => {
      const result = await toolsInstance.redo(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_history',
    {
      title: 'Show History',
      description: 'Show renaming history',
      inputSchema: {
        limit: z
          .number()
          .optional()
          .default(10)
          .describe('Number of history entries to show'),
      },
    },
    async (params) => {
      const result = await toolsInstance.history(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_status',
    {
      title: 'Show Status',
      description: 'Show current renamify status and pending plans',
      inputSchema: {},
    },
    async (params) => {
      const result = await toolsInstance.status(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_preview',
    {
      title: 'Preview Plan',
      description: 'Preview a plan without applying it',
      inputSchema: {
        planId: z.string().optional().describe('Plan ID to preview'),
        planPath: z
          .string()
          .optional()
          .describe('Path to plan file to preview'),
        format: z
          .enum(['table', 'diff', 'summary'])
          .optional()
          .default('summary')
          .describe('Preview format'),
      },
    },
    async (params) => {
      const result = await toolsInstance.preview(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  return server;
}

export async function main() {
  const server = createServer();
  const transport = new StdioServerTransport();
  await server.connect(transport);
  // Server started successfully - MCP servers communicate via stdio
}

// Only run main if this file is executed directly
import { pathToFileURL } from 'node:url';

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  main().catch((error) => {
    // Fatal errors need to be reported before exit
    process.stderr.write(`Fatal error: ${error}\n`);
    process.exit(1);
  });
}
