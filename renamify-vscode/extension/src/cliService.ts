import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { ProcessCoordinator } from './processCoordinator';
import type { SearchOptions, SearchResult, Status } from './types';

export type { SearchOptions, SearchResult } from './types';

type VersionInfo = {
  name: string;
  version: string;
};

export class RenamifyCliService {
  private readonly extensionVersion: string;
  private readonly processCoordinator: ProcessCoordinator;

  constructor(processCoordinator?: ProcessCoordinator) {
    console.log(
      `[${new Date().toISOString()}] [CliService] CONSTRUCTOR CALLED - creating new CliService instance`
    );

    this.processCoordinator = processCoordinator ?? new ProcessCoordinator();

    console.log(
      `[${new Date().toISOString()}] [CliService] CliService constructor completed - ProcessCoordinator instance created`
    );

    // Read version from package.json
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    this.extensionVersion = packageJson.version;
  }

  private get cliPath(): string | null {
    // Always find the CLI path fresh - don't cache it
    try {
      return this.findCliPath();
    } catch {
      return null;
    }
  }

  private get isAvailable(): boolean {
    return !!this.cliPath;
  }

  private findCliPath(): string {
    const config = vscode.workspace.getConfiguration('renamify');
    const configuredPath = config.get<string>('cliPath');

    if (configuredPath) {
      // If it's a relative path, resolve it relative to the workspace root
      const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      let resolvedPath = configuredPath;

      if (!path.isAbsolute(configuredPath) && workspaceRoot) {
        resolvedPath = path.resolve(workspaceRoot, configuredPath);
      }

      if (fs.existsSync(resolvedPath)) {
        return resolvedPath;
      }
    }

    // FIRST: Try local development path relative to workspace (for development)
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspaceRoot) {
      const devPath = path.join(workspaceRoot, 'target', 'debug', 'renamify');
      if (fs.existsSync(devPath)) {
        console.log(`[Renamify] Using local development binary: ${devPath}`);
        return devPath;
      }
      const devPathExe = path.join(
        workspaceRoot,
        'target',
        'debug',
        'renamify.exe'
      );
      if (fs.existsSync(devPathExe)) {
        console.log(`[Renamify] Using local development binary: ${devPathExe}`);
        return devPathExe;
      }
    }

    // THEN: Try to find in PATH
    const pathEnv = process.env.PATH || '';
    const paths = pathEnv.split(path.delimiter);

    for (const p of paths) {
      const cliPath = path.join(p, 'renamify');
      if (fs.existsSync(cliPath)) {
        return cliPath;
      }
      const cliPathExe = path.join(p, 'renamify.exe');
      if (fs.existsSync(cliPathExe)) {
        return cliPathExe;
      }
    }

    // Try local development path relative to the extension directory (for tests)
    const extensionDevPath = path.join(
      __dirname,
      '..',
      '..',
      'target',
      'debug',
      'renamify'
    );
    if (fs.existsSync(extensionDevPath)) {
      return extensionDevPath;
    }
    const extensionDevPathExe = path.join(
      __dirname,
      '..',
      '..',
      'target',
      'debug',
      'renamify.exe'
    );
    if (fs.existsSync(extensionDevPathExe)) {
      return extensionDevPathExe;
    }

