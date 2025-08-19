import { spawn } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import type {
  HistoryEntry,
  Match,
  Plan,
  SearchOptions,
  SearchResult,
  Status,
} from './types';

export type { Match, SearchOptions, SearchResult } from './types';

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

  async search(
    searchTerm: string,
    replaceTerm: string,
    options: SearchOptions
  ): Promise<SearchResult[]> {
    const args = [
      'plan',
      searchTerm,
      replaceTerm,
      '--dry-run',
      '--preview',
      'json',
    ];

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
    return this.parseSearchResults(result);
  }

  async createPlan(
    searchTerm: string,
    replaceTerm: string,
    options: SearchOptions
  ): Promise<Plan> {
    const args = ['plan', searchTerm, replaceTerm, '--preview', 'json'];

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

  private parseSearchResults(jsonStr: string): SearchResult[] {
    try {
      const data = JSON.parse(jsonStr);

      if (!data.matches) {
        return [];
      }

      const results: SearchResult[] = [];
      const fileMap = new Map<string, Match[]>();

      for (const match of data.matches) {
        const file = match.file;
        if (!fileMap.has(file)) {
          fileMap.set(file, []);
        }

        fileMap.get(file)?.push({
          line: match.line,
          column: match.col || match.column,
          text: match.line_before || match.text,
          replacement: match.line_after || match.replacement,
          context: match.context,
        });
      }

      for (const [file, matches] of fileMap) {
        results.push({ file, matches });
      }

      return results;
    } catch (_error) {
      return [];
    }
  }
}
