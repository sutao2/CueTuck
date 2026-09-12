import { beforeEach, expect, it, vi } from 'vitest';
import { cachedPromptThumbnail, clearThumbnailCache } from './thumbnailCache.js';
import { firstPromptThumbnail } from './assets.js';
vi.mock('./assets.js', () => ({ firstPromptThumbnail: vi.fn() }));
beforeEach(() => { clearThumbnailCache(); firstPromptThumbnail.mockReset().mockResolvedValue({ data: 'small', mime: 'image/png' }); });
it('shares concurrent reads and revisits, but loads a new revision', async () => {
  await Promise.all([cachedPromptThumbnail('one', 1), cachedPromptThumbnail('one', 1)]);
  await cachedPromptThumbnail('one', 1);
  expect(firstPromptThumbnail).toHaveBeenCalledTimes(1);
  await cachedPromptThumbnail('one', 2);
  expect(firstPromptThumbnail).toHaveBeenCalledTimes(2);
});
it('evicts least recently used entries by count and encoded-string memory', async () => {
  for (let i = 0; i < 40; i++) await cachedPromptThumbnail(String(i), 1);
  await cachedPromptThumbnail('0', 1); await cachedPromptThumbnail('40', 1);
  await cachedPromptThumbnail('0', 1); expect(firstPromptThumbnail).toHaveBeenCalledTimes(41);
  await cachedPromptThumbnail('1', 1); expect(firstPromptThumbnail).toHaveBeenCalledTimes(42);
  clearThumbnailCache(); firstPromptThumbnail.mockClear().mockResolvedValue({ data: 'a'.repeat(2 * 1024 * 1024) });
  for (const id of ['a', 'b', 'c', 'b']) await cachedPromptThumbnail(id, 1);
  expect(firstPromptThumbnail).toHaveBeenCalledTimes(3);
  await cachedPromptThumbnail('a', 1); expect(firstPromptThumbnail).toHaveBeenCalledTimes(4);
});
it('does not cache failures, missing images or late results after reset', async () => {
  firstPromptThumbnail.mockRejectedValueOnce(Error('busy')).mockResolvedValueOnce(null);
  await expect(cachedPromptThumbnail('one', 1)).rejects.toThrow('busy');
  await cachedPromptThumbnail('one', 1); await cachedPromptThumbnail('one', 1);
  expect(firstPromptThumbnail).toHaveBeenCalledTimes(3);
  let finish; firstPromptThumbnail.mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
  const old = cachedPromptThumbnail('old', 1); clearThumbnailCache();
  finish({ data: 'old' }); await old;
  await cachedPromptThumbnail('old', 1); expect(firstPromptThumbnail).toHaveBeenCalledTimes(5);
});
it('cannot repopulate an older revision while a new revision is pending', async () => {
  let finish;
  firstPromptThumbnail.mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
  const old = cachedPromptThumbnail('one', 1);
  const current = await cachedPromptThumbnail('one', 2);
  finish({ data: 'stale' }); await old;
  expect(await cachedPromptThumbnail('one', 2)).toBe(current);
  expect(firstPromptThumbnail).toHaveBeenCalledTimes(2);
});
