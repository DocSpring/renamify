import { execa } from 'execa';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { RenamifyService } from './renamify-service';

// Mock execa
vi.mock('execa');
const mockedExeca = vi.mocked(execa);

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

describe('RenamifyService', () => {
  let service: RenamifyService;

  beforeEach(() => {
    service = new RenamifyService();
    vi.clearAllMocks();
  });

  describe('checkAvailability', () => {
    it('should return true when renamify is available', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: 'renamify 0.1.0' } as never);
      const result = await service.checkAvailability();
      expect(result).toBe(true);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', ['--version']);
    });

    it('should return false when renamify is not available', async () => {
      mockedExeca.mockRejectedValueOnce(new Error('Command not found'));
      const result = await service.checkAvailability();
      expect(result).toBe(false);
    });
  });

  describe('version compatibility', () => {
    // For version compatibility tests, we need to override NODE_ENV
    // and manually test the version checking logic
    const originalEnv = process.env.NODE_ENV;

    beforeEach(() => {
      // Temporarily disable test mode to test version checking
      // biome-ignore lint/performance/noDelete: needed for test
      delete process.env.NODE_ENV;
    });

    afterEach(() => {
      // Restore original NODE_ENV
      process.env.NODE_ENV = originalEnv;
    });

    it('should pass when versions are compatible', async () => {
      // MCP version is 0.1.1, CLI version is 0.1.1
      mockedExeca
        .mockResolvedValueOnce({
          stdout: '{"name":"renamify","version":"0.1.1"}',
        } as never) // version check
        .mockResolvedValueOnce({ stdout: 'Plan created' } as never); // actual command

      const result = await service.plan({
        search: 'oldName',
        replace: 'newName',
      });

      expect(result).toBe('Plan created');
    });

    it('should pass when CLI has higher minor version', async () => {
      // MCP version is 0.1.1, CLI version is 0.2.0
      mockedExeca
        .mockResolvedValueOnce({
          stdout: '{"name":"renamify","version":"0.2.0"}',
        } as never) // version check
        .mockResolvedValueOnce({ stdout: 'Plan created' } as never); // actual command

      const result = await service.plan({
        search: 'oldName',
        replace: 'newName',
      });

      expect(result).toBe('Plan created');
    });

    it('should fail when major versions differ', async () => {
      // MCP version is 0.1.1, CLI version is 1.0.0
      mockedExeca.mockResolvedValueOnce({
        stdout: '{"name":"renamify","version":"1.0.0"}',
      } as never);

      await expect(
        service.plan({ search: 'foo', replace: 'bar' })
      ).rejects.toThrow(
        'Version mismatch: MCP server v0.1.1 is not compatible with CLI v1.0.0'
      );
    });

    it('should fail when MCP minor version is higher than CLI', async () => {
      // MCP version is 0.1.1, CLI version is 0.0.5
      mockedExeca.mockResolvedValueOnce({
        stdout: '{"name":"renamify","version":"0.0.5"}',
      } as never);

      await expect(
        service.plan({ search: 'foo', replace: 'bar' })
      ).rejects.toThrow(
        'Version mismatch: MCP server v0.1.1 requires CLI v0.1.x or later'
      );
    });

    it('should handle CLI not found error', async () => {
      const error = new Error('Command not found') as any;
      error.code = 'ENOENT';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(
        service.plan({ search: 'foo', replace: 'bar' })
      ).rejects.toThrow('Renamify CLI not found. Please install it using');
    });
  });

  describe('plan', () => {
    it('should create a basic plan', async () => {
      const mockOutput =
        'PLAN SUMMARY\nOld: oldName\nNew: newName\nTotal matches: 5';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      const result = await service.plan({
        search: 'oldName',
        replace: 'newName',
      });

      expect(result).toBe(mockOutput);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'plan',
        'oldName',
        'newName',
      ]);
    });

    it('should handle all plan options', async () => {
      const mockOutput = 'Plan created';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      await service.plan({
        search: 'oldName',
        replace: 'newName',
        includes: ['src/**/*.ts', 'lib/**/*.js'],
        excludes: ['node_modules/**', 'dist/**'],
        styles: ['snake', 'camel', 'pascal'],
        preview: 'summary',
        dryRun: true,
        renameFiles: false,
        renameDirs: false,
      });

      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'plan',
        'oldName',
        'newName',
        '--include',
        'src/**/*.ts',
        '--include',
        'lib/**/*.js',
        '--exclude',
        'node_modules/**',
        '--exclude',
        'dist/**',
        '--styles',
        'snake,camel,pascal',
        '--preview',
        'summary',
        '--dry-run',
        '--no-rename-files',
        '--no-rename-dirs',
      ]);
    });

    it('should handle plan errors', async () => {
      const errorMessage = 'No matches found';
      const error = new Error('Command failed') as never;
      error.stderr = errorMessage;
      mockedExeca.mockRejectedValueOnce(error);

      await expect(
        service.plan({ search: 'foo', replace: 'bar' })
      ).rejects.toThrow(`Renamify plan failed: ${errorMessage}`);
    });
  });

  describe('apply', () => {
    it('should apply with plan ID', async () => {
      const mockOutput = 'Applied 10 changes';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      const result = await service.apply({ planId: 'abc123' });

      expect(result).toBe(mockOutput);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'apply',
        '--id',
        'abc123',
      ]);
    });

    it('should apply with plan path', async () => {
      const mockOutput = 'Applied changes';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      await service.apply({ planPath: '/path/to/plan.json' });

      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'apply',
        '--plan',
        '/path/to/plan.json',
      ]);
    });

    it('should handle apply options', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: 'Applied' } as never);

      await service.apply({
        atomic: false,
        commit: true,
      });

      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'apply',
        '--no-atomic',
        '--commit',
      ]);
    });
  });

  describe('undo', () => {
    it('should undo a renaming', async () => {
      const mockOutput = 'Undone successfully';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      const result = await service.undo('history123');

      expect(result).toBe(mockOutput);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'undo',
        'history123',
      ]);
    });

    it('should handle undo errors', async () => {
      const error = new Error('Command failed') as never;
      error.stderr = 'History entry not found';
      mockedExeca.mockRejectedValueOnce(error);

      await expect(service.undo('invalid')).rejects.toThrow(
        'Renamify undo failed: History entry not found'
      );
    });
  });

  describe('redo', () => {
    it('should redo a renaming', async () => {
      const mockOutput = 'Redone successfully';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      const result = await service.redo('history123');

      expect(result).toBe(mockOutput);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'redo',
        'history123',
      ]);
    });
  });

  describe('history', () => {
    it('should get history without limit', async () => {
      const mockOutput = 'History entries...';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      const result = await service.history();

      expect(result).toBe(mockOutput);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', ['history']);
    });

    it('should get history with limit', async () => {
      const mockOutput = 'Limited history';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      await service.history(5);

      expect(mockedExeca).toHaveBeenCalledWith('renamify', [
        'history',
        '--limit',
        '5',
      ]);
    });
  });

  describe('status', () => {
    it('should get status', async () => {
      const mockOutput = 'Status: 1 pending plan';
      mockedExeca.mockResolvedValueOnce({ stdout: mockOutput } as never);

      const result = await service.status();

      expect(result).toBe(mockOutput);
      expect(mockedExeca).toHaveBeenCalledWith('renamify', ['status']);
    });
  });
});
