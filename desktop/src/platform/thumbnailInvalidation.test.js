import { afterEach, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { cachedPromptThumbnail, clearThumbnailCache } from './thumbnailCache.js';
import { applyLocalImport, restoreLocalLibrary } from './library.js';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
afterEach(() => { delete window.__TAURI_INTERNALS__; clearThumbnailCache(); vi.resetAllMocks(); });
it('invalidates same-version thumbnails after native import or restore, but preserves cache on failed restore', async () => {
  window.__TAURI_INTERNALS__ = {};
  invoke.mockImplementation(async command => command === 'get_local_prompt_thumbnail' ? { data: 'preview', mime: 'image/png' } : {});
  const load = () => cachedPromptThumbnail('one', 'same-revision');
  const reads = () => invoke.mock.calls.filter(([command]) => command === 'get_local_prompt_thumbnail').length;
  await load(); await load(); expect(reads()).toBe(1);
  expect(invoke).toHaveBeenCalledWith('get_local_prompt_thumbnail', { promptId: 'one' });
  await restoreLocalLibrary('/test/backup.sqlite'); await load(); expect(reads()).toBe(2);
  await applyLocalImport('{}'); await load(); expect(reads()).toBe(3);
  invoke.mockRejectedValueOnce(Error('restore failed'));
  await expect(restoreLocalLibrary('/test/backup.sqlite')).rejects.toThrow('restore failed');
  await load(); expect(reads()).toBe(3);
});
