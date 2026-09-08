import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import * as library from './library.js';
import { loginSession, resetMemorySession, setSessionTransport } from './session.js';
import { resetLibrarySync, setLibrarySyncTransport, syncLocalLibraryNow } from './librarySync.js';

let account, transport;
beforeEach(async () => {
  library.resetMemoryLibrary(); resetMemorySession(); resetLibrarySync();
  setSessionTransport(async () => ({ email: 'recovery@example.test', access_token: 'test' }));
  await loginSession({ email: 'recovery@example.test', password: 'test' });
  account = new Map();
  transport = {
    put: vi.fn(async rows => {
      for (const row of rows) if (!account.has(row.id) || library.timestampMillis(row.updated_at) > library.timestampMillis(account.get(row.id).updated_at)) account.set(row.id, structuredClone(row));
      return {};
    }),
    get: vi.fn(async () => ({ items: structuredClone([...account.values()]) })),
  };
  setLibrarySyncTransport(transport);
});
afterEach(() => { vi.useRealTimers(); vi.restoreAllMocks(); });

it('recovers after cloud write succeeds but read fails without duplicate records or losing local text', async () => {
  const prompt = await library.createLocalPrompt({ title: 'recover', content: 'original' });
  transport.get.mockRejectedValueOnce(Error('disconnected'));
  await expect(syncLocalLibraryNow()).rejects.toThrow('disconnected');
  expect(account.get(prompt.id).payload.content).toBe('original');
  expect((await library.listLocalPrompts())[0].content).toBe('original');
  await syncLocalLibraryNow();
  expect(account.size).toBeGreaterThan(0);
  expect([...account.values()].filter(row => row.kind === 'prompt')).toHaveLength(1);
  expect(await library.listLocalPrompts()).toHaveLength(1);
});

it('rejects concurrent sync and allows retry after the first request fails', async () => {
  let fail;
  transport.put.mockImplementationOnce(() => new Promise((_, reject) => { fail = reject; }));
  const first = syncLocalLibraryNow();
  const rejection = expect(first).rejects.toThrow('offline');
  await vi.waitFor(() => expect(transport.put).toHaveBeenCalledTimes(1));
  await expect(syncLocalLibraryNow()).rejects.toThrow('同步正在进行');
  fail(Error('offline')); await rejection;
  await syncLocalLibraryNow(); expect(transport.put).toHaveBeenCalledTimes(2);
});

it('preserves a newer local edit made while the network response is in flight', async () => {
  vi.useFakeTimers(); vi.setSystemTime(new Date('2026-09-08T01:00:00Z'));
  const prompt = await library.createLocalPrompt({ title: 'concurrent', content: 'old' });
  transport.get.mockImplementationOnce(async () => {
    vi.advanceTimersByTime(1000);
    await library.updateLocalPrompt({ id: prompt.id, title: 'concurrent', content: 'new edit' });
    return { items: structuredClone([...account.values()]) };
  });
  await syncLocalLibraryNow();
  expect((await library.listLocalPrompts())[0].content).toBe('new edit');
  await syncLocalLibraryNow();
  expect(account.get(prompt.id).payload.content).toBe('new edit');
});
