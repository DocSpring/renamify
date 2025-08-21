import * as assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { EventEmitter } from 'node:events';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as sinon from 'sinon';
import * as vscode from 'vscode';
import { RenamifyCliService } from '../../cliService';

suite('CLI Service Test Suite', () => {
  let sandbox: sinon.SinonSandbox;
  let cliService: RenamifyCliService;
  let mockSpawn: sinon.SinonStub;

  setup(() => {
    sandbox = sinon.createSandbox();

    // Mock vscode.workspace.getConfiguration to use local dev build
    sandbox.stub(vscode.workspace, 'getConfiguration').returns({
      get: (key: string) => {
        if (key === 'cliPath') {
          // Point to the local dev build using absolute path
          return path.join(
            __dirname,
            '..',
            '..',
            '..',
            '..',
            '..',
            'target',
            'debug',
            'renamify'
          );
        }
        if (key === 'respectGitignore') {
          return true;
        }
        return;
      },
    } as any);

    // Create a mock spawn function
    mockSpawn = sandbox.stub();

    // Create CLI service with mocked spawn
    cliService = new RenamifyCliService(mockSpawn as any);
  });

  teardown(() => {
    sandbox.restore();
  });

  test('Search should call CLI with correct arguments', async () => {
    const mockProcess = new EventEmitter() as any;
    mockProcess.stdout = new EventEmitter();
    mockProcess.stderr = new EventEmitter();

    mockSpawn.returns(mockProcess);

    const searchPromise = cliService.search('oldName', {
      include: '**/*.ts',
      exclude: 'node_modules/**',
      caseStyles: ['camel', 'pascal'],
    });

    // Simulate successful CLI response with wrapper object
    setTimeout(() => {
      mockProcess.stdout.emit(
        'data',
        JSON.stringify({
          success: true,
          operation: 'search',
          plan_id: 'test-plan',
          search: 'oldName',
          replace: '',
          dry_run: false,
          summary: {
            files_with_matches: 1,
            total_matches: 1,
            renames: 0,
          },
          plan: {
            id: 'test-plan',
            created_at: '2024-01-01T00:00:00Z',
            search: 'oldName',
            replace: 'newName',
            styles: [],
            includes: [],
            excludes: [],
            matches: [
              {
                file: 'test.ts',
                line: 10,
                column: 5,
                text: 'oldName',
                replacement: 'newName',
                context: 'const oldName = 123;',
              },
            ],
            paths: [],
            stats: {
              files_scanned: 1,
              total_matches: 1,
              matches_by_variant: {},
              files_with_matches: 1,
            },
            version: '1.0.0',
          },
        })
      );
      mockProcess.emit('close', 0);
    }, 10);

    const results = await searchPromise;

    assert.ok(mockSpawn.calledOnce);
    const args = mockSpawn.firstCall.args[1];
    assert.ok(args.includes('search'));
    assert.ok(args.includes('oldName'));
    assert.ok(args.includes('--output'));
    assert.ok(args.includes('json'));
    assert.ok(args.includes('--include'));
    assert.ok(args.includes('**/*.ts'));
    assert.ok(args.includes('--exclude'));
    assert.ok(args.includes('node_modules/**'));
    assert.ok(args.includes('--only-styles'));
    assert.ok(args.includes('camel,pascal'));

    assert.strictEqual(results.matches.length, 1);
    assert.strictEqual(results.matches[0].file, 'test.ts');
    assert.strictEqual(results.id, 'test-plan');
  });

  test('CLI service performs a real search', async function () {
    this.timeout(5000); // Increase timeout to 5 seconds

    // Create a temporary test directory with a small file to search
    const testDir = path.join(os.tmpdir(), `renamify-test-${Date.now()}`);
    fs.mkdirSync(testDir, { recursive: true });

    // Create a test file with searchable content
    const testFile = path.join(testDir, 'test.txt');
    fs.writeFileSync(testFile, 'This is a test file with test content');

    // Change to test directory to limit search scope
    const originalCwd = process.cwd();
    process.chdir(testDir);

    try {
      const service = new RenamifyCliService();

      // Test basic search with default case styles
      const results = await service.search('test', {
        caseStyles: ['snake'],
      });

      // Results should be a Plan object
      assert.ok(results.id, 'Plan should have an ID');
      assert.ok(Array.isArray(results.matches), 'Matches should be an array');
    } finally {
      // Restore original directory and clean up
      process.chdir(originalCwd);
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  test('CLI service uses --only-styles argument correctly', async () => {
    // Configure the mock to capture arguments
    let capturedArgs: string[] = [];
    mockSpawn.callsFake((command: string, args?: string[], _options?: any) => {
      capturedArgs = args ?? [];
      const proc = spawn(command, args, _options);
      return proc;
    });

    try {
      await cliService.search('test', {
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

  test('Create plan should not use --dry-run flag', async () => {
    const mockProcess = new EventEmitter() as any;
    mockProcess.stdout = new EventEmitter();
    mockProcess.stderr = new EventEmitter();

    mockSpawn.returns(mockProcess);

    const planPromise = cliService.createPlan('oldName', 'newName', {
      dryRun: false,
    });

    setTimeout(() => {
      mockProcess.stdout.emit(
        'data',
        JSON.stringify({
          success: true,
          operation: 'plan',
          plan_id: 'plan-123',
          search: 'oldName',
          replace: 'newName',
          dry_run: false,
          summary: {
            files_with_matches: 1,
            total_matches: 5,
            renames: 0,
          },
          plan: {
            id: 'plan-123',
            stats: { total_matches: 5 },
          },
        })
      );
      mockProcess.emit('close', 0);
    }, 10);

    const plan = (await planPromise) as any;

    const args = mockSpawn.firstCall.args[1];
    assert.ok(!args.includes('--dry-run'));
    assert.ok(args.includes('plan'));
    assert.strictEqual(plan.id, 'plan-123');
  });

  test('Apply should call CLI with correct arguments', async () => {
    const mockProcess = new EventEmitter() as any;
    mockProcess.stdout = new EventEmitter();
    mockProcess.stderr = new EventEmitter();

    mockSpawn.returns(mockProcess);

    const applyPromise = cliService.apply('plan-123');

    setTimeout(() => {
      mockProcess.stdout.emit('data', '');
      mockProcess.emit('close', 0);
    }, 10);

    await applyPromise;

    const args = mockSpawn.firstCall.args[1];
    assert.ok(args.includes('apply'));
    assert.ok(args.includes('--id'));
    assert.ok(args.includes('plan-123'));
  });

  test('Should handle CLI errors correctly', async () => {
    // Temporarily restore the original stub to modify it
    sandbox.restore();
    
    // Re-create the stub with mocked-cli-path to skip version check
    sandbox.stub(vscode.workspace, 'getConfiguration').returns({
      get: (key: string) => {
        if (key === 'cliPath') {
          return 'mocked-cli-path';  // Use special value to skip version check
        }
        if (key === 'respectGitignore') {
          return true;
        }
        return;
      },
    } as any);

    // Re-create the mock spawn
    mockSpawn = sandbox.stub();
    
    // Create a new CLI service with the updated configuration
    const testCliService = new RenamifyCliService(mockSpawn as any);

    // Create two mock processes - one for each attempt (initial + retry)
    const mockProcess1 = new EventEmitter() as any;
    mockProcess1.stdout = new EventEmitter();
    mockProcess1.stderr = new EventEmitter();
    mockProcess1.kill = sandbox.stub();

    const mockProcess2 = new EventEmitter() as any;
    mockProcess2.stdout = new EventEmitter();
    mockProcess2.stderr = new EventEmitter();
    mockProcess2.kill = sandbox.stub();

    // Return first process on first call, second process on retry
    mockSpawn.onFirstCall().returns(mockProcess1);
    mockSpawn.onSecondCall().returns(mockProcess2);

    const searchPromise = testCliService.search('oldName', {});

    // First attempt fails
    setTimeout(() => {
      mockProcess1.stderr.emit('data', 'Error: First attempt failed');
      mockProcess1.emit('close', 1);
    }, 10);

    // Second attempt (retry) also fails
    setTimeout(() => {
      mockProcess2.stderr.emit('data', 'Error: Something went wrong');
      mockProcess2.emit('close', 1);
    }, 120); // After 100ms retry delay

    try {
      await searchPromise;
      assert.fail('Should have thrown an error');
    } catch (error: any) {
      assert.ok(error.message.includes('Something went wrong'));
    }
  });

  test('History should parse JSON response', async () => {
    const mockProcess = new EventEmitter() as any;
    mockProcess.stdout = new EventEmitter();
    mockProcess.stderr = new EventEmitter();

    mockSpawn.returns(mockProcess);

    const historyPromise = cliService.history(5);

    setTimeout(() => {
      mockProcess.stdout.emit(
        'data',
        JSON.stringify([
          {
            id: 'op-1',
            search: 'oldName',
            replace: 'newName',
            created_at: '2024-01-01T00:00:00Z',
            stats: { total_matches: 10 },
          },
        ])
      );
      mockProcess.emit('close', 0);
    }, 10);

    const history = await historyPromise;

    const args = mockSpawn.firstCall.args[1];
    assert.ok(args.includes('history'));
    assert.ok(args.includes('--limit'));
    assert.ok(args.includes('5'));

    assert.strictEqual(history.length, 1);
    assert.strictEqual(history[0].id, 'op-1');
  });
});
