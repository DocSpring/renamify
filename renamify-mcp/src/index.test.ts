import { spawn } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { afterEach, describe, expect, it } from 'vitest';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

describe('MCP Server', () => {
  let serverProcess: ReturnType<typeof spawn> | null = null;

  afterEach(() => {
    if (serverProcess) {
      serverProcess.kill();
      serverProcess = null;
    }
  });

  it('should start without errors', async () => {
    const serverPath = join(__dirname, 'index.ts');

    // Start the server with bun
    serverProcess = spawn('bun', ['run', serverPath], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    // Wait for any immediate errors
    await new Promise((resolve, reject) => {
      let errorOutput = '';

      serverProcess?.stderr?.on('data', (data) => {
        errorOutput += data.toString();
        if (errorOutput.includes('Fatal error')) {
          reject(new Error(`Server failed to start: ${errorOutput}`));
        }
      });

      // Give it a moment to start
      setTimeout(() => {
        if (!errorOutput.includes('Fatal error')) {
          resolve(undefined);
        }
      }, 500);
    });

    // Send a proper MCP initialization message
    const initMessage = `${JSON.stringify({
      jsonrpc: '2.0',
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'test-client',
          version: '1.0.0',
        },
      },
      id: 1,
    })}\n`;

    serverProcess?.stdin?.write(initMessage);

    // Wait for response
    const response = await new Promise<string>((resolve, reject) => {
      let output = '';

      serverProcess?.stdout?.on('data', (data) => {
        output += data.toString();
        // MCP responses are newline-delimited JSON
        if (output.includes('\n')) {
          resolve(output);
        }
      });

      setTimeout(() => {
        reject(new Error('No response from server'));
      }, 2000);
    });

    // Parse and verify response
    const responseObj = JSON.parse(response.trim());
    expect(responseObj).toHaveProperty('jsonrpc', '2.0');
    expect(responseObj).toHaveProperty('id', 1);
    expect(responseObj).toHaveProperty('result');
    expect(responseObj.result).toHaveProperty('protocolVersion');
    expect(responseObj.result).toHaveProperty('capabilities');
    expect(responseObj.result.capabilities).toHaveProperty('tools');
    expect(responseObj.result.serverInfo).toEqual({
      name: 'renamify-mcp',
      version: '0.1.0',
    });
  });

  it('should list available tools', async () => {
    const serverPath = join(__dirname, 'index.ts');

    serverProcess = spawn('bun', ['run', serverPath], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    // Collect all responses
    const responses: string[] = [];
    serverProcess?.stdout?.on('data', (data) => {
      const lines = data
        .toString()
        .split('\n')
        .filter((l: string) => l.trim());
      responses.push(...lines);
    });

    // Initialize first
    const initMessage = `${JSON.stringify({
      jsonrpc: '2.0',
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'test-client',
          version: '1.0.0',
        },
      },
      id: 1,
    })}\n`;

    serverProcess?.stdin?.write(initMessage);

    // Wait for init response
    await new Promise<void>((resolve) => {
      const checkResponse = () => {
        if (responses.length > 0) {
          resolve();
        } else {
          setTimeout(checkResponse, 50);
        }
      };
      checkResponse();
    });

    // Clear responses for next request
    responses.length = 0;

    // Request tools list
    const listToolsMessage = `${JSON.stringify({
      jsonrpc: '2.0',
      method: 'tools/list',
      params: {},
      id: 2,
    })}\n`;

    serverProcess?.stdin?.write(listToolsMessage);

    // Wait for tools/list response
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('No response from server for tools/list'));
      }, 2000);

      const checkResponse = () => {
        if (responses.length > 0) {
          clearTimeout(timeout);
          resolve();
        } else {
          setTimeout(checkResponse, 50);
        }
      };
      checkResponse();
    });

    const responseObj = JSON.parse(responses[0]);

    // Check if there's an error
    if (responseObj.error) {
      throw new Error(
        `Server returned error: ${JSON.stringify(responseObj.error)}`
      );
    }

    expect(responseObj).toHaveProperty('result');
    expect(responseObj.result).toHaveProperty('tools');

    const tools = responseObj.result.tools;
    expect(tools).toBeInstanceOf(Array);
    expect(tools.length).toBe(7);

    const toolNames = tools.map((t: { name: string }) => t.name);
    expect(toolNames).toContain('renamify_plan');
    expect(toolNames).toContain('renamify_apply');
    expect(toolNames).toContain('renamify_undo');
    expect(toolNames).toContain('renamify_redo');
    expect(toolNames).toContain('renamify_history');
    expect(toolNames).toContain('renamify_status');
    expect(toolNames).toContain('renamify_preview');
  });
});
