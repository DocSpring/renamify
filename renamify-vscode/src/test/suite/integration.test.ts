import * as assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { RenamifyCliService } from '../../cliService';

suite('Integration Test Suite', () => {
  test('CLI service works with real renamify binary', async function () {
    // Skip test if renamify is not installed
    const checkCli = spawn('which', ['renamify']);
    const hasRenamify = await new Promise<boolean>((resolve) => {
      checkCli.on('close', (code) => resolve(code === 0));
    });

    if (!hasRenamify) {
      this.skip();
      return;
    }

    const service = new RenamifyCliService();

    // Test basic search with default case styles
    const results = await service.search('test', 'renamed', {
      caseStyles: ['original', 'snake', 'kebab', 'camel', 'pascal'],
    });

    // Results should be an array (may be empty if no matches)
    assert.ok(Array.isArray(results));
  });

  test('CLI service uses --only-styles argument correctly', async function () {
    // Skip test if renamify is not installed
    const checkCli = spawn('which', ['renamify']);
    const hasRenamify = await new Promise<boolean>((resolve) => {
      checkCli.on('close', (code) => resolve(code === 0));
    });

    if (!hasRenamify) {
      this.skip();
      return;
    }

    let capturedArgs: string[] = [];
    const mockSpawn = (command: string, args?: string[], _options?: any) => {
      capturedArgs = args ?? [];
      const proc = spawn(command, args, _options);
      return proc;
    };

    const service = new RenamifyCliService(mockSpawn as any);

    try {
      await service.search('test', 'renamed', {
        caseStyles: ['snake', 'kebab'],
      });
    } catch {
      // Error is expected if no matches found
    }

    // Verify that --only-styles was used, not --styles
    assert.ok(capturedArgs.includes('--only-styles'));
    assert.ok(!capturedArgs.includes('--styles'));
    assert.ok(capturedArgs.includes('snake,kebab'));
  });
});
