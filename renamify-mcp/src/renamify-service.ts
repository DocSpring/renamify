import { readFileSync } from 'node:fs';
import { access } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { type ExecaError, execa } from 'execa';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const packageJson = JSON.parse(
  readFileSync(resolve(__dirname, '..', 'package.json'), 'utf-8')
);

type VersionInfo = {
  name: string;
  version: string;
};

export type SearchOptions = {
  search: string;
  paths?: string[];
  includes?: string[];
  excludes?: string[];
  styles?: string[];
  preview?: 'table' | 'matches' | 'summary';
  dryRun?: boolean;
  renameFiles?: boolean;
  renameDirs?: boolean;
  ignoreAmbiguous?: boolean;
};

export type PlanOptions = {
  search: string;
  replace: string;
  paths?: string[];
  includes?: string[];
  excludes?: string[];
  styles?: string[];
  preview?: 'table' | 'diff' | 'summary';
  dryRun?: boolean;
  renameFiles?: boolean;
  renameDirs?: boolean;
  ignoreAmbiguous?: boolean;
};

export type ApplyOptions = {
  planId?: string;
  planPath?: string;
  atomic?: boolean;
  commit?: boolean;
};

export type PreviewOptions = {
  planId?: string;
  planPath?: string;
  format?: 'table' | 'diff' | 'summary';
};

export class RenamifyService {
  private readonly renamifyPath: string;
  private readonly mcpVersion: string;

  constructor(renamifyPath?: string) {
    // Default to 'renamify' which should be in PATH
    // Users can override with full path if needed
    this.renamifyPath = renamifyPath || 'renamify';
    this.mcpVersion = packageJson.version;
  }

