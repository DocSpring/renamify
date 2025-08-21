import * as fs from 'node:fs';
import * as path from 'node:path';
import * as Handlebars from 'handlebars';
import * as vscode from 'vscode';
import { RenamifyCliService } from './cliService';
import type {
  ApplyMessage,
  ExtensionMessage,
  OpenFileMessage,
  OpenPreviewMessage,
  PlanMessage,
  SearchMessage,
  SearchResult,
  WebviewMessage,
} from './types';

export class RenamifyViewProvider implements vscode.WebviewViewProvider {
  public static readonly viewType = 'renamify.searchView';
  private _view?: vscode.WebviewView;
  // biome-ignore lint/style/useReadonlyClassProperties: _cliService is reassigned in refresh handler
  private _cliService: RenamifyCliService;
  private _webviewTemplate?: HandlebarsTemplateDelegate;
  private _splashTemplate?: HandlebarsTemplateDelegate;

  constructor(
    private readonly _extensionUri: vscode.Uri,
    cliService: RenamifyCliService
  ) {
    this._cliService = cliService;
    this._loadTemplates();
  }

  private _loadTemplates() {
    const templatesPath = path.join(
      this._extensionUri.fsPath,
      'extension',
      'templates'
    );

    // Load and compile webview template
    const webviewTemplatePath = path.join(templatesPath, 'webview.hbs');
    if (fs.existsSync(webviewTemplatePath)) {
      const webviewTemplateSource = fs.readFileSync(
        webviewTemplatePath,
        'utf-8'
      );
      this._webviewTemplate = Handlebars.compile(webviewTemplateSource);
    }

    // Load and compile splash template
    const splashTemplatePath = path.join(templatesPath, 'splash.hbs');
    if (fs.existsSync(splashTemplatePath)) {
      const splashTemplateSource = fs.readFileSync(splashTemplatePath, 'utf-8');
      this._splashTemplate = Handlebars.compile(splashTemplateSource);
    }
  }

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
      // console.log('Received message:', data);
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
          await this.openFile(
            openFileData.file,
            openFileData.line,
            openFileData.column
          );
          break;
        }
        case 'openPreview': {
          const previewData = data as OpenPreviewMessage;
          await this.openPreviewInEditor(previewData);
          break;
        }
        case 'openSettings':
          await vscode.commands.executeCommand(
            'workbench.action.openSettings',
            '@ext:DocSpring.renamify'
          );
          break;
        case 'refresh':
          // Recreate the CLI service and refresh the webview
          this._cliService = new RenamifyCliService();
          this._loadTemplates(); // Reload templates
          if (this._view) {
            this._view.webview.html = this._getHtmlForWebview(
              this._view.webview
            );
          }
          break;
        default:
          console.warn(
            `Unknown message type: ${(data as { type: string }).type}`
          );
          break;
      }
    });
  }

  private async handleSearch(data: SearchMessage) {
    // Kill any running command before starting a new search
    this._cliService.killCurrentCommand();
    
    try {
      let results: SearchResult[];
      let paths: Rename[] = [];

      if (data.replace) {
        // If replace is provided, use plan with --dry-run to get both matches and paths
        const planResult = await this._cliService.createPlan(
          data.search,
          data.replace,
          {
            include: data.include,
            exclude: data.exclude,
            excludeMatchingLines: data.excludeMatchingLines,
            caseStyles: data.caseStyles,
            renamePaths: data.renamePaths,
            ignoreAmbiguous: data.ignoreAmbiguous,
            dryRun: true,
          }
        );

        // When dryRun is true with replace, we get a Plan object
        const plan = planResult as Plan;

        // Convert plan matches to SearchResult format
        const fileMap = new Map<string, MatchHunk[]>();
        for (const match of plan.matches) {
          if (!fileMap.has(match.file)) {
            fileMap.set(match.file, []);
          }
          fileMap.get(match.file)?.push(match);
        }

        results = Array.from(fileMap.entries()).map(([file, matches]) => ({
          file,
          matches,
        }));

        paths = plan.paths;
      } else {
        // Search-only mode - use search command to get full results including paths
        const plan = await this._cliService.search(data.search, {
          include: data.include,
          exclude: data.exclude,
          excludeMatchingLines: data.excludeMatchingLines,
          caseStyles: data.caseStyles,
          renamePaths: data.renamePaths,
          ignoreAmbiguous: data.ignoreAmbiguous,
        });

        // Convert plan matches to SearchResult format
        const fileMap = new Map<string, MatchHunk[]>();
        for (const match of plan.matches) {
          if (!fileMap.has(match.file)) {
            fileMap.set(match.file, []);
          }
          fileMap.get(match.file)?.push(match);
        }

        results = Array.from(fileMap.entries()).map(([file, matches]) => ({
          file,
          matches,
        }));

        paths = plan.paths;
      }

      this._view?.webview.postMessage({
        type: 'searchResults',
        results,
        paths,
        searchId: (data as any).searchId,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Search failed:', errorMessage);
      
      // Send error to webview to display in results area
      this._view?.webview.postMessage({
        type: 'searchError',
        error: errorMessage,
        searchId: (data as any).searchId,
      });
      
      // Show error notification for lock errors or other critical failures
      if (errorMessage.includes('lock') || errorMessage.includes('Another renamify process')) {
        vscode.window.showErrorMessage(`Search failed: ${errorMessage}`);
      }
    }
  }

  private async handlePlan(data: PlanMessage) {
    // Kill any running command before starting a new plan
    this._cliService.killCurrentCommand();
    
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
        `Plan creation failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private async handleApply(data: ApplyMessage) {
    // Kill any running command before starting apply
    this._cliService.killCurrentCommand();
    
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
        renamePaths: data.renamePaths,
        ignoreAmbiguous: data.ignoreAmbiguous,
      });

      vscode.window.showInformationMessage('Changes applied successfully');

      // Trigger a new search to refresh results
      this._view?.webview.postMessage({
        type: 'changesApplied',
      });
    } catch (error) {
      vscode.window.showErrorMessage(
        `Apply failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private async openFile(filePath: string, line?: number, column?: number) {
    const uri = vscode.Uri.file(filePath);
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);

    if (line !== undefined) {
      const col = column !== undefined ? column : 0; // Column is already 1-based from Rust
      const position = new vscode.Position(line - 1, col);
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
        `Failed to open preview: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  public postMessage(message: ExtensionMessage) {
    this._view?.webview.postMessage(message);
  }

  private _getHtmlForWebview(webview: vscode.Webview) {
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'media', 'bundle.js')
    );

    const styleUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'media', 'style.css')
    );

    const codiconsUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'media', 'codicon.css')
    );

    const nonce = getNonce();

    // Check if CLI is available
    if (!this._cliService.isBinaryAvailable()) {
      return this._getSplashScreenHtml(webview, styleUri, nonce);
    }

    // Template is required
    if (!this._webviewTemplate) {
      throw new Error('Webview template not loaded');
    }

    const workspaceRoot =
      vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '';

    return this._webviewTemplate({
      cspSource: webview.cspSource,
      nonce,
      scriptUri: scriptUri.toString(),
      styleUri: styleUri.toString(),
      codiconsUri: codiconsUri.toString(),
      workspaceRootJson: JSON.stringify(workspaceRoot),
    });
  }

  private _getSplashScreenHtml(
    webview: vscode.Webview,
    styleUri: vscode.Uri,
    nonce: string
  ) {
    // Template is required
    if (!this._splashTemplate) {
      throw new Error('Splash template not loaded');
    }

    return this._splashTemplate({
      cspSource: webview.cspSource,
      nonce,
      styleUri: styleUri.toString(),
    });
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
