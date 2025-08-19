import * as assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import * as sinon from 'sinon';
import * as vscode from 'vscode';
import { RenamifyCliService } from '../../cliService';

suite('CLI Service Test Suite', () => {
  let sandbox: sinon.SinonSandbox;
  let cliService: RenamifyCliService;
  let mockSpawn: sinon.SinonStub;

  setup(() => {
    sandbox = sinon.createSandbox();

    // Mock vscode.workspace.getConfiguration
    sandbox.stub(vscode.workspace, 'getConfiguration').returns({
      get: (key: string) => {
        if (key === 'cliPath') {
          return '';
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

    const searchPromise = cliService.search('oldName', 'newName', {
      include: '**/*.ts',
      exclude: 'node_modules/**',
      caseStyles: ['camelCase', 'PascalCase'],
    });

    // Simulate successful CLI response
    setTimeout(() => {
      mockProcess.stdout.emit(
        'data',
        JSON.stringify({
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
        })
      );
      mockProcess.emit('close', 0);
    }, 10);

    const results = await searchPromise;

    assert.ok(mockSpawn.calledOnce);
    const args = mockSpawn.firstCall.args[1];
    assert.ok(args.includes('plan'));
    assert.ok(args.includes('oldName'));
    assert.ok(args.includes('newName'));
    assert.ok(args.includes('--dry-run'));
    assert.ok(args.includes('--preview'));
    assert.ok(args.includes('json'));
    assert.ok(args.includes('--include'));
    assert.ok(args.includes('**/*.ts'));
    assert.ok(args.includes('--exclude'));
    assert.ok(args.includes('node_modules/**'));
    assert.ok(args.includes('--only-styles'));
    assert.ok(args.includes('camelCase,PascalCase'));

    assert.strictEqual(results.length, 1);
    assert.strictEqual(results[0].file, 'test.ts');
    assert.strictEqual(results[0].matches.length, 1);
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
          id: 'plan-123',
          stats: { total_matches: 5 },
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
    const mockProcess = new EventEmitter() as any;
    mockProcess.stdout = new EventEmitter();
    mockProcess.stderr = new EventEmitter();

    mockSpawn.returns(mockProcess);

    const searchPromise = cliService.search('oldName', 'newName', {});

    setTimeout(() => {
      mockProcess.stderr.emit('data', 'Error: Something went wrong');
      mockProcess.emit('close', 1);
    }, 10);

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