    throw new Error(
      'Renamify CLI not found. Please install it or configure the path in settings.'
    );
  }

  /**
   * Get CLI version information (bypasses ProcessCoordinator since it doesn't need locking)
   */
  private async getCliVersion(): Promise<VersionInfo> {
    const result = await this.runCliDirect(['version', '--output', 'json']);
    return JSON.parse(result) as VersionInfo;
  }

  /**
   * Check version compatibility between extension and CLI
   */
  private async checkVersionCompatibility(): Promise<void> {
    const cliInfo = await this.getCliVersion();
    const cliVersion = cliInfo.version;

    // Parse versions
    const [extMajor, extMinor] = this.extensionVersion.split('.').map(Number);
    const [cliMajor, cliMinor] = cliVersion.split('.').map(Number);

    // Check major version must match
    if (extMajor !== cliMajor) {
      const message = `Renamify version mismatch: VS Code extension v${this.extensionVersion} is not compatible with CLI v${cliVersion}.\nMajor versions must match (Extension major: ${extMajor}, CLI major: ${cliMajor}).\n\nPlease update your Renamify CLI: [Installation Guide](https://docspring.github.io/renamify/installation/)`;
      const markdown = new vscode.MarkdownString(message);
      markdown.isTrusted = true;
      vscode.window
        .showErrorMessage(message, 'Open Installation Guide')
        .then((selection) => {
          if (selection === 'Open Installation Guide') {
            vscode.env.openExternal(
              vscode.Uri.parse(
                'https://docspring.github.io/renamify/installation/'
              )
            );
          }
        });
      throw new Error(message);
    }

    // Check minor version: Extension minor must be <= CLI minor
    if (extMinor > cliMinor) {
      const message = `Renamify version mismatch: VS Code extension v${this.extensionVersion} requires CLI v${extMajor}.${extMinor}.x or later. Current CLI version is v${cliVersion}.\n\nPlease update your Renamify CLI: [Installation Guide](https://docspring.github.io/renamify/installation/)`;
      const markdown = new vscode.MarkdownString(message);
      markdown.isTrusted = true;
      vscode.window
        .showErrorMessage(message, 'Open Installation Guide')
        .then((selection) => {
          if (selection === 'Open Installation Guide') {
            vscode.env.openExternal(
              vscode.Uri.parse(
                'https://docspring.github.io/renamify/installation/'
              )
            );
          }
        });
      throw new Error(message);
    }
  }

  async search(
    searchTerm: string,
    options: SearchOptions
  ): Promise<Plan | null> {
    const args = ['search', searchTerm, '--output', 'json'];

    if (options.include) {
      args.push('--include', options.include);
    }

    if (options.exclude) {
      args.push('--exclude', options.exclude);
    }

    if (options.excludeMatchingLines) {
      args.push('--exclude-matching-lines', options.excludeMatchingLines);
    }

    if (options.caseStyles && options.caseStyles.length > 0) {
      args.push('--only-styles', options.caseStyles.join(','));
    }

    if (options.ignoreAmbiguous) {
      args.push('--ignore-ambiguous');
    }

    if (options.renamePaths === false) {
      args.push('--no-rename-paths');
    }

    if (options.atomicSearch) {
      args.push('--atomic-search');
    }

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);
    if (!result) {
      return null; // Cancelled request
    }

    const parsed = JSON.parse(result);
    // The CLI returns a wrapper object with the plan nested inside
    if (!parsed.plan) {
      throw new Error('Invalid response from CLI: missing plan data');
    }
    return parsed.plan;
  }

  async createPlan(
    searchTerm: string,
    replaceTerm: string,
    options: SearchOptions & { dryRun?: boolean }
  ): Promise<Plan | SearchResult[] | null> {
    const args = ['plan', searchTerm, replaceTerm, '--output', 'json'];

    if (options.dryRun) {
      args.push('--dry-run');
    }

    if (options.include) {
      args.push('--include', options.include);
    }

    if (options.exclude) {
      args.push('--exclude', options.exclude);
    }

    if (options.excludeMatchingLines) {
      args.push('--exclude-matching-lines', options.excludeMatchingLines);
    }

    if (options.caseStyles && options.caseStyles.length > 0) {
      args.push('--only-styles', options.caseStyles.join(','));
    }

    if (options.ignoreAmbiguous) {
      args.push('--ignore-ambiguous');
    }

    if (options.renamePaths === false) {
      args.push('--no-rename-paths');
    }

    if (options.atomicSearch) {
      args.push('--atomic-search');
    }

    if (options.atomicReplace) {
      args.push('--atomic-replace');
    }

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);
    if (!result) {
      return null; // Cancelled request
    }

    const parsed = JSON.parse(result);
    // The CLI returns a wrapper object with the plan nested inside
    if (!parsed.plan) {
      throw new Error('Invalid response from CLI: missing plan data');
    }
    return parsed.plan;
  }

  async rename(
    searchTerm: string,
    replaceTerm: string,
    options: SearchOptions
  ): Promise<{ planId: string } | null> {
    const args = ['rename', searchTerm, replaceTerm, '-y', '--output', 'json'];

    if (options.include) {
      args.push('--include', options.include);
    }

    if (options.exclude) {
      args.push('--exclude', options.exclude);
    }

    if (options.excludeMatchingLines) {
      args.push('--exclude-matching-lines', options.excludeMatchingLines);
    }

    if (options.caseStyles && options.caseStyles.length > 0) {
      args.push('--only-styles', options.caseStyles.join(','));
    }

    if (options.ignoreAmbiguous) {
      args.push('--ignore-ambiguous');
    }

    if (options.renamePaths === false) {
      args.push('--no-rename-paths');
    }

    if (options.atomicSearch) {
      args.push('--atomic-search');
    }

    if (options.atomicReplace) {
      args.push('--atomic-replace');
    }

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);
    if (!result) {
      return null; // Cancelled request
    }

    // Parse the JSON response
    const parsed = JSON.parse(result);
    // Extract the plan ID from the wrapper or plan
    const planId = parsed.plan_id || parsed.plan?.id;
    if (!planId) {
      throw new Error('Invalid response from CLI: missing plan ID');
    }
    return { planId };
  }

  async apply(planId?: string): Promise<undefined | null> {
    const args = ['apply', '--output', 'json'];

    if (planId) {
      args.push('--id', planId);
    }

    const result = await this.runCli(args);
    if (!result) {
      return null; // Cancelled request
    }
  }

  async undo(id: string): Promise<undefined | null> {
    const result = await this.runCli(['undo', id, '--output', 'json']);
    if (!result) {
      return null; // Cancelled request
    }
  }

  async redo(id: string): Promise<undefined | null> {
    const result = await this.runCli(['redo', id, '--output', 'json']);
    if (!result) {
      return null; // Cancelled request
    }
  }

  async history(limit?: number): Promise<HistoryEntry[] | null> {
    const args = ['history', '--output', 'json'];

    if (limit) {
      args.push('--limit', limit.toString());
    }

    const result = await this.runCli(args);
    if (!result) {
      return null; // Cancelled request
    }
    return JSON.parse(result);
  }

  async status(): Promise<Status | null> {
    const result = await this.runCli(['status', '--output', 'json']);
    if (!result) {
      return null; // Cancelled request
    }
    return JSON.parse(result);
  }

  public runCliRaw(args: string[]): Promise<string | null> {
    // runCliRaw is used by openPreviewInEditor which needs the ProcessCoordinator
    return this.runCli(args);
  }

  public isBinaryAvailable(): boolean {
    return this.isAvailable;
  }

  public getBinaryPath(): string | undefined {
    return this.cliPath ?? undefined;
  }

  public async killCurrentCommand(): Promise<void> {
    await this.processCoordinator.killCurrentCommand();
  }

  /**
   * Run CLI directly without ProcessCoordinator (for version checks and other non-locking operations)
   */
  private runCliDirect(args: string[]): Promise<string> {
    if (!(this.isAvailable && this.cliPath)) {
      throw new Error(
        'Renamify CLI not found. Please install it or configure the path in settings.'
      );
    }

    return new Promise((resolve, reject) => {
      const { spawn } = require('node:child_process');
      const workspaceRoot =
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();

      const proc = spawn(this.cliPath, args, {
        cwd: workspaceRoot,
        env: process.env,
      });

      let stdout = '';
      let stderr = '';

      proc.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      proc.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      proc.on('close', (code: number | null) => {
        if (code === 0) {
          resolve(stdout);
        } else {
          const errorMessage = stderr.trim() || `CLI exited with code ${code}`;
          reject(new Error(errorMessage));
        }
      });

      proc.on('error', (err: Error) => {
        reject(err);
      });
    });
  }

  private async runCli(args: string[]): Promise<string | null> {
    if (!(this.isAvailable && this.cliPath)) {
      throw new Error(
        'Renamify CLI not found. Please install it or configure the path in settings.'
      );
    }

    // Check version compatibility before every command (except for version command itself)
    if (!args.includes('version')) {
      await this.checkVersionCompatibility();
    }

    // Use ProcessCoordinator to execute the command with proper serialization
    const result = await this.processCoordinator.execute(this.cliPath, args);
    return result;
  }
}
