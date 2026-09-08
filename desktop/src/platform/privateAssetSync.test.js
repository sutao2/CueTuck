import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { webcrypto } from 'node:crypto';
import { createLocalPrompt, updateLocalPrompt, listLocalPrompts, resetMemoryLibrary, setLocalSetting, exportLocalSyncChanges, applyLocalSyncChanges, timestampMillis } from './library.js';
import { memoryAssets } from './assets.js';
import { assetHash } from './privateMedia.js';
import { loginSession, resetMemorySession, setSessionTransport } from './session.js';
import { resetLibrarySync, setLibrarySyncTransport, setNetworkType, syncLocalLibraryNow } from './librarySync.js';

let account, files, transport;
const asset = () => ({ id: crypto.randomUUID(), name: 'notes.txt', mime: 'text/plain', data: btoa('private notes') });
beforeEach(async () => {
  vi.stubGlobal('crypto', webcrypto);
  resetMemoryLibrary(); resetMemorySession(); resetLibrarySync();
  setSessionTransport(async () => ({ email: 'a@example.com', access_token: 'token-a' }));
  await loginSession({ email: 'a@example.com', password: 'test' });
  account = new Map(); files = new Map();
  transport = {
    get: vi.fn(async () => ({ items: structuredClone([...account.values()]) })),
    put: vi.fn(async items => {
      for (const row of items) {
        const existing = account.get(row.id);
        if (existing && timestampMillis(existing.updated_at) >= timestampMillis(row.updated_at)) continue;
        const item = structuredClone(row);
        if (existing?.payload.asset_refs && !item.payload.asset_refs) item.payload.asset_refs = existing.payload.asset_refs;
        account.set(item.id, item);
      }
      return { items: [...account.values()] };
    }),
    upload: vi.fn(async file => {
      const ref = { id: file.id, media_id: `media.${crypto.randomUUID()}`, name: file.name, mime: file.mime, size: atob(file.data).length, sha256: await assetHash(file) };
      files.set(ref.media_id, structuredClone(file)); return ref;
    }),
    download: vi.fn(async ref => structuredClone(files.get(ref.media_id))),
  };
  setLibrarySyncTransport(transport);
});
afterEach(() => vi.unstubAllGlobals());

it('defaults to text only and restores verified private assets to a second library on explicit consent', async () => {
  const file = asset(); const prompt = await createLocalPrompt({ title: 'notes', content: 'body', assets: [file] });
  await syncLocalLibraryNow();
  expect(transport.upload).not.toHaveBeenCalled();
  expect(account.get(prompt.id).payload.assets).toBeUndefined();
  expect(account.get(prompt.id).payload.asset_refs).toBeUndefined();
  await syncLocalLibraryNow({ includeAssets: true });
  expect(transport.upload).toHaveBeenCalledTimes(1);
  expect(account.get(prompt.id).payload.assets).toBeUndefined();
  expect(account.get(prompt.id).payload.asset_refs).toHaveLength(1);
  await syncLocalLibraryNow({ includeAssets: true });
  expect(transport.upload).toHaveBeenCalledTimes(1);
  resetMemoryLibrary();
  await syncLocalLibraryNow();
  expect(memoryAssets(prompt.id)).toEqual([]);
  await syncLocalLibraryNow({ includeAssets: true });
  expect(memoryAssets(prompt.id)).toEqual([file]);
  expect((await listLocalPrompts())[0].asset_count).toBe(1);
  expect(transport.download).toHaveBeenCalledTimes(1);
});

it('preserves cloud references for text-only edits and does not propagate attachment deletion', async () => {
  const file = asset(); const prompt = await createLocalPrompt({ title: 'notes', content: 'body', assets: [file] });
  await syncLocalLibraryNow({ includeAssets: true });
  await updateLocalPrompt({ id: prompt.id, title: 'changed', content: 'new', assets: [] });
  await syncLocalLibraryNow();
  expect(account.get(prompt.id).payload.asset_refs).toHaveLength(1);
  expect(memoryAssets(prompt.id)).toEqual([]);
  await syncLocalLibraryNow({ includeAssets: true });
  expect(memoryAssets(prompt.id)).toEqual([file]);
});

it('keeps unsynced local files when a newer remote revision adds another attachment', async () => {
  const first = asset(); const prompt = await createLocalPrompt({ title: 'notes', content: 'body', assets: [first] });
  await syncLocalLibraryNow({ includeAssets: true });
  const second = asset(); await updateLocalPrompt({ id: prompt.id, title: 'local', content: 'local', assets: [first, second] });
  const remote = account.get(prompt.id); remote.updated_at = String(timestampMillis((await exportLocalSyncChanges()).find(p => p.id === prompt.id).updated_at) + 100); remote.payload.title = 'new remote';
  await syncLocalLibraryNow({ includeAssets: true });
  expect(memoryAssets(prompt.id)).toEqual([first, second]);
  expect((await listLocalPrompts())[0].title).toBe('new remote');
  expect(account.get(prompt.id).payload.asset_refs).toHaveLength(2);
});

it('defers all files on unknown wifi and retries successfully when wifi returns', async () => {
  await createLocalPrompt({ title: 'notes', content: 'body', assets: [asset()] });
  await setLocalSetting('sync_wifi_images', '1'); setNetworkType('unknown');
  expect((await syncLocalLibraryNow({ includeAssets: true })).attachmentsDeferred).toBe(true);
  expect(transport.upload).not.toHaveBeenCalled();
  setNetworkType('wifi');
  expect((await syncLocalLibraryNow({ includeAssets: true })).attachmentsDeferred).toBe(false);
  expect(transport.upload).toHaveBeenCalledTimes(1);
});

it('does not land corrupt downloads or partial prompt changes and supports retry', async () => {
  const file = asset(); const prompt = await createLocalPrompt({ title: 'notes', content: 'body', assets: [file] });
  await syncLocalLibraryNow({ includeAssets: true }); resetMemoryLibrary();
  transport.download.mockImplementationOnce(async () => ({ ...file, data: btoa('corrupt') }));
  await expect(syncLocalLibraryNow({ includeAssets: true })).rejects.toThrow(/校验/);
  expect(await listLocalPrompts()).toEqual([]); expect(memoryAssets(prompt.id)).toEqual([]);
  await syncLocalLibraryNow({ includeAssets: true }); expect(memoryAssets(prompt.id)).toEqual([file]);
});

it('checks session before the next upload or local write after logout', async () => {
  await createLocalPrompt({ title: 'notes', content: 'body', assets: [asset(), asset()] });
  transport.upload.mockImplementationOnce(async file => { resetMemorySession(); return file; });
  await expect(syncLocalLibraryNow({ includeAssets: true })).rejects.toThrow(/登录状态/);
  expect(transport.put).not.toHaveBeenCalled(); expect(transport.upload).toHaveBeenCalledTimes(1);
});

it('does not run overlapping syncs', async () => {
  let release; transport.put.mockImplementationOnce(() => new Promise(resolve => { release = resolve; }));
  const first = syncLocalLibraryNow();
  await vi.waitFor(() => expect(release).toBeTypeOf('function'));
  await expect(syncLocalLibraryNow()).rejects.toThrow(/正在进行/);
  release({ items: [] }); await first;
});

it('merges equal-version assets atomically and preserves existing files with keep-local', async () => {
  const file = asset(); const prompt = await createLocalPrompt({ title: 'notes', content: 'body', assets: [file] });
  const change = (await exportLocalSyncChanges()).find(row => row.id === prompt.id);
  const addition = asset(); change.payload.assets = [addition];
  await applyLocalSyncChanges([change], { includeAssets: true, keepLocal: true });
  expect(memoryAssets(prompt.id)).toEqual([file]);
  const invalid = { ...change, id: 'bad', payload: { title: 'bad', category_id: 'missing', assets: [addition] } };
  await expect(applyLocalSyncChanges([change, invalid], { includeAssets: true })).rejects.toThrow();
  expect(memoryAssets(prompt.id)).toEqual([file]);
  await applyLocalSyncChanges([change], { includeAssets: true });
  expect(memoryAssets(prompt.id)).toEqual([file, addition]);
});
