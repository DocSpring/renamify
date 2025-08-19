import * as vscode from 'vscode';
import { RenamifyCliService } from './cliService';
import { RenamifyCommands } from './commands';
import { RenamifyViewProvider } from './webviewProvider';

export function activate(context: vscode.ExtensionContext) {
  const cliService = new RenamifyCliService();
  const provider = new RenamifyViewProvider(context.extensionUri, cliService);
  const commands = new RenamifyCommands(cliService, provider);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      RenamifyViewProvider.viewType,
      provider,
      {
        webviewOptions: {
          retainContextWhenHidden: true,
        },
      }
    )
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('renamify.search', () => {
      commands.openSearchPanel();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('renamify.plan', () => {
      commands.createPlan();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('renamify.apply', () => {
      commands.applyChanges();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('renamify.undo', () => {
      commands.undoLastOperation();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('renamify.history', () => {
      commands.showHistory();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('renamify.clearResults', () => {
      commands.clearResults();
    })
  );
}

export function deactivate() {}
