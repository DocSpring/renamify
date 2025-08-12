import { execa, ExecaError } from 'execa';
import * as path from 'path';
import * as fs from 'fs/promises';

export interface PlanOptions {
  old: string;
  new: string;
  includes?: string[];
  excludes?: string[];
  styles?: string[];
  previewFormat?: 'table' | 'diff' | 'json' | 'summary';
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

export class RefaktorService {
  private refaktorPath: string;

  constructor(refaktorPath?: string) {
    // Default to 'refaktor' which should be in PATH
    // Users can override with full path if needed
    this.refaktorPath = refaktorPath || 'refaktor';
  }

  /**
   * Check if refaktor CLI is available
   */
  async checkAvailability(): Promise<boolean> {
    try {
      await execa(this.refaktorPath, ['--version']);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Create a refactoring plan
   */
  async plan(options: PlanOptions): Promise<string> {
    const args = ['plan', options.old, options.new];

    // Add optional arguments
    if (options.includes && options.includes.length > 0) {
      for (const pattern of options.includes) {
        args.push('--include', pattern);
      }
    }

    if (options.excludes && options.excludes.length > 0) {
      for (const pattern of options.excludes) {
        args.push('--exclude', pattern);
      }
    }

    if (options.styles && options.styles.length > 0) {
      args.push('--styles', options.styles.join(','));
    }

    if (options.previewFormat) {
      args.push('--preview-format', options.previewFormat);
    }

    if (options.dryRun) {
      args.push('--dry-run');
    }

    if (options.renameFiles === false) {
      args.push('--no-rename-files');
    }

    if (options.renameDirs === false) {
      args.push('--no-rename-dirs');
    }

    try {
      const result = await execa(this.refaktorPath, args);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(`Refaktor plan failed: ${execaError.stderr || execaError.message}`);
      }
      throw error;
    }
  }

  /**
   * Apply a refactoring plan
   */
  async apply(options: ApplyOptions): Promise<string> {
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

    try {
      const result = await execa(this.refaktorPath, args);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(`Refaktor apply failed: ${execaError.stderr || execaError.message}`);
      }
      throw error;
    }
  }

  /**
   * Undo a refactoring
   */
  async undo(id: string): Promise<string> {
    try {
      const result = await execa(this.refaktorPath, ['undo', id]);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(`Refaktor undo failed: ${execaError.stderr || execaError.message}`);
      }
      throw error;
    }
  }

  /**
   * Redo a refactoring
   */
  async redo(id: string): Promise<string> {
    try {
      const result = await execa(this.refaktorPath, ['redo', id]);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(`Refaktor redo failed: ${execaError.stderr || execaError.message}`);
      }
      throw error;
    }
  }

  /**
   * Get refactoring history
   */
  async history(limit?: number): Promise<string> {
    const args = ['history'];
    if (limit !== undefined) {
      args.push('--limit', limit.toString());
    }

    try {
      const result = await execa(this.refaktorPath, args);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(`Refaktor history failed: ${execaError.stderr || execaError.message}`);
      }
      throw error;
    }
  }

  /**
   * Get refaktor status
   */
  async status(): Promise<string> {
    try {
      const result = await execa(this.refaktorPath, ['status']);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error && 'stderr' in error) {
        const execaError = error as ExecaError;
        throw new Error(`Refaktor status failed: ${execaError.stderr || execaError.message}`);
      }
      throw error;
    }
  }

  /**
   * Preview a plan without applying it
   */
  async preview(options: PreviewOptions): Promise<string> {
    // Read the plan file and format it according to the requested format
    let planPath: string;

    if (options.planPath) {
      planPath = options.planPath;
    } else if (options.planId) {
      // Construct path from ID (assuming default .refaktor directory)
      planPath = path.join('.refaktor', 'plans', `${options.planId}.json`);
    } else {
      // Use the latest plan
      planPath = path.join('.refaktor', 'plan.json');
    }

    try {
      // Check if plan file exists
      await fs.access(planPath);

      // Use refaktor CLI to preview the plan with the specified format
      const args = ['plan', '--preview-only'];
      
      if (options.planPath || options.planId) {
        args.push('--plan', planPath);
      }

      if (options.format) {
        args.push('--preview-format', options.format);
      }

      const result = await execa(this.refaktorPath, args);
      return result.stdout;
    } catch (error) {
      if (error instanceof Error) {
        if ('code' in error && error.code === 'ENOENT') {
          throw new Error(`Plan file not found: ${planPath}`);
        }
        if ('stderr' in error) {
          const execaError = error as ExecaError;
          throw new Error(`Refaktor preview failed: ${execaError.stderr || execaError.message}`);
        }
      }
      throw error;
    }
  }
}