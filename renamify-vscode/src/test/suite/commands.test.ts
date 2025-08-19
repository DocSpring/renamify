import * as assert from 'node:assert/strict';
import * as sinon from 'sinon';
import * as vscode from 'vscode';
import { RenamifyCliService } from '../../cliService';
import { RenamifyCommands } from '../../commands';
import { RenamifyViewProvider } from '../../webviewProvider';

suite('Commands Test Suite', () => {
  let sandbox: sinon.SinonSandbox;
  let commands: RenamifyCommands;
  let cliService: RenamifyCliService;
  let viewProvider: RenamifyViewProvider;
  let showInputBoxStub: sinon.SinonStub;
  let showInformationMessageStub: sinon.SinonStub;
  let showErrorMessageStub: sinon.SinonStub;
  let showWarningMessageStub: sinon.SinonStub;
  let showQuickPickStub: sinon.SinonStub;

  setup(() => {
    sandbox = sinon.createSandbox();

    // Create mocks
    cliService = sandbox.createStubInstance(RenamifyCliService) as any;
    viewProvider = sandbox.createStubInstance(RenamifyViewProvider) as any;

    commands = new RenamifyCommands(cliService, viewProvider);

    // Stub VS Code API methods
    showInputBoxStub = sandbox.stub(vscode.window, 'showInputBox');
    showInformationMessageStub = sandbox.stub(
      vscode.window,
      'showInformationMessage'
    );
    showErrorMessageStub = sandbox.stub(vscode.window, 'showErrorMessage');
    showWarningMessageStub = sandbox.stub(vscode.window, 'showWarningMessage');
    showQuickPickStub = sandbox.stub(vscode.window, 'showQuickPick');

    // Stub workspace configuration
    sandbox.stub(vscode.workspace, 'getConfiguration').returns({
      get: (key: string) => {
        if (key === 'confirmBeforeApply') {
          return true;
        }
        if (key === 'autoSaveBeforeApply') {
          return true;
        }
        return;
      },
    } as any);
  });

  teardown(() => {
    sandbox.restore();
  });

  test('createPlan should prompt for search and replace terms', async () => {
    showInputBoxStub.onFirstCall().resolves('oldName');
    showInputBoxStub.onSecondCall().resolves('newName');

    (cliService.createPlan as sinon.SinonStub).resolves({
      stats: { total_matches: 5 },
    });

    await commands.createPlan();

    assert.ok(showInputBoxStub.calledTwice);
    assert.ok(
      (cliService.createPlan as sinon.SinonStub).calledWith(
        'oldName',
        'newName'
      )
    );
    assert.ok(
      showInformationMessageStub.calledWith('Plan created with 5 matches')
    );
  });

  test('createPlan should handle cancellation', async () => {
    showInputBoxStub.onFirstCall().resolves(undefined);

    await commands.createPlan();

    assert.ok((cliService.createPlan as sinon.SinonStub).notCalled);
  });

  test('applyChanges should ask for confirmation', async () => {
    showWarningMessageStub.resolves('Yes');
    sandbox.stub(vscode.workspace, 'saveAll').resolves(true);
    (cliService.apply as sinon.SinonStub).resolves();

    await commands.applyChanges();

    assert.ok(
      showWarningMessageStub.calledWith(
        'Are you sure you want to apply the changes?',
        'Yes',
        'No'
      )
    );
    assert.ok((cliService.apply as sinon.SinonStub).called);
    assert.ok(
      showInformationMessageStub.calledWith('Changes applied successfully')
    );
  });

  test('applyChanges should cancel if user says no', async () => {
    showWarningMessageStub.resolves('No');

    await commands.applyChanges();

    assert.ok((cliService.apply as sinon.SinonStub).notCalled);
  });

  test('undoLastOperation should undo the last operation', async () => {
    (cliService.history as sinon.SinonStub).resolves([
      { id: 'op-1', old: 'oldName', new: 'newName' },
    ]);
    (cliService.undo as sinon.SinonStub).resolves();

    await commands.undoLastOperation();

    assert.ok((cliService.history as sinon.SinonStub).calledWith(1));
    assert.ok((cliService.undo as sinon.SinonStub).calledWith('op-1'));
    assert.ok(
      showInformationMessageStub.calledWith('Operation undone successfully')
    );
  });

  test('undoLastOperation should handle empty history', async () => {
    (cliService.history as sinon.SinonStub).resolves([]);

    await commands.undoLastOperation();

    assert.ok((cliService.undo as sinon.SinonStub).notCalled);
    assert.ok(showInformationMessageStub.calledWith('No operations to undo'));
  });

  test('showHistory should display history and allow undo/redo', async () => {
    (cliService.history as sinon.SinonStub).resolves([
      {
        id: 'op-1',
        search: 'oldName',
        replace: 'newName',
        created_at: '2024-01-01T00:00:00Z',
        stats: { total_matches: 10 },
      },
    ]);

    showQuickPickStub.resolves({
      label: 'op-1: oldName → newName',
      description: '1/1/2024, 12:00:00 AM - 10 matches',
      id: 'op-1',
    });

    showInformationMessageStub.resolves('Undo');
    (cliService.undo as sinon.SinonStub).resolves();

    await commands.showHistory();

    assert.ok(showQuickPickStub.called);
    assert.ok((cliService.undo as sinon.SinonStub).calledWith('op-1'));
  });

  test('clearResults should post message to view provider', () => {
    commands.clearResults();

    assert.ok(
      (viewProvider.postMessage as sinon.SinonStub).calledWith({
        type: 'clearResults',
      })
    );
  });

  test('Error handling should show error messages', async () => {
    showInputBoxStub.onFirstCall().resolves('oldName');
    showInputBoxStub.onSecondCall().resolves('newName');

    const error = new Error('CLI not found');
    (cliService.createPlan as sinon.SinonStub).rejects(error);

    await commands.createPlan();

    assert.ok(
      showErrorMessageStub.calledWith('Failed to create plan: CLI not found')
    );
  });
});
