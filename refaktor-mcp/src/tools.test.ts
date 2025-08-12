import { describe, it, expect, beforeEach, vi } from 'vitest';
import { RefaktorTools } from './tools';
import { RefaktorService } from './refaktor-service';

// Mock RefaktorService
vi.mock('./refaktor-service');

describe('RefaktorTools', () => {
  let tools: RefaktorTools;
  let mockService: any;

  beforeEach(() => {
    mockService = new RefaktorService() as any;
    tools = new RefaktorTools(mockService);
    vi.clearAllMocks();
  });

  describe('plan', () => {
    it('should check availability and return error if not available', async () => {
      mockService.checkAvailability.mockResolvedValue(false);

      const result = await tools.plan({ old: 'foo', new: 'bar' });

      expect(result).toContain('Refaktor CLI is not available');
      expect(result).toContain('Please ensure \'refaktor\' is installed');
    });

    it('should create plan and add helpful context', async () => {
      mockService.checkAvailability.mockResolvedValue(true);
      mockService.plan.mockResolvedValue('PLAN SUMMARY\nTotal matches: 5');

      const result = await tools.plan({ old: 'foo', new: 'bar' });

      expect(result).toContain('PLAN SUMMARY');
      expect(result).toContain('NEXT STEPS:');
      expect(result).toContain('Use \'refaktor_apply\' to apply these changes');
      expect(result).toContain('SAFETY NOTES:');
    });

    it('should not add context for dry run', async () => {
      mockService.checkAvailability.mockResolvedValue(true);
      mockService.plan.mockResolvedValue('PLAN SUMMARY\nTotal matches: 5');

      const result = await tools.plan({ old: 'foo', new: 'bar', dryRun: true });

      expect(result).toBe('PLAN SUMMARY\nTotal matches: 5');
      expect(result).not.toContain('NEXT STEPS:');
    });

    it('should handle plan errors gracefully', async () => {
      mockService.checkAvailability.mockResolvedValue(true);
      mockService.plan.mockRejectedValue(new Error('No matches found'));

      const result = await tools.plan({ old: 'foo', new: 'bar' });

      expect(result).toContain('Error creating plan: No matches found');
    });
  });

  describe('apply', () => {
    it('should apply plan and provide success message', async () => {
      mockService.apply.mockResolvedValue('Applied 10 changes to 3 files');

      const result = await tools.apply({});

      expect(result).toContain('Applied 10 changes to 3 files');
      expect(result).toContain('SUCCESS: Changes have been applied successfully');
      expect(result).toContain('NEXT STEPS:');
      expect(result).toContain('Run your tests to ensure everything works');
    });

    it('should handle apply errors with troubleshooting tips', async () => {
      mockService.apply.mockRejectedValue(new Error('Conflicts detected'));

      const result = await tools.apply({});

      expect(result).toContain('Error applying plan: Conflicts detected');
      expect(result).toContain('TROUBLESHOOTING:');
      expect(result).toContain('Check if files have been modified');
    });
  });

  describe('undo', () => {
    it('should undo and confirm success', async () => {
      mockService.undo.mockResolvedValue('Undone 5 changes');

      const result = await tools.undo({ id: 'history123' });

      expect(result).toContain('Undone 5 changes');
      expect(result).toContain('Refactoring has been undone successfully');
    });

    it('should handle undo errors', async () => {
      mockService.undo.mockRejectedValue(new Error('History not found'));

      const result = await tools.undo({ id: 'invalid' });

      expect(result).toContain('Error undoing refactoring: History not found');
    });
  });

  describe('redo', () => {
    it('should redo and confirm success', async () => {
      mockService.redo.mockResolvedValue('Redone 5 changes');

      const result = await tools.redo({ id: 'history123' });

      expect(result).toContain('Redone 5 changes');
      expect(result).toContain('Refactoring has been redone successfully');
    });
  });

  describe('history', () => {
    it('should show history with usage hint', async () => {
      mockService.history.mockResolvedValue('ID: abc123 - Applied at 2024-01-01');

      const result = await tools.history({});

      expect(result).toContain('ID: abc123');
      expect(result).toContain('Use \'refaktor_undo <id>\' to revert');
    });

    it('should handle empty history', async () => {
      mockService.history.mockResolvedValue('');

      const result = await tools.history({});

      expect(result).toBe('No refactoring history found.');
    });

    it('should handle history errors', async () => {
      mockService.history.mockRejectedValue(new Error('Failed to read history'));

      const result = await tools.history({ limit: 10 });

      expect(result).toContain('Error getting history: Failed to read history');
    });
  });

  describe('status', () => {
    it('should show status with action hints', async () => {
      mockService.status.mockResolvedValue('1 pending plan');

      const result = await tools.status({});

      expect(result).toContain('1 pending plan');
      expect(result).toContain('Use \'refaktor_apply\' to apply pending plans');
    });

    it('should handle empty status', async () => {
      mockService.status.mockResolvedValue('');

      const result = await tools.status({});

      expect(result).toBe('No pending plans or active refactorings.');
    });
  });

  describe('preview', () => {
    it('should preview plan with reminder', async () => {
      mockService.preview.mockResolvedValue('PLAN PREVIEW\nTotal: 5 changes');

      const result = await tools.preview({ format: 'summary' });

      expect(result).toContain('PLAN PREVIEW');
      expect(result).toContain('This is a preview only. Use \'refaktor_apply\' to make these changes');
    });

    it('should handle preview errors', async () => {
      mockService.preview.mockRejectedValue(new Error('Plan not found'));

      const result = await tools.preview({ planId: 'invalid' });

      expect(result).toContain('Error previewing plan: Plan not found');
    });
  });
});