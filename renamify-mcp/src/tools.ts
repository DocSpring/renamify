import type {
  ApplyOptions,
  PlanOptions,
  PreviewOptions,
  RenamifyService,
  SearchOptions,
} from './renamify-service.js';

export class RenamifyTools {
  private readonly renamifyService: RenamifyService;

  constructor(renamifyService: RenamifyService) {
    this.renamifyService = renamifyService;
  }

  /**
   * Search for identifiers
   */
  async search(params: SearchOptions): Promise<string> {
    // Check if renamify is available
    const isAvailable = await this.renamifyService.checkAvailability();
    if (!isAvailable) {
      return `Error: Renamify CLI is not available. Please ensure 'renamify' is installed and in your PATH.

To install renamify:
1. Download the latest release from GitHub
2. Add the binary to your PATH
3. Or build from source using Rust

For more information, visit: https://github.com/docspring/renamify`;
    }

    try {
      const result = await this.renamifyService.search(params);

      // Add helpful context for AI agents
      const helpText = `
${result}

SEARCH RESULTS:
- Found all occurrences of "${params.search}" in various case styles
- Files and directories that would be renamed are listed above

NEXT STEPS:
1. Review the search results to see where the identifier appears
2. Use 'renamify_plan' with a replacement to create a renaming plan
3. Or refine your search with different paths, includes, or excludes

SEARCH OPTIONS:
- Use 'includes' to focus on specific file patterns
- Use 'excludes' to skip certain files or directories
- Use 'styles' to search for specific case variants only`;

      return params.dryRun ? result : helpText;
    } catch (error) {
      return `Error searching: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Create a renaming plan
   */
  async plan(params: PlanOptions): Promise<string> {
    // Check if renamify is available
    const isAvailable = await this.renamifyService.checkAvailability();
    if (!isAvailable) {
      return `Error: Renamify CLI is not available. Please ensure 'renamify' is installed and in your PATH.

To install renamify:
1. Download the latest release from GitHub
2. Add the binary to your PATH
3. Or build from source using Rust

For more information, visit: https://github.com/docspring/renamify`;
    }

    try {
      const result = await this.renamifyService.plan(params);

      // Add helpful context for AI agents
      const helpText = `
${result}

NEXT STEPS:
1. Review the plan above to ensure it matches your expectations
2. Use 'renamify_apply' to apply these changes
3. Or use 'renamify_preview' with different formats to see more details
4. Use 'renamify_status' to see pending plans

SAFETY NOTES:
- Plans are saved in .renamify/plan.json
- Apply is atomic by default - all changes succeed or none are made
- Use 'renamify_undo' to revert if needed`;

      return params.dryRun ? result : helpText;
    } catch (error) {
      return `Error creating plan: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Apply a renaming plan
   */
  async apply(params: ApplyOptions): Promise<string> {
    try {
      const result = await this.renamifyService.apply(params);

      return `${result}

SUCCESS: Changes have been applied successfully.

NEXT STEPS:
- Review the changes
- Run your tests to ensure everything works
- Use 'renamify_undo' with the history ID if you need to revert
- Use 'renamify_history' to see all applied renames`;
    } catch (error) {
      return `Error applying plan: ${error instanceof Error ? error.message : String(error)}

TROUBLESHOOTING:
- Ensure you have a valid plan (use 'renamify_status' to check)
- Check if files have been modified since the plan was created
- Try running with atomic=false if some files are causing issues`;
    }
  }

  /**
   * Undo a renaming
   */
  async undo(params: { id?: string }): Promise<string> {
    try {
      const result = await this.renamifyService.undo(params.id);
      return `${result}

Renaming has been undone successfully.`;
    } catch (error) {
      return `Error undoing renaming: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Redo a renaming
   */
  async redo(params: { id?: string }): Promise<string> {
    try {
      const result = await this.renamifyService.redo(params.id);
      return `${result}

Renaming has been redone successfully.`;
    } catch (error) {
      return `Error redoing renaming: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Show renaming history
   */
  async history(params: { limit?: number }): Promise<string> {
    try {
      const result = await this.renamifyService.history(params.limit);

      if (!result || result.trim() === '') {
        return 'No renaming history found.';
      }

      return `${result}

Use 'renamify_undo <id>' to revert any of these changes.`;
    } catch (error) {
      return `Error getting history: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Show renamify status
   */
  async status(_params: Record<string, never>): Promise<string> {
    try {
      const result = await this.renamifyService.status();

      if (!result || result.trim() === '') {
        return 'No pending plans or active renames.';
      }

      return `${result}

Use 'renamify_apply' to apply pending plans or 'renamify_preview' to review them.`;
    } catch (error) {
      return `Error getting status: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Preview a plan
   */
  async preview(params: PreviewOptions): Promise<string> {
    try {
      const result = await this.renamifyService.preview(params);
      return `${result}

This is a preview only. Use 'renamify_apply' to make these changes.`;
    } catch (error) {
      return `Error previewing plan: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Rename with case-aware transformations (plan + apply in one step)
   */
  async rename(params: {
    old: string;
    new: string;
    paths?: string[];
    dryRun?: boolean;
    preview?: 'table' | 'diff' | 'summary' | 'json';
    excludeStyles?: string[];
    includeStyles?: string[];
    onlyStyles?: string[];
  }): Promise<string> {
    try {
      const result = await this.renamifyService.rename(params);

      if (params.dryRun) {
        return `${result}

This was a dry run. Use without --dry-run to apply these changes.`;
      }

      return `${result}

Renaming completed successfully. Use 'renamify_undo' if you need to revert.`;
    } catch (error) {
      return `Error during rename: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Simple regex or literal replacement without case transformations
   */
  async replace(params: {
    pattern: string;
    replacement: string;
    paths?: string[];
    noRegex?: boolean;
    preview?: 'table' | 'diff' | 'summary' | 'json';
    dryRun?: boolean;
  }): Promise<string> {
    try {
      const result = await this.renamifyService.replace(params);

      if (params.dryRun) {
        return `${result}

This was a dry run. Use without --dry-run to apply these changes.`;
      }

      return `${result}

Replacement completed successfully. Use 'renamify_undo' if you need to revert.`;
    } catch (error) {
      return `Error during replace: ${error instanceof Error ? error.message : String(error)}`;
    }
  }
}
