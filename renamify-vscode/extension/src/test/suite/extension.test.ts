import * as assert from 'node:assert/strict';
import * as vscode from 'vscode';

suite('Extension Test Suite', () => {
  vscode.window.showInformationMessage('Start all tests.');

  test('Extension should be present', () => {
    assert.ok(vscode.extensions.getExtension('DocSpring.renamify'));
  });

  test('Should register all commands', async () => {
    // Activate the extension first
    const extension = vscode.extensions.getExtension('DocSpring.renamify');
    if (extension && !extension.isActive) {
      await extension.activate();
    }

    const commands = await vscode.commands.getCommands();

    assert.ok(commands.includes('renamify.search'));
    assert.ok(commands.includes('renamify.plan'));
    assert.ok(commands.includes('renamify.apply'));
    assert.ok(commands.includes('renamify.undo'));
    assert.ok(commands.includes('renamify.history'));
    assert.ok(commands.includes('renamify.clearResults'));
  });

  test('Should activate extension', async () => {
    const extension = vscode.extensions.getExtension('DocSpring.renamify');
    if (extension) {
      await extension.activate();
      assert.ok(extension.isActive);
    }
  });

  test('Should register webview view provider', async () => {
    const extension = vscode.extensions.getExtension('DocSpring.renamify');
    if (extension) {
      await extension.activate();

      // Try to focus the view
      try {
        await vscode.commands.executeCommand('renamify.searchView.focus');
        assert.ok(true, 'View provider is registered');
      } catch (_error) {
        assert.fail('View provider not registered');
      }
    }
  });

  test('Configuration should have correct defaults', () => {
    const config = vscode.workspace.getConfiguration('renamify');

    assert.strictEqual(config.get('respectGitignore'), true);
    assert.strictEqual(config.get('showContextLines'), 2);
    assert.strictEqual(config.get('autoSaveBeforeApply'), true);
    assert.strictEqual(config.get('confirmBeforeApply'), false);
  });
});
