import { type ChildProcess, spawn } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import type { SearchOptions, SearchResult, Status } from './types';

export type { SearchOptions, SearchResult } from './types';

type VersionInfo = {
  name: string;
  version: string;
};

class CommandMutex {
  private currentProcess: ChildProcess | null = null;

  acquire(): void {
    // Kill any existing process immediately
    this.killCurrentProcess();
    // No waiting, no queuing - just proceed
  }

  release(): void {
    this.currentProcess = null;
  }

  setCurrentProcess(process: ChildProcess): void {
    this.currentProcess = process;
  }

  killCurrentProcess(): void {
    if (this.currentProcess) {
      this.currentProcess.kill('SIGKILL');
      this.currentProcess = null;
    }
  }
}

export class RenamifyCliService {
  private readonly cliPath?: string;
  private readonly spawnFn: typeof spawn;
  private readonly isAvailable: boolean;
  private readonly extensionVersion: string;
  private readonly commandMutex = new CommandMutex();

  constructor(spawnFn?: typeof spawn) {
    this.spawnFn = spawnFn || spawn;

    // Read version from package.json
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    this.extensionVersion = packageJson.version;

    // If we're using a mock spawn function, assume the CLI is available for testing
    if (spawnFn) {
      this.cliPath = 'mocked-cli-path';
      this.isAvailable = true;
    } else {
      // Only do real file system checks when not mocking
      try {
        this.cliPath = this.findCliPath();
        this.isAvailable = true;
      } catch {
        this.cliPath = undefined;
        this.isAvailable = false;
      }
    }
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

    // Try to find in PATH
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

    // Try local development path relative to workspace
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspaceRoot) {
      const devPath = path.join(workspaceRoot, 'target', 'debug', 'renamify');
      if (fs.existsSync(devPath)) {
        return devPath;
      }
      const devPathExe = path.join(
        workspaceRoot,
        'target',
        'debug',
        'renamify.exe'
      );
      if (fs.existsSync(devPathExe)) {
        return devPathExe;
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
   * Get CLI version information
   */
  private async getCliVersion(): Promise<VersionInfo> {
    const result = await this.runCliRaw(['version', '--output', 'json']);
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
      const message = `Version mismatch: VS Code extension v${this.extensionVersion} is not compatible with CLI v${cliVersion}.\nMajor versions must match (Extension major: ${extMajor}, CLI major: ${cliMajor}).`;
      vscode.window.showErrorMessage(message);
      throw new Error(message);
    }

    // Check minor version: Extension minor must be <= CLI minor
    if (extMinor > cliMinor) {
      const message = `Version mismatch: VS Code extension v${this.extensionVersion} requires CLI v${extMajor}.${extMinor}.x or later.\nCurrent CLI version is v${cliVersion}.`;
      vscode.window.showErrorMessage(message);
      throw new Error(message);
    }
  }

  async search(searchTerm: string, options: SearchOptions): Promise<Plan> {
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

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);
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
  ): Promise<Plan | SearchResult[]> {
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

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);
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
  ): Promise<{ planId: string }> {
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

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);

    // Parse the JSON response
    const parsed = JSON.parse(result);
    // Extract the plan ID from the wrapper or plan
    const planId = parsed.plan_id || parsed.plan?.id;
    if (!planId) {
      throw new Error('Invalid response from CLI: missing plan ID');
    }
    return { planId };
  }

  async apply(planId?: string): Promise<void> {
    const args = ['apply', '--output', 'json'];

    if (planId) {
      args.push('--id', planId);
    }

    await this.runCli(args);
  }

  async undo(id: string): Promise<void> {
    await this.runCli(['undo', id, '--output', 'json']);
  }

  async redo(id: string): Promise<void> {
    await this.runCli(['redo', id, '--output', 'json']);
  }

  async history(limit?: number): Promise<HistoryEntry[]> {
    const args = ['history', '--output', 'json'];

    if (limit) {
      args.push('--limit', limit.toString());
    }

    const result = await this.runCli(args);
    return JSON.parse(result);
  }

  async status(): Promise<Status> {
    const result = await this.runCli(['status', '--output', 'json']);
    return JSON.parse(result);
  }

  public runCliRaw(args: string[]): Promise<string> {
    return this.runCli(args);
  }

  public isBinaryAvailable(): boolean {
    return this.isAvailable;
  }

  public getBinaryPath(): string | undefined {
    return this.cliPath;
  }

  public killCurrentCommand(): void {
    this.commandMutex.killCurrentProcess();
  }

  private async runCli(args: string[]): Promise<string> {
    if (!(this.isAvailable && this.cliPath)) {
      throw new Error(
        'Renamify CLI not found. Please install it or configure the path in settings.'
      );
    }

    // Acquire mutex to ensure only one command runs at a time
    this.commandMutex.acquire();

    try {
      // Check version compatibility before every command (except for version command itself)
      // Skip version check when using mock spawn (in tests)
      if (!args.includes('version') && this.cliPath !== 'mocked-cli-path') {
        await this.checkVersionCompatibility();
      }

      // Try the command, with one retry after 100ms if it fails
      try {
        return await this.executeCliCommand(args);
      } catch (_error) {
        // Wait 100ms and retry once
        await new Promise((resolve) => setTimeout(resolve, 100));
        return await this.executeCliCommand(args);
      }
    } finally {
      this.commandMutex.release();
    }
  }

  private executeCliCommand(args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
      const workspaceRoot =
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();

      // Log the full command for debugging
      console.log(`[Renamify] Executing: ${this.cliPath} ${args.join(' ')}`);

      const proc = this.spawnFn(this.cliPath as string, args, {
        cwd: workspaceRoot,
        env: process.env,
      });

      // Track this process so it can be killed if needed
      this.commandMutex.setCurrentProcess(proc);

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
        } else if (code === null) {
          // Process was killed
          reject(new Error('Command was cancelled'));
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
}
