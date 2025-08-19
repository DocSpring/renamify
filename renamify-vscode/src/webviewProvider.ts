import * as vscode from 'vscode';
import type { RenamifyCliService } from './cliService';
import type {
  ApplyMessage,
  ExtensionMessage,
  OpenFileMessage,
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
      const results = await this._cliService.search(data.search, data.replace, {
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
      vscode.window.showErrorMessage(
        `Search failed: ${error instanceof Error ? error.message : String(error)}`
      );
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

    if (config.get('confirmBeforeApply')) {
      const answer = await vscode.window.showInformationMessage(
        'Are you sure you want to apply these changes?',
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
      await this._cliService.apply(data.planId);
      vscode.window.showInformationMessage('Changes applied successfully');

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
                        <label for="caseStyles">Case styles</label>
                        <div class="case-styles-container">
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
                        <button id="searchBtn" class="primary">Search</button>
                        <button id="planBtn">Create Plan</button>
                        <button id="applyBtn">Apply</button>
                        <button id="clearBtn">Clear</button>
                    </div>
                </div>
                
                <div class="results-container">
                    <div class="results-header">
                        <span id="resultsSummary"></span>
                        <div class="results-actions">
                            <button id="expandAll" title="Expand All">⊞</button>
                            <button id="collapseAll" title="Collapse All">⊟</button>
                        </div>
                    </div>
                    <div id="resultsTree" class="results-tree"></div>
                </div>
                
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
