import type {
  ApplyOptions,
  PlanOptions,
  PreviewOptions,
  RefaktorService,
} from './refaktor-service.js';

export class RefaktorTools {
  private refaktorService: RefaktorService;

  constructor(refaktorService: RefaktorService) {
    this.refaktorService = refaktorService;
  }

  /**
   * Create a refactoring plan
   */
  async plan(params: PlanOptions): Promise<string> {
    // Check if refaktor is available
    const isAvailable = await this.refaktorService.checkAvailability();
    if (!isAvailable) {
      return `Error: Refaktor CLI is not available. Please ensure 'refaktor' is installed and in your PATH.

To install refaktor:
1. Download the latest release from GitHub
2. Add the binary to your PATH
3. Or build from source using Rust

For more information, visit: https://github.com/docspring/refaktor`;
    }

    try {
      const result = await this.refaktorService.plan(params);

      // Add helpful context for AI agents
      const helpText = `
${result}

NEXT STEPS:
1. Review the plan above to ensure it matches your expectations
2. Use 'refaktor_apply' to apply these changes
3. Or use 'refaktor_preview' with different formats to see more details
4. Use 'refaktor_status' to see pending plans

SAFETY NOTES:
- Plans are saved in .refaktor/plan.json
- Apply is atomic by default - all changes succeed or none are made
- Use 'refaktor_undo' to revert if needed`;

      return params.dryRun ? result : helpText;
    } catch (error) {
      return `Error creating plan: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Apply a refactoring plan
   */
  async apply(params: ApplyOptions): Promise<string> {
    try {
      const result = await this.refaktorService.apply(params);

      return `${result}

SUCCESS: Changes have been applied successfully.

NEXT STEPS:
- Review the changes
- Run your tests to ensure everything works
- Use 'refaktor_undo' with the history ID if you need to revert
- Use 'refaktor_history' to see all applied refactorings`;
    } catch (error) {
      return `Error applying plan: ${error instanceof Error ? error.message : String(error)}

TROUBLESHOOTING:
- Ensure you have a valid plan (use 'refaktor_status' to check)
- Check if files have been modified since the plan was created
- Try running with atomic=false if some files are causing issues`;
    }
  }

  /**
   * Undo a refactoring
   */
  async undo(params: { id: string }): Promise<string> {
    try {
      const result = await this.refaktorService.undo(params.id);
      return `${result}

Refactoring has been undone successfully.`;
    } catch (error) {
      return `Error undoing refactoring: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Redo a refactoring
   */
  async redo(params: { id: string }): Promise<string> {
    try {
      const result = await this.refaktorService.redo(params.id);
      return `${result}

Refactoring has been redone successfully.`;
    } catch (error) {
      return `Error redoing refactoring: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Show refactoring history
   */
  async history(params: { limit?: number }): Promise<string> {
    try {
      const result = await this.refaktorService.history(params.limit);

      if (!result || result.trim() === '') {
        return 'No refactoring history found.';
      }

      return `${result}

Use 'refaktor_undo <id>' to revert any of these changes.`;
    } catch (error) {
      return `Error getting history: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Show refaktor status
   */
  async status(_params: Record<string, never>): Promise<string> {
    try {
      const result = await this.refaktorService.status();

      if (!result || result.trim() === '') {
        return 'No pending plans or active refactorings.';
      }

      return `${result}

Use 'refaktor_apply' to apply pending plans or 'refaktor_preview' to review them.`;
    } catch (error) {
      return `Error getting status: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  /**
   * Preview a plan
   */
  async preview(params: PreviewOptions): Promise<string> {
    try {
      const result = await this.refaktorService.preview(params);
      return `${result}

This is a preview only. Use 'refaktor_apply' to make these changes.`;
    } catch (error) {
      return `Error previewing plan: ${error instanceof Error ? error.message : String(error)}`;
    }
  }
}
