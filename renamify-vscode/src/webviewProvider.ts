import * as vscode from 'vscode';
import type { RenamifyCliService } from './cliService';
import type {
  ApplyMessage,
  ExtensionMessage,
  OpenFileMessage,
  OpenPreviewMessage,
  PlanMessage,
  SearchMessage,
  WebviewMessage,
} from './types';

export class RenamifyViewProvider implements vscode.WebviewViewProvider {
  public static readonly viewType = 'renamify.searchView';
  private _view?: vscode.WebviewView;

  constructor(
    private readonly _extensionUri: vscode.Uri,
    private readonly _cliService: RenamifyCliService
  ) {}

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ) {
    this._view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri],
    };

    webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

    webviewView.webview.onDidReceiveMessage(async (data: WebviewMessage) => {
      switch (data.type) {
        case 'search':
          await this.handleSearch(data);
          break;
        case 'plan':
          await this.handlePlan(data);
          break;
        case 'apply':
          await this.handleApply(data);
          break;
        case 'openFile': {
          const openFileData = data as OpenFileMessage;
          await this.openFile(openFileData.file, openFileData.line);
          break;
        }
        case 'openPreview': {
          const previewData = data as OpenPreviewMessage;
          await this.openPreviewInEditor(previewData);
          break;
        }
        default:
          console.warn(
            `Unknown message type: ${(data as { type: string }).type}`
          );
          break;
      }
    });
  }

  private async handleSearch(data: SearchMessage) {
    try {
      // If replace is provided, use plan with --dry-run, otherwise use search
      const results = data.replace
        ? await this._cliService.createPlan(data.search, data.replace, {
            include: data.include,
            exclude: data.exclude,
            excludeMatchingLines: data.excludeMatchingLines,
            caseStyles: data.caseStyles,
            dryRun: true,
          })
        : await this._cliService.search(data.search, '', {
            include: data.include,
            exclude: data.exclude,
            excludeMatchingLines: data.excludeMatchingLines,
            caseStyles: data.caseStyles,
          });

      this._view?.webview.postMessage({
        type: 'searchResults',
        results,
      });
    } catch (error) {
      // Don't show error messages for debounced searches
      console.error('Search failed:', error);
    }
  }

  private async handlePlan(data: PlanMessage) {
    try {
      const plan = await this._cliService.createPlan(
        data.search,
        data.replace,
        {
          include: data.include,
          exclude: data.exclude,
          excludeMatchingLines: data.excludeMatchingLines,
          caseStyles: data.caseStyles,
        }
      );

      this._view?.webview.postMessage({
        type: 'planCreated',
        plan,
      });

      vscode.window.showInformationMessage('Plan created successfully');
    } catch (error) {
      vscode.window.showErrorMessage(
        `Plan creation failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleApply(data: ApplyMessage) {
    const config = vscode.workspace.getConfiguration('renamify');

    // Get current search and replace from the message
    if (!(data.search && data.replace)) {
      vscode.window.showErrorMessage(
        'Both search and replace terms are required to apply changes'
      );
      return;
    }

    if (config.get('confirmBeforeApply')) {
      const answer = await vscode.window.showInformationMessage(
        `Apply rename: ${data.search} → ${data.replace}?`,
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
      // Create and apply the plan directly (without dry-run)
      await this._cliService.rename(data.search, data.replace, {
        include: data.include,
        exclude: data.exclude,
        excludeMatchingLines: data.excludeMatchingLines,
        caseStyles: data.caseStyles,
      });

      vscode.window.showInformationMessage('Changes applied successfully');

      // Trigger a new search to refresh results
      this._view?.webview.postMessage({
        type: 'changesApplied',
      });
    } catch (error) {
      vscode.window.showErrorMessage(
        `Apply failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async openFile(filePath: string, line?: number) {
    const uri = vscode.Uri.file(filePath);
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);

    if (line !== undefined) {
      const position = new vscode.Position(line - 1, 0);
      editor.selection = new vscode.Selection(position, position);
      editor.revealRange(new vscode.Range(position, position));
    }
  }

  private async openPreviewInEditor(data: OpenPreviewMessage) {
    try {
      // Build CLI args for preview
      const args = ['plan', data.search];

      // Use diff preview if replace is provided, otherwise use matches
      const previewFormat = data.replace ? 'diff' : 'matches';
      args.push(data.replace || '""'); // Empty string for search-only
      args.push('--dry-run', '--preview', previewFormat);

      if (data.include) {
        args.push('--include', data.include);
      }

      if (data.exclude) {
        args.push('--exclude', data.exclude);
      }

      if (data.excludeMatchingLines) {
        args.push('--exclude-matching-lines', data.excludeMatchingLines);
      }

      if (data.caseStyles && data.caseStyles.length > 0) {
        args.push('--only-styles', data.caseStyles.join(','));
      }

      const config = vscode.workspace.getConfiguration('renamify');
      if (!config.get('respectGitignore')) {
        args.push('-u');
      }

      // Run CLI to get preview output
      const result = await this._cliService.runCliRaw(args);

      // Create title for the document
      const title = data.replace
        ? `Renamify: ${data.search} → ${data.replace}`
        : `Renamify: Search for "${data.search}"`;

      // Create a new untitled document with the preview content
      const doc = await vscode.workspace.openTextDocument({
        content: `# ${title}\n\n${result}`,
        language: 'diff', // Use diff language for syntax highlighting
      });

      // Show the document in the editor
      await vscode.window.showTextDocument(doc, {
        preview: false,
        viewColumn: vscode.ViewColumn.Active,
      });
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to open preview: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  public postMessage(message: ExtensionMessage) {
    this._view?.webview.postMessage(message);
  }

  private _getHtmlForWebview(webview: vscode.Webview) {
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'media', 'main.js')
    );

    const styleUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'media', 'style.css')
    );

    const nonce = getNonce();

    return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <link href="${styleUri}" rel="stylesheet">
                <title>Renamify Search & Replace</title>
            </head>
            <body>
                <div class="search-container">
                    <div class="input-group">
                        <label for="search">Search</label>
                        <input type="text" id="search" placeholder="Enter search term...">
                    </div>

                    <div class="input-group">
                        <label for="replace">Replace</label>
                        <input type="text" id="replace" placeholder="Enter replacement...">
                    </div>

                    <div class="input-group">
                        <label for="include">Files to include</label>
                        <input type="text" id="include" placeholder="e.g., **/*.ts, src/**/*">
                    </div>

                    <div class="input-group">
                        <label for="exclude">Files to exclude</label>
                        <input type="text" id="exclude" placeholder="e.g., node_modules/**, *.min.js">
                    </div>

                    <div class="input-group">
                        <label for="excludeLines">Exclude matching lines (regex)</label>
                        <input type="text" id="excludeLines" placeholder="e.g., ^\\s*//.*">
                    </div>

                    <div class="input-group">
                        <div class="case-styles-header" id="caseStylesHeader">
                            <span class="expand-icon">▼</span>
                            <label for="caseStyles">Case styles (<span id="checkedCount">8</span>)</label>
                        </div>
                        <div class="case-styles-container" id="caseStylesContainer">
                            <label class="checkbox-label">
                                <input type="checkbox" value="original" checked> Original
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="snake" checked> snake_case
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="kebab" checked> kebab-case
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="camel" checked> camelCase
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="pascal" checked> PascalCase
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="screaming-snake" checked> SCREAMING_SNAKE_CASE
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="train" checked> Train-Case
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="screaming-train" checked> SCREAMING-TRAIN-CASE
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="title"> Title Case
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" value="dot"> dot.case
                            </label>
                        </div>
                    </div>

                    <div class="button-group">
                        <button id="applyBtn" class="primary">Apply Rename</button>
                    </div>
                </div>

                <div class="results-container">
                    <div class="results-header">
                        <span id="resultsSummary"></span>
                        <a href="#" id="openInEditor" class="open-in-editor">Open in editor</a>
                        <div class="results-actions">
                            <button id="expandAll" title="Expand All">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="transform: scaleX(-1);">
                                    <line x1="15" x2="15" y1="12" y2="18"/>
                                    <line x1="12" x2="18" y1="15" y2="15"/>
                                    <rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>
                                    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>
                                </svg>
                            </button>
                            <button id="collapseAll" title="Collapse All">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="transform: scaleX(-1);">
                                    <line x1="12" x2="18" y1="15" y2="15"/>
                                    <rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>
                                    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>
                                </svg>
                            </button>
                        </div>
                    </div>
                    <div id="resultsTree" class="results-tree"></div>
                </div>

                <script nonce="${nonce}">
                    window.workspaceRoot = ${JSON.stringify(vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '')};
                </script>
                <script nonce="${nonce}" src="${scriptUri}"></script>
            </body>
            </html>`;
  }
}

function getNonce() {
  let text = '';
  const possible =
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}
