import { spawn } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import type { SearchOptions, SearchResult, Status } from './types';

export type { SearchOptions, SearchResult } from './types';

// Regex patterns for extracting plan IDs from text output
const PLAN_ID_PATTERN = /Plan ID:\s*([a-f0-9]+)/i;
const ID_PATTERN = /ID:\s*([a-f0-9]+)/i;

export class RenamifyCliService {
  private readonly cliPath: string;
  private readonly spawnFn: typeof spawn;

  constructor(spawnFn?: typeof spawn) {
    this.cliPath = this.findCliPath();
    this.spawnFn = spawnFn || spawn;
  }

  private findCliPath(): string {
    const config = vscode.workspace.getConfiguration('renamify');
    const configuredPath = config.get<string>('cliPath');

    if (configuredPath && fs.existsSync(configuredPath)) {
      return configuredPath;
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

    // Try local development path
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

    throw new Error(
      'Renamify CLI not found. Please install it or configure the path in settings.'
    );
  }

  async search(searchTerm: string, options: SearchOptions): Promise<Plan> {
    const args = ['search', searchTerm, '--preview', 'json'];

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

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);
    return JSON.parse(result);
  }

  async createPlan(
    searchTerm: string,
    replaceTerm: string,
    options: SearchOptions & { dryRun?: boolean }
  ): Promise<Plan | SearchResult[]> {
    const args = ['plan', searchTerm, replaceTerm, '--preview', 'json'];

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

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);

    // Always return the full Plan object, whether dry-run or not
    return JSON.parse(result);
  }

  async rename(
    searchTerm: string,
    replaceTerm: string,
    options: SearchOptions
  ): Promise<{ planId: string }> {
    const args = ['rename', searchTerm, replaceTerm, '-y'];

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

    const config = vscode.workspace.getConfiguration('renamify');
    if (!config.get('respectGitignore')) {
      args.push('-u');
    }

    const result = await this.runCli(args);

    // Parse the plan ID from the output
    try {
      const planData = JSON.parse(result);
      return { planId: planData.id };
    } catch {
      // If we can't parse JSON, try to extract plan ID from text output
      const planIdMatch =
        result.match(PLAN_ID_PATTERN) || result.match(ID_PATTERN);
      if (planIdMatch) {
        return { planId: planIdMatch[1] };
      }
      // Fallback if no plan ID found
      return { planId: 'unknown' };
    }
  }

  async apply(planId?: string): Promise<void> {
    const args = ['apply'];

    if (planId) {
      args.push('--id', planId);
    }

    await this.runCli(args);
  }

  async undo(id: string): Promise<void> {
    await this.runCli(['undo', id]);
  }

  async redo(id: string): Promise<void> {
    await this.runCli(['redo', id]);
  }

  async history(limit?: number): Promise<HistoryEntry[]> {
    const args = ['history'];

    if (limit) {
      args.push('--limit', limit.toString());
    }

    const result = await this.runCli(args);
    return JSON.parse(result);
  }

  async status(): Promise<Status> {
    const result = await this.runCli(['status']);
    return JSON.parse(result);
  }

  public runCliRaw(args: string[]): Promise<string> {
    return this.runCli(args);
  }

  private runCli(args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
      const workspaceRoot =
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();

      const proc = this.spawnFn(this.cliPath, args, {
        cwd: workspaceRoot,
        env: process.env,
      });

      let stdout = '';
      let stderr = '';

      proc.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      proc.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve(stdout);
        } else {
          const errorMessage = stderr.trim() || `CLI exited with code ${code}`;
          reject(new Error(errorMessage));
        }
      });

      proc.on('error', (err) => {
        reject(err);
      });
    });
  }
}
