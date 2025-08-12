#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { RefaktorService } from './refaktor-service.js';
import { RefaktorTools } from './tools.js';

// Tool schemas
const PlanToolSchema = z.object({
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
});

const ApplyToolSchema = z.object({
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
});

const UndoToolSchema = z.object({
  id: z.string().describe('History ID to undo'),
});

const RedoToolSchema = z.object({
  id: z.string().describe('History ID to redo'),
});

const HistoryToolSchema = z.object({
  limit: z
    .number()
    .optional()
    .default(10)
    .describe('Number of history entries to show'),
});

const StatusToolSchema = z.object({});

const PreviewToolSchema = z.object({
  planId: z.string().optional().describe('Plan ID to preview'),
  planPath: z.string().optional().describe('Path to plan file to preview'),
  format: z
    .enum(['table', 'diff', 'json', 'summary'])
    .optional()
    .default('summary')
    .describe('Preview format'),
});

async function main() {
  const server = new Server(
    {
      name: 'refaktor-mcp',
      version: '0.1.0',
    },
    {
      capabilities: {
        tools: {},
      },
    }
  );

  const refaktorService = new RefaktorService();
  const tools = new RefaktorTools(refaktorService);

  // Register tool handlers
  server.setRequestHandler('tools/list', async () => ({
    tools: [
      {
        name: 'refaktor_plan',
        description:
          'Create a refactoring plan to replace identifiers across a codebase with case-awareness',
        inputSchema: {
          type: 'object',
          properties: {
            old: {
              type: 'string',
              description: 'The old name/identifier to replace',
            },
            new: {
              type: 'string',
              description: 'The new name/identifier to replace with',
            },
            includes: {
              type: 'array',
              items: { type: 'string' },
              description: 'Glob patterns for files to include',
            },
            excludes: {
              type: 'array',
              items: { type: 'string' },
              description: 'Glob patterns for files to exclude',
            },
            styles: {
              type: 'array',
              items: { type: 'string' },
              description:
                'Case styles to detect (snake, camel, pascal, kebab, etc.)',
            },
            preview: {
              type: 'string',
              enum: ['table', 'diff', 'json', 'summary'],
              default: 'summary',
              description: 'Output format for the plan',
            },
            dryRun: {
              type: 'boolean',
              default: false,
              description: 'If true, only preview without creating plan file',
            },
            renameFiles: {
              type: 'boolean',
              default: true,
              description: 'Whether to rename files',
            },
            renameDirs: {
              type: 'boolean',
              default: true,
              description: 'Whether to rename directories',
            },
          },
          required: ['old', 'new'],
        },
      },
      {
        name: 'refaktor_apply',
        description: 'Apply a refactoring plan to make the actual changes',
        inputSchema: {
          type: 'object',
          properties: {
            planId: {
              type: 'string',
              description: 'Plan ID to apply (uses latest if not specified)',
            },
            planPath: { type: 'string', description: 'Path to plan file' },
            atomic: {
              type: 'boolean',
              default: true,
              description: 'Apply changes atomically',
            },
            commit: {
              type: 'boolean',
              default: false,
              description: 'Create a git commit after applying',
            },
          },
        },
      },
      {
        name: 'refaktor_undo',
        description: 'Undo a previously applied refactoring',
        inputSchema: {
          type: 'object',
          properties: {
            id: { type: 'string', description: 'History ID to undo' },
          },
          required: ['id'],
        },
      },
      {
        name: 'refaktor_redo',
        description: 'Redo a previously undone refactoring',
        inputSchema: {
          type: 'object',
          properties: {
            id: { type: 'string', description: 'History ID to redo' },
          },
          required: ['id'],
        },
      },
      {
        name: 'refaktor_history',
        description: 'Show refactoring history',
        inputSchema: {
          type: 'object',
          properties: {
            limit: {
              type: 'number',
              default: 10,
              description: 'Number of history entries to show',
            },
          },
        },
      },
      {
        name: 'refaktor_status',
        description: 'Show current refaktor status and pending plans',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
      {
        name: 'refaktor_preview',
        description: 'Preview a plan without applying it',
        inputSchema: {
          type: 'object',
          properties: {
            planId: { type: 'string', description: 'Plan ID to preview' },
            planPath: {
              type: 'string',
              description: 'Path to plan file to preview',
            },
            format: {
              type: 'string',
              enum: ['table', 'diff', 'json', 'summary'],
              default: 'summary',
              description: 'Preview format',
            },
          },
        },
      },
    ],
  }));

  server.setRequestHandler(
    'tools/call',
    async (request: { params: { name: string; arguments: unknown } }) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'refaktor_plan': {
            const params = PlanToolSchema.parse(args);
            const result = await tools.plan(params);
            return { content: [{ type: 'text', text: result }] };
          }
          case 'refaktor_apply': {
            const params = ApplyToolSchema.parse(args);
            const result = await tools.apply(params);
            return { content: [{ type: 'text', text: result }] };
          }
          case 'refaktor_undo': {
            const params = UndoToolSchema.parse(args);
            const result = await tools.undo(params);
            return { content: [{ type: 'text', text: result }] };
          }
          case 'refaktor_redo': {
            const params = RedoToolSchema.parse(args);
            const result = await tools.redo(params);
            return { content: [{ type: 'text', text: result }] };
          }
          case 'refaktor_history': {
            const params = HistoryToolSchema.parse(args);
            const result = await tools.history(params);
            return { content: [{ type: 'text', text: result }] };
          }
          case 'refaktor_status': {
            const params = StatusToolSchema.parse(args);
            const result = await tools.status(params);
            return { content: [{ type: 'text', text: result }] };
          }
          case 'refaktor_preview': {
            const params = PreviewToolSchema.parse(args);
            const result = await tools.preview(params);
            return { content: [{ type: 'text', text: result }] };
          }
          default:
            throw new Error(`Unknown tool: ${name}`);
        }
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        return {
          content: [{ type: 'text', text: `Error: ${errorMessage}` }],
          isError: true,
        };
      }
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
