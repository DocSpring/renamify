import { access } from 'node:fs/promises';
import { join } from 'node:path';
import { type ExecaError, execa } from 'execa';

export interface PlanOptions {
  old: string;
  new: string;
  includes?: string[];
  excludes?: string[];
  styles?: string[];
  preview?: 'table' | 'diff' | 'json' | 'summary';
  dryRun?: boolean;
  renameFiles?: boolean;
  renameDirs?: boolean;
}

export interface ApplyOptions {
  planId?: string;
  planPath?: string;
  atomic?: boolean;
  commit?: boolean;
}

export interface PreviewOptions {
  planId?: string;
  planPath?: string;
  format?: 'table' | 'diff' | 'json' | 'summary';
}

export class RenamifyService {
  private renamifyPath: string;

  constructor(renamifyPath?: string) {
    // Default to 'renamify' which should be in PATH
    // Users can override with full path if needed
    this.renamifyPath = renamifyPath || 'renamify';
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
   * Create a renaming plan
   */
  async plan(options: PlanOptions): Promise<string> {
    const args = this.buildPlanArgs(options);
    return await this.executeCommand(args, 'plan');
  }

  private buildPlanArgs(options: PlanOptions): string[] {
    const args = ['plan', options.old, options.new];

    this.addIncludeArgs(args, options.includes);
    this.addExcludeArgs(args, options.excludes);
    this.addStylesArg(args, options.styles);
    this.addPreviewArg(args, options.preview);
    this.addDryRunArg(args, options.dryRun);
    this.addRenameArgs(args, options.renameFiles, options.renameDirs);

    return args;
  }

  private addIncludeArgs(args: string[], includes?: string[]): void {
    if (includes && includes.length > 0) {
      for (const pattern of includes) {
        args.push('--include', pattern);
      }
    }
  }

  private addExcludeArgs(args: string[], excludes?: string[]): void {
    if (excludes && excludes.length > 0) {
      for (const pattern of excludes) {
        args.push('--exclude', pattern);
      }
    }
  }

  private addStylesArg(args: string[], styles?: string[]): void {
    if (styles && styles.length > 0) {
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

  private async executeCommand(
    args: string[],
    operation: string
  ): Promise<string> {
    try {
      const result = await execa(this.renamifyPath, args);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(
          `Renamify ${operation} failed: ${execaError.stderr || execaError.message}`
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
  async undo(id: string): Promise<string> {
    return await this.executeCommand(['undo', id], 'undo');
  }

  /**
   * Redo a renaming
   */
  async redo(id: string): Promise<string> {
    return await this.executeCommand(['redo', id], 'redo');
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
}
