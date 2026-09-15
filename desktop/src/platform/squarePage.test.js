import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { listSquarePage, resetSquare, setSquarePageTransport } from './square.js';
import { resetMemorySession } from './session.js';
beforeEach(() => { resetSquare(); resetMemorySession(); });
afterEach(() => { vi.unstubAllGlobals(); vi.useRealTimers(); });

it('only requests a bounded browse page and forwards cancellation', async () => {
  let request;
  vi.stubGlobal('fetch', vi.fn((_url, options) => { request = options; return new Promise((_, reject) => options.signal.addEventListener('abort', () => reject(new DOMException('canceled', 'AbortError')))); }));
  const controller = new AbortController();
  const result = listSquarePage({ query: 'hello', offset: 48, signal: controller.signal });
  const rejected = expect(result).rejects.toThrow('canceled');
  expect(fetch.mock.calls[0][0]).toContain('/v1/square/browse?');
  expect(fetch.mock.calls[0][0]).toContain('offset=48&limit=48');
  controller.abort(); await rejected;
  expect(request.signal.aborted).toBe(true);
  expect(fetch).toHaveBeenCalledTimes(1);
});

it('times out stalled requests and never falls back to all items', async () => {
  vi.useFakeTimers();
  vi.stubGlobal('fetch', vi.fn((_url, options) => new Promise((_, reject) => options.signal.addEventListener('abort', () => reject(new Error('timeout'))))));
  const rejected = expect(listSquarePage()).rejects.toThrow('timeout');
  await vi.advanceTimersByTimeAsync(10_000); await rejected;
  expect(fetch).toHaveBeenCalledTimes(1);
});

it('rejects oversized pages, invalid totals and non-advancing cursors', async () => {
  for (const page of [
    { items: Array(49).fill({ id: 'x' }), total: 49, next_offset: null },
    { items: [{ id: 'x' }], total: -1, next_offset: null },
    { items: [{ id: 'x' }], total: 3, next_offset: 48 },
  ]) {
    setSquarePageTransport(async () => page);
    await expect(listSquarePage({ offset: 48 })).rejects.toThrow('分页响应无效');
  }
});


it('passes recommendation snapshots and accepts an empty removed slice with a next offset', async () => {
  vi.stubGlobal('fetch', vi.fn(async () => ({ok:true,json:async()=>({items:[],total:120,next_offset:96,recommendation:'test-seed'})})));
  const page = await listSquarePage({offset:48,recommendation:'test-seed'});
  expect(fetch.mock.calls[0][0]).toContain('recommendation=test-seed');
  expect(page.next_offset).toBe(96);
});
it('explains an expired recommendation without falling back to an unrelated page', async () => {
  vi.stubGlobal('fetch', vi.fn(async () => ({ok:false,status:409})));
  await expect(listSquarePage({offset:48,recommendation:'expired'})).rejects.toThrow('本轮推荐已过期');
  expect(fetch).toHaveBeenCalledTimes(1);
});
