import * as vscode from 'vscode';
import type { RenamifyCliService } from './cliService';
import type { Plan } from './types';
import type { RenamifyViewProvider } from './webviewProvider';

export class RenamifyCommands {
  constructor(
    private readonly cliService: RenamifyCliService,
    private readonly viewProvider: RenamifyViewProvider
  ) {}

  async openSearchPanel() {
    await vscode.commands.executeCommand('renamify.searchView.focus');
  }

  async createPlan() {
    const searchTerm = await vscode.window.showInputBox({
      prompt: 'Enter search term',
      placeHolder: 'Search term',
    });

    if (!searchTerm) {
      return;
    }

    const replaceTerm = await vscode.window.showInputBox({
      prompt: 'Enter replacement term',
      placeHolder: 'Replacement term',
    });

    if (replaceTerm === undefined) {
      return;
    }

    try {
      const plan = (await this.cliService.createPlan(searchTerm, replaceTerm, {
        dryRun: false,
      })) as Plan;
      vscode.window.showInformationMessage(
        `Plan created with ${plan.stats?.total_matches || 0} matches`
      );
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to create plan: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  async applyChanges() {
    const config = vscode.workspace.getConfiguration('renamify');

    if (config.get('confirmBeforeApply')) {
      const answer = await vscode.window.showWarningMessage(
        'Are you sure you want to apply the changes?',
        'Yes',
        'No'
      );

      if (answer !== 'Yes') {
        return;
      }
    }

    if (config.get('autoSaveBeforeApply')) {
      await vscode.workspace.saveAll();
    }

    try {
      await this.cliService.apply();
      vscode.window.showInformationMessage('Changes applied successfully');
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to apply changes: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  async undoLastOperation() {
    try {
      const history = await this.cliService.history(1);
      if (history.length === 0) {
        vscode.window.showInformationMessage('No operations to undo');
        return;
      }

      await this.cliService.undo(history[0].id);
      vscode.window.showInformationMessage('Operation undone successfully');
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to undo: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  async showHistory() {
    try {
      const history = await this.cliService.history(10);

      if (history.length === 0) {
        vscode.window.showInformationMessage('No history available');
        return;
      }

      const items = history.map((h) => ({
        label: `${h.id}: ${h.search} → ${h.replace}`,
        description: `${new Date(h.created_at).toLocaleString()} - ${h.stats?.total_matches || 0} matches`,
        id: h.id,
      }));

      const selected = await vscode.window.showQuickPick(items, {
        placeHolder: 'Select an operation to view details',
      });

      if (selected) {
        const answer = await vscode.window.showInformationMessage(
          `Operation ${selected.id}`,
          'Undo',
          'Redo',
          'Cancel'
        );

        if (answer === 'Undo') {
          await this.cliService.undo(selected.id);
          vscode.window.showInformationMessage('Operation undone');
        } else if (answer === 'Redo') {
          await this.cliService.redo(selected.id);
          vscode.window.showInformationMessage('Operation redone');
        }
      }
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to show history: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  clearResults() {
    this.viewProvider.postMessage({ type: 'clearResults' });
  }
}
