import { beforeEach, expect, it } from 'vitest';
import { validateAssets, readAssetFiles, listPromptAssets, textPreview } from './assets.js';
import { createLocalPrompt, updateLocalPrompt, resetMemoryLibrary, exportLocalLibrary, applyLocalImport, listLocalPrompts, exportLocalSyncChanges, applyLocalSyncChanges } from './library.js';
const file = (name = 'notes.txt', text = 'reference') => ({ id: crypto.randomUUID(), name, mime: 'text/plain', data: btoa(text) });
beforeEach(resetMemoryLibrary);
it('keeps local assets with atomic validation, metadata counts and JSON import/export', async () => {
  const assets = [file()];
  const prompt = await createLocalPrompt({ title: 'with file', content: 'body', assets });
  expect(prompt.asset_count).toBe(1);
  expect((await listPromptAssets(prompt.id))[0]).toEqual(assets[0]);
  await expect(updateLocalPrompt({ id: prompt.id, title: 'bad', content: 'bad', assets: [file('../bad.txt')] })).rejects.toThrow();
  expect(prompt.title).toBe('with file');
  await applyLocalImport(await exportLocalLibrary());
  const copies = await listLocalPrompts(); expect(copies).toHaveLength(2);
  expect(await listPromptAssets(copies[0].id)).toEqual(assets);
});
it('does not upload assets or let remote payloads clear local assets', async () => {
  const assets = [file()]; const p = await createLocalPrompt({ title: 'local', content: 'body', assets });
  const changes = await exportLocalSyncChanges();
  expect(JSON.stringify(changes)).not.toContain(assets[0].data);
  await applyLocalSyncChanges([{ id: p.id, kind: 'prompt', updated_at: String(Number(p.updated_at) + 1), payload: { title: 'remote', assets: [] } }]);
  expect(await listPromptAssets(p.id)).toEqual(assets);
});
it('reads real selected files and rejects mixed unsupported batches without losing existing assets', async () => {
  const current = [file()];
  const result = await readAssetFiles([new File(['hello'], 'hello.md', { type: 'text/markdown' })], current);
  expect(result).toHaveLength(2); expect(textPreview(result[1])).toBe('hello');
  await expect(readAssetFiles([new File(['ok'], 'ok.txt'), new File(['<svg/>'], 'bad.svg')], current)).rejects.toThrow('不支持');
  expect(current).toHaveLength(1);
});
it('rejects spoofed signatures, unsafe names and oversized files, and bounds text previews', () => {
  expect(() => validateAssets([{ ...file('fake.png'), mime: 'image/png' }])).toThrow('不匹配');
  expect(() => validateAssets([file('bad\\file.txt')])).toThrow('名称');
  expect(() => validateAssets([file('large.txt', 'x'.repeat(5 * 1024 * 1024 + 1))])).toThrow('上限');
  expect(() => validateAssets(Array.from({length:13}, () => file()))).toThrow('12');
  expect(textPreview(file('long.txt', 'x'.repeat(15000)))).toContain('仅预览前 12 KB');
});
