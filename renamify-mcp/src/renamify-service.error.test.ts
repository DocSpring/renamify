import { access } from 'node:fs/promises';
import { execa } from 'execa';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { RenamifyService } from './renamify-service';

// Mock execa
vi.mock('execa');
const mockedExeca = vi.mocked(execa);

// Mock fs/promises
vi.mock('node:fs/promises', () => ({
  access: vi.fn(),
}));
const mockedAccess = vi.mocked(access);

// Mock the package.json import
vi.mock('node:fs', async () => {
  const actual = await vi.importActual('node:fs');
  return {
    ...actual,
    readFileSync: vi.fn((path: string) => {
      if (path.includes('package.json')) {
        return JSON.stringify({ version: '0.1.1' });
      }
      return '';
    }),
  };
});

describe('RenamifyService Error Handling', () => {
  let service: RenamifyService;

  beforeEach(() => {
    service = new RenamifyService();
    vi.clearAllMocks();
  });

  describe('executeCommand error handling', () => {
    it('should handle missing binary with helpful error message', async () => {
      const error: any = new Error('Command failed');
      error.code = 'ENOENT';
      error.command = 'renamify plan old new';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(
        service.plan({ search: 'old', replace: 'new' })
      ).rejects.toThrow(
        'Renamify CLI not found. Please install it using:\n\n' +
          'curl -fsSL https://docspring.github.io/renamify/install.sh | bash\n\n' +
          'For more installation options, visit: https://docspring.github.io/renamify/installation/'
      );
    });

    it('should handle stderr errors', async () => {
      const error: any = new Error('Command failed');
      error.stderr = 'Error: Invalid syntax';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(
        service.plan({ search: 'old', replace: 'new' })
      ).rejects.toThrow('Renamify plan failed: Error: Invalid syntax');
    });

    it('should handle errors with stderr but no content', async () => {
      const error: any = new Error('Some error');
      error.stderr = '';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(
        service.plan({ search: 'old', replace: 'new' })
      ).rejects.toThrow('Renamify plan failed: Some error');
    });

    it('should throw errors without stderr as-is', async () => {
      const error = new Error('Some error');
      mockedExeca.mockRejectedValueOnce(error);

      await expect(
        service.plan({ search: 'old', replace: 'new' })
      ).rejects.toThrow('Some error');
    });

    it('should throw unknown errors as-is', async () => {
      const error = 'Not an Error object';
      mockedExeca.mockRejectedValueOnce(error as any);

      await expect(
        service.plan({ search: 'old', replace: 'new' })
      ).rejects.toBe(error);
    });
  });

  describe('handlePreviewError', () => {
    it('should handle ENOENT error for missing plan file', async () => {
      const error: any = new Error('File not found');
      error.code = 'ENOENT';
      mockedAccess.mockRejectedValueOnce(error);

      await expect(
        service.preview({ planPath: '/path/to/plan.json' })
      ).rejects.toThrow('Plan file not found: /path/to/plan.json');
    });

    it('should handle ENOENT error for missing binary in preview', async () => {
      const error: any = new Error('Command failed');
      error.code = 'ENOENT';
      error.command = 'renamify plan --preview-only';
      mockedAccess.mockResolvedValueOnce(undefined);
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.preview({})).rejects.toThrow(
        'Renamify CLI not found. Please install it using:\n\n' +
          'curl -fsSL https://docspring.github.io/renamify/install.sh | bash\n\n' +
          'For more installation options, visit: https://docspring.github.io/renamify/installation/'
      );
    });

    it('should handle stderr errors in preview', async () => {
      const error: any = new Error('Preview failed');
      error.stderr = 'Invalid plan format';
      mockedAccess.mockResolvedValueOnce(undefined);
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.preview({})).rejects.toThrow(
        'Renamify preview failed: Invalid plan format'
      );
    });

    it('should handle errors without stderr in preview', async () => {
      const error = new Error('Preview error');
      mockedAccess.mockResolvedValueOnce(undefined);
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.preview({})).rejects.toThrow('Preview error');
    });

    it('should throw non-Error objects as-is in preview', async () => {
      const error = { some: 'object' };
      mockedAccess.mockResolvedValueOnce(undefined);
      mockedExeca.mockRejectedValueOnce(error as any);

      await expect(service.preview({})).rejects.toBe(error);
    });
  });

  describe('apply command error handling', () => {
    it('should handle apply command failures', async () => {
      const error: any = new Error('Apply failed');
      error.stderr = 'Conflicts detected';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.apply({})).rejects.toThrow(
        'Renamify apply failed: Conflicts detected'
      );
    });
  });

  describe('undo command error handling', () => {
    it('should handle undo command failures', async () => {
      const error: any = new Error('Undo failed');
      error.stderr = 'No backup found';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.undo()).rejects.toThrow(
        'Renamify undo failed: No backup found'
      );
    });
  });

  describe('redo command error handling', () => {
    it('should handle redo command failures', async () => {
      const error: any = new Error('Redo failed');
      error.stderr = 'Nothing to redo';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.redo()).rejects.toThrow(
        'Renamify redo failed: Nothing to redo'
      );
    });
  });

  describe('history command error handling', () => {
    it('should handle history command failures', async () => {
      const error: any = new Error('History failed');
      error.stderr = 'History file corrupted';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.history()).rejects.toThrow(
        'Renamify history failed: History file corrupted'
      );
    });
  });

  describe('status command error handling', () => {
    it('should handle status command failures', async () => {
      const error: any = new Error('Status failed');
      error.stderr = 'Cannot read status';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.status()).rejects.toThrow(
        'Renamify status failed: Cannot read status'
      );
    });
  });
});