  /**
   * Check if renamify CLI is available
   */
  async checkAvailability(): Promise<boolean> {
    try {
      await execa(this.renamifyPath, ['--version']);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get CLI version information
   */
  private async getCliVersion(): Promise<VersionInfo> {
    try {
      const result = await execa(this.renamifyPath, [
        'version',
        '--output',
        'json',
      ]);
      return JSON.parse(result.stdout) as VersionInfo;
    } catch (error) {
      if (
        error instanceof Error &&
        'code' in error &&
        error.code === 'ENOENT'
      ) {
        throw new Error(
          'Renamify CLI not found. Please install it using:\n\n' +
            'curl -fsSL https://docspring.github.io/renamify/install.sh | bash\n\n' +
            'For more installation options, visit: https://docspring.github.io/renamify/installation/'
        );
      }
      throw new Error(`Failed to get CLI version: ${error}`);
    }
  }

  /**
   * Check version compatibility between MCP and CLI
   */
  private async checkVersionCompatibility(): Promise<void> {
    const cliInfo = await this.getCliVersion();
    const cliVersion = cliInfo.version;

    // Parse versions
    const [mcpMajor, mcpMinor] = this.mcpVersion.split('.').map(Number);
    const [cliMajor, cliMinor] = cliVersion.split('.').map(Number);

    // Check major version must match
    if (mcpMajor !== cliMajor) {
      throw new Error(
        `Version mismatch: MCP server v${this.mcpVersion} is not compatible with CLI v${cliVersion}.\n` +
          `Major versions must match (MCP major: ${mcpMajor}, CLI major: ${cliMajor}).\n` +
          `Please update the ${mcpMajor < cliMajor ? 'MCP server' : 'CLI'} to a compatible version.`
      );
    }

    // Check minor version: MCP minor must be <= CLI minor
    if (mcpMinor > cliMinor) {
      throw new Error(
        `Version mismatch: MCP server v${this.mcpVersion} requires CLI v${mcpMajor}.${mcpMinor}.x or later.\n` +
          `Current CLI version is v${cliVersion}.\n` +
          'Please update the CLI to a compatible version.'
      );
    }
  }

  /**
   * Create a renaming plan
   */
  async plan(options: PlanOptions): Promise<string> {
    const args = this.buildPlanArgs(options);
    return await this.executeCommand(args, 'plan');
  }

  /**
   * Search for identifiers without replacement
   */
  async search(options: SearchOptions): Promise<string> {
    const args = this.buildSearchArgs(options);
    return await this.executeCommand(args, 'search');
  }

  private buildSearchArgs(options: SearchOptions): string[] {
    const args = ['search', options.search];

    // Add paths after search term
    if (options.paths?.length) {
      args.push(...options.paths);
    }

    this.addIncludeArgs(args, options.includes);
    this.addExcludeArgs(args, options.excludes);
    this.addStylesArg(args, options.styles);
    this.addPreviewArg(args, options.preview);
    this.addDryRunArg(args, options.dryRun);
    this.addRenameArgs(args, options.renameFiles, options.renameDirs);
    this.addIgnoreAmbiguousArg(args, options.ignoreAmbiguous);

    return args;
  }

  private buildPlanArgs(options: PlanOptions): string[] {
    const args = ['plan', options.search, options.replace];

    // Add paths after old and new
    if (options.paths?.length) {
      args.push(...options.paths);
    }

    this.addIncludeArgs(args, options.includes);
    this.addExcludeArgs(args, options.excludes);
    this.addStylesArg(args, options.styles);
    this.addPreviewArg(args, options.preview);
    this.addDryRunArg(args, options.dryRun);
    this.addRenameArgs(args, options.renameFiles, options.renameDirs);
    this.addIgnoreAmbiguousArg(args, options.ignoreAmbiguous);

    return args;
  }

  private addIncludeArgs(args: string[], includes?: string[]): void {
    if (includes?.length) {
      for (const pattern of includes) {
        args.push('--include', pattern);
      }
    }
  }

  private addExcludeArgs(args: string[], excludes?: string[]): void {
    if (excludes?.length) {
      for (const pattern of excludes) {
        args.push('--exclude', pattern);
      }
    }
  }

  private addStylesArg(args: string[], styles?: string[]): void {
    if (styles?.length) {
      args.push('--styles', styles.join(','));
    }
  }

  private addPreviewArg(args: string[], preview?: string): void {
    if (preview) {
      args.push('--preview', preview);
    }
  }

  private addDryRunArg(args: string[], dryRun?: boolean): void {
    if (dryRun) {
      args.push('--dry-run');
    }
  }

  private addRenameArgs(
    args: string[],
    renameFiles?: boolean,
    renameDirs?: boolean
  ): void {
    if (renameFiles === false) {
      args.push('--no-rename-files');
    }
    if (renameDirs === false) {
      args.push('--no-rename-dirs');
    }
  }

  private addIgnoreAmbiguousArg(
    args: string[],
    ignoreAmbiguous?: boolean
  ): void {
    if (ignoreAmbiguous) {
      args.push('--ignore-ambiguous');
    }
  }

  private async executeCommand(
    args: string[],
    operation: string
  ): Promise<string> {
    // Check version compatibility before every command
    // Skip in test environment where execa is mocked
    if (process.env.NODE_ENV !== 'test') {
      await this.checkVersionCompatibility();
    }

    try {
      const result = await execa(this.renamifyPath, args);
      return result.stdout;
    } catch (error) {
      // Check if the binary is missing (ENOENT error)
      if (
        error instanceof Error &&
        'code' in error &&
        error.code === 'ENOENT'
      ) {
        throw new Error(
          'Renamify CLI not found. Please install it using:\n\n' +
            'curl -fsSL https://docspring.github.io/renamify/install.sh | bash\n\n' +
            'For more installation options, visit: https://docspring.github.io/renamify/installation/'
        );
      }

      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(
          `Renamify ${operation} failed: ${
            execaError.stderr || execaError.message
          }`
        );
      }
      throw error;
    }
  }

  /**
   * Apply a renaming plan
   */
  async apply(options: ApplyOptions): Promise<string> {
    const args = this.buildApplyArgs(options);
    return await this.executeCommand(args, 'apply');
  }

  private buildApplyArgs(options: ApplyOptions): string[] {
    const args = ['apply'];

    if (options.planPath) {
      args.push('--plan', options.planPath);
    } else if (options.planId) {
      args.push('--id', options.planId);
    }

    if (options.atomic === false) {
      args.push('--no-atomic');
    }

    if (options.commit) {
      args.push('--commit');
    }

    return args;
  }

  /**
   * Undo a renaming
   */
  async undo(id?: string): Promise<string> {
    const args = ['undo', id || 'latest'];
    return await this.executeCommand(args, 'undo');
  }

  /**
   * Redo a renaming
   */
  async redo(id?: string): Promise<string> {
    const args = ['redo', id || 'latest'];
    return await this.executeCommand(args, 'redo');
  }

  /**
   * Get renaming history
   */
  async history(limit?: number): Promise<string> {
    const args = ['history'];
    if (limit !== undefined) {
      args.push('--limit', limit.toString());
    }
    return await this.executeCommand(args, 'history');
  }

  /**
   * Get renamify status
   */
  async status(): Promise<string> {
    return await this.executeCommand(['status'], 'status');
  }

  /**
   * Preview a plan without applying it
   */
  async preview(options: PreviewOptions): Promise<string> {
    const planPath = this.resolvePlanPath(options);
    return await this.executePreview(planPath, options);
  }

  private resolvePlanPath(options: PreviewOptions): string {
    if (options.planPath) {
      return options.planPath;
    }
    if (options.planId) {
      return join('.renamify', 'plans', `${options.planId}.json`);
    }
    return join('.renamify', 'plan.json');
  }

  private async executePreview(
    planPath: string,
    options: PreviewOptions
  ): Promise<string> {
    // Check version compatibility before every command
    // Skip in test environment where execa is mocked
    if (process.env.NODE_ENV !== 'test') {
      await this.checkVersionCompatibility();
    }

    try {
      await access(planPath);
      const args = this.buildPreviewArgs(planPath, options);
      const result = await execa(this.renamifyPath, args);
      return result.stdout;
    } catch (error) {
      return this.handlePreviewError(error, planPath);
    }
  }

  private buildPreviewArgs(
    planPath: string,
    options: PreviewOptions
  ): string[] {
    const args = ['plan', '--preview-only'];

    if (options.planPath || options.planId) {
      args.push('--plan', planPath);
    }

    if (options.format) {
      args.push('--preview', options.format);
    }

    return args;
  }

  private handlePreviewError(error: unknown, planPath: string): never {
    if (error instanceof Error) {
      if ('code' in error && error.code === 'ENOENT') {
        // Check if this is a file access error (accessing plan file) vs binary not found
        if ('command' in error && typeof error.command === 'string') {
          // This is an execa error - binary not found
          throw new Error(
            'Renamify CLI not found. Please install it using:\n\n' +
              'curl -fsSL https://docspring.github.io/renamify/install.sh | bash\n\n' +
              'For more installation options, visit: https://docspring.github.io/renamify/installation/'
          );
        }
        // This is a file access error - plan file not found
        throw new Error(`Plan file not found: ${planPath}`);
      }
      if ('stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(
          `Renamify preview failed: ${execaError.stderr || execaError.message}`
        );
      }
    }
    throw error;
  }

