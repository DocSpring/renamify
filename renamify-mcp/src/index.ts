#!/usr/bin/env node

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { RenamifyService } from './renamify-service.js';
import { RenamifyTools } from './tools.js';

async function main() {
  const server = new McpServer({
    name: 'renamify-mcp',
    version: '0.1.0',
  });

  const renamifyService = new RenamifyService();
  const tools = new RenamifyTools(renamifyService);

  // Register tools using the new API - inputSchema needs Zod schema properties, not the full schema
  server.registerTool(
    'renamify_plan',
    {
      title: 'Create Renaming Plan',
      description:
        'Create a renaming plan to replace identifiers across a codebase with case-awareness',
      inputSchema: {
        old: z.string().describe('The old name/identifier to replace'),
        new: z.string().describe('The new name/identifier to replace with'),
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
          .describe('Case styles to detect and transform'),
        preview: z
          .enum(['table', 'diff', 'json', 'summary'])
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
      const result = await tools.plan(params);
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
      const result = await tools.apply(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_undo',
    {
      title: 'Undo Renaming',
      description: 'Undo a previously applied renaming',
      inputSchema: {
        id: z.string().describe('History ID to undo'),
      },
    },
    async (params) => {
      const result = await tools.undo(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  server.registerTool(
    'renamify_redo',
    {
      title: 'Redo Renaming',
      description: 'Redo a previously undone renaming',
      inputSchema: {
        id: z.string().describe('History ID to redo'),
      },
    },
    async (params) => {
      const result = await tools.redo(params);
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
      const result = await tools.history(params);
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
      const result = await tools.status(params);
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
          .enum(['table', 'diff', 'json', 'summary'])
          .optional()
          .default('summary')
          .describe('Preview format'),
      },
    },
    async (params) => {
      const result = await tools.preview(params);
      return { content: [{ type: 'text', text: result }] };
    }
  );

  const transport = new StdioServerTransport();
  await server.connect(transport);
  // Server started successfully - MCP servers communicate via stdio
}

main().catch((error) => {
  // Fatal errors need to be reported before exit
  process.stderr.write(`Fatal error: ${error}\n`);
  process.exit(1);
});
