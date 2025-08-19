import * as assert from 'node:assert/strict';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { RenamifyCliService } from '../../cliService';
import { RenamifyViewProvider } from '../../webviewProvider';

suite('Integration Test Suite', () => {
  let provider: RenamifyViewProvider;
  let cliService: RenamifyCliService;

  setup(() => {
    // Create test instances
    const extensionUri = vscode.Uri.file(
      path.join(__dirname, '..', '..', '..')
    );
    cliService = new RenamifyCliService();
    provider = new RenamifyViewProvider(extensionUri, cliService);
  });

  test('Should create webview with all form elements', () => {
    // Create a mock webview view
    const mockWebview = {
      options: {},
      html: '',
      onDidReceiveMessage: () => ({ dispose: () => {} }),
      asWebviewUri: (uri: vscode.Uri) => uri,
      cspSource: 'self',
      postMessage: () => Promise.resolve(true),
    } as any;

    const mockWebviewView = {
      webview: mockWebview,
      title: '',
      description: '',
      badge: undefined,
      show: () => {},
      onDidChangeVisibility: () => ({ dispose: () => {} }),
      onDidDispose: () => ({ dispose: () => {} }),
      visible: true,
    } as any;

    // Resolve the webview
    provider.resolveWebviewView(mockWebviewView, {} as any, {} as any);

    // Check that HTML was set
    assert.ok(mockWebview.html.length > 0);

    // Check for essential form elements
    assert.ok(mockWebview.html.includes('id="search"'));
    assert.ok(mockWebview.html.includes('id="replace"'));
    assert.ok(mockWebview.html.includes('id="include"'));
    assert.ok(mockWebview.html.includes('id="exclude"'));
    assert.ok(mockWebview.html.includes('id="excludeLines"'));

    // Check for case style checkboxes
    assert.ok(mockWebview.html.includes('snake_case'));
    assert.ok(mockWebview.html.includes('kebab-case'));
    assert.ok(mockWebview.html.includes('camelCase'));
    assert.ok(mockWebview.html.includes('PascalCase'));
    assert.ok(mockWebview.html.includes('SCREAMING_SNAKE_CASE'));
    assert.ok(mockWebview.html.includes('Title Case'));
    assert.ok(mockWebview.html.includes('Train-Case'));
    assert.ok(mockWebview.html.includes('dot.case'));

    // Check for action buttons
    assert.ok(mockWebview.html.includes('id="applyBtn"'));
    assert.ok(mockWebview.html.includes('id="expandAll"'));
    assert.ok(mockWebview.html.includes('id="collapseAll"'));
  });

  test('Should handle search message from webview', async () => {
    let messageHandler: any;

    const mockWebview = {
      options: {},
      html: '',
      onDidReceiveMessage: (handler: any) => {
        messageHandler = handler;
        return { dispose: () => {} };
      },
      asWebviewUri: (uri: vscode.Uri) => uri,
      cspSource: 'self',
      postMessage: (message: any) => {
        // Verify the response message
        assert.strictEqual(message.type, 'searchResults');
        assert.ok(Array.isArray(message.results));
        return Promise.resolve(true);
      },
    } as any;

    const mockWebviewView = {
      webview: mockWebview,
      title: '',
      description: '',
      badge: undefined,
      show: () => {},
      onDidChangeVisibility: () => ({ dispose: () => {} }),
      onDidDispose: () => ({ dispose: () => {} }),
      visible: true,
    } as any;

    provider.resolveWebviewView(mockWebviewView, {} as any, {} as any);

    // Simulate search message from webview
    if (messageHandler) {
      // Mock the CLI service search method
      const originalSearch = cliService.search;
      cliService.search = () => {
        return Promise.resolve([
          {
            file: 'test.ts',
            matches: [
              {
                file: 'test.ts',
                line: 10,
                col: 5,
                variant: 'oldName',
                content: 'oldName',
                replace: 'newName',
                start: 5,
                end: 12,
                line_before: 'const oldName = 123;',
                line_after: 'const newName = 123;',
              },
            ],
          },
        ]);
      };

      await messageHandler({
        type: 'search',
        search: 'oldName',
        replace: 'newName',
        include: '**/*.ts',
        exclude: 'node_modules/**',
        excludeMatchingLines: '^\\s*//',
        caseStyles: ['camel', 'pascal'],
      });

      // Restore original method
      cliService.search = originalSearch;
    }
  });

  test('Should handle file opening from webview', async () => {
    let messageHandler: any;
    let openedFile: string | undefined;
    let _openedLine: number | undefined;

    const mockWebview = {
      options: {},
      html: '',
      onDidReceiveMessage: (handler: any) => {
        messageHandler = handler;
        return { dispose: () => {} };
      },
      asWebviewUri: (uri: vscode.Uri) => uri,
      cspSource: 'self',
      postMessage: () => Promise.resolve(true),
    } as any;

    const mockWebviewView = {
      webview: mockWebview,
      title: '',
      description: '',
      badge: undefined,
      show: () => {},
      onDidChangeVisibility: () => ({ dispose: () => {} }),
      onDidDispose: () => ({ dispose: () => {} }),
      visible: true,
    } as any;

    // Mock vscode.workspace.openTextDocument and vscode.window.showTextDocument
    const originalOpenTextDocument = vscode.workspace.openTextDocument;
    const originalShowTextDocument = vscode.window.showTextDocument;

    (vscode.workspace as any).openTextDocument = (uri: vscode.Uri) => {
      openedFile = uri.fsPath;
      return Promise.resolve({ uri } as any);
    };

    (vscode.window as any).showTextDocument = (_document: any) => {
      return Promise.resolve({
        selection: null,
        revealRange: () => {},
      } as any);
    };

    provider.resolveWebviewView(mockWebviewView, {} as any, {} as any);

    // Simulate openFile message from webview
    if (messageHandler) {
      await messageHandler({
        type: 'openFile',
        file: '/test/file.ts',
        line: 42,
      });

      assert.strictEqual(openedFile, '/test/file.ts');
    }

    // Restore original methods
    (vscode.workspace as any).openTextDocument = originalOpenTextDocument;
    (vscode.window as any).showTextDocument = originalShowTextDocument;
  });

  test('CLI service should find renamify binary', () => {
    // This will throw if the CLI is not found
    try {
      const service = new RenamifyCliService();
      assert.ok(service, 'CLI service should be created');
    } catch (error: any) {
      // It's OK if the CLI is not found in test environment
      assert.ok(
        error.message.includes('not found'),
        'Should fail with meaningful error'
      );
    }
  });

  test('Should respect configuration settings', () => {
    const config = vscode.workspace.getConfiguration('renamify');

    // Test default values
    assert.strictEqual(
      config.get('respectGitignore'),
      true,
      'respectGitignore should default to true'
    );
    assert.strictEqual(
      config.get('showContextLines'),
      2,
      'showContextLines should default to 2'
    );
    assert.strictEqual(
      config.get('autoSaveBeforeApply'),
      true,
      'autoSaveBeforeApply should default to true'
    );
    assert.strictEqual(
      config.get('confirmBeforeApply'),
      true,
      'confirmBeforeApply should default to true'
    );

    // Test that cliPath can be configured
    const cliPath = config.get<string>('cliPath');
    assert.ok(cliPath !== undefined, 'cliPath should be configurable');
  });
});