  /**
   * Rename with case-aware transformations (plan + apply in one step)
   */
  async rename(options: {
    old: string;
    new: string;
    paths?: string[];
    dryRun?: boolean;
    preview?: 'table' | 'diff' | 'summary' | 'json';
    excludeStyles?: string[];
    includeStyles?: string[];
    onlyStyles?: string[];
  }): Promise<string> {
    const args = ['rename', options.old, options.new];

    // Add paths after old and new
    if (options.paths?.length) {
      args.push(...options.paths);
    }

    // Add preview format
    if (options.preview) {
      args.push('--preview', options.preview);
    }

    // Add style options
    if (options.excludeStyles?.length) {
      args.push('--exclude-styles', options.excludeStyles.join(','));
    }
    if (options.includeStyles?.length) {
      args.push('--include-styles', options.includeStyles.join(','));
    }
    if (options.onlyStyles?.length) {
      args.push('--only-styles', options.onlyStyles.join(','));
    }

    // Add dry-run flag
    if (options.dryRun) {
      args.push('--dry-run');
    }

    // Always skip confirmation for MCP usage
    args.push('--yes');

    return await this.executeCommand(args, 'rename');
  }

  /**
   * Simple regex or literal replacement without case transformations
   */
  async replace(options: {
    pattern: string;
    replacement: string;
    paths?: string[];
    noRegex?: boolean;
    preview?: 'table' | 'diff' | 'summary' | 'json';
    dryRun?: boolean;
  }): Promise<string> {
    const args = ['replace', options.pattern, options.replacement];

    // Add paths after pattern and replacement
    if (options.paths?.length) {
      args.push(...options.paths);
    }

    // Add no-regex flag
    if (options.noRegex) {
      args.push('--no-regex');
    }

    // Add preview format
    if (options.preview) {
      args.push('--preview', options.preview);
    }

    // Add dry-run flag
    if (options.dryRun) {
      args.push('--dry-run');
    }

    // Always skip confirmation for MCP usage
    args.push('--yes');

    return await this.executeCommand(args, 'replace');
  }
}
