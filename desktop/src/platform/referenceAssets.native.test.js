import { afterEach, expect, it, vi } from 'vitest';
import { downloadReferenceImages } from './referenceAssets.js';
import { invoke } from '@tauri-apps/api/core';
vi.mock('@tauri-apps/api/core', () => ({
  Channel: class { constructor(callback) { this.onmessage = callback; } },
  invoke: vi.fn(),
}));
afterEach(() => vi.unstubAllGlobals());
it('uses one native batch and ignores channel progress after the command settles', async () => {
  vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
  const progress = vi.fn(), urls = ['https://cms-assets.youmind.com/a.png'];
  const asset = {id:crypto.randomUUID(),name:'参考图-1.png',mime:'image/png',data:btoa('\x89PNG\r\n\x1a\n')};
  invoke.mockImplementation(async (command, args) => {
    expect(command).toBe('download_reference_images'); expect(args.urls).toEqual(urls);
    args.onProgress.onmessage({completed:1,total:1});
    return [asset];
  });
  expect(await downloadReferenceImages({reference:{images:urls}}, progress)).toEqual([asset]);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(progress).toHaveBeenLastCalledWith({completed:1,total:1});
  const calls = progress.mock.calls.length;
  invoke.mock.calls[0][1].onProgress.onmessage({completed:0,total:1});
  expect(progress).toHaveBeenCalledTimes(calls);
});
