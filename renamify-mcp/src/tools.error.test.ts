import { beforeEach, describe, expect, it, vi } from 'vitest';
import { RenamifyService } from './renamify-service';
import { RenamifyTools } from './tools';

describe('RenamifyTools Error Handling', () => {
  let service: RenamifyService;
  let tools: RenamifyTools;

  beforeEach(() => {
    service = new RenamifyService();
    tools = new RenamifyTools(service);
    vi.clearAllMocks();
  });

  describe('redo error handling', () => {
    it('should handle redo errors with Error objects', async () => {
      vi.spyOn(service, 'redo').mockRejectedValueOnce(
        new Error('Redo operation failed')
      );

      const result = await tools.redo({});
      expect(result).toBe('Error redoing renaming: Redo operation failed');
    });

    it('should handle redo errors with non-Error objects', async () => {
      vi.spyOn(service, 'redo').mockRejectedValueOnce('String error');

      const result = await tools.redo({ id: 'test' });
      expect(result).toBe('Error redoing renaming: String error');
    });
  });

  describe('status error handling', () => {
    it('should handle status errors with Error objects', async () => {
      vi.spyOn(service, 'status').mockRejectedValueOnce(
        new Error('Status check failed')
      );

      const result = await tools.status({});
      expect(result).toBe('Error getting status: Status check failed');
    });

    it('should handle status errors with non-Error objects', async () => {
      vi.spyOn(service, 'status').mockRejectedValueOnce(123);

      const result = await tools.status({});
      expect(result).toBe('Error getting status: 123');
    });
  });
});
