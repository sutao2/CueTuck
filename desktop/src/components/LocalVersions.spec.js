import { mount, flushPromises } from '@vue/test-utils';
import { it, expect } from 'vitest';
import LocalVersions from './LocalVersions.vue';
import * as library from '../platform/library.js';
import { listPromptAssets } from '../platform/assets.js';

it('keeps the losing local version and files, then restores a separate copy', async () => {
  library.resetMemoryLibrary();
  const file = { id: crypto.randomUUID(), name: 'a.txt', mime: 'text/plain', data: btoa('original') };
  const prompt = await library.createLocalPrompt({ title: '原始标题', content: '本机正文', assets: [file] });
  const incoming = (await library.exportLocalSyncChanges()).find(item => item.id === prompt.id);
  incoming.updated_at = String(Number(incoming.updated_at) + 1000); incoming.payload.content = '远端正文';
  await library.applyLocalSyncChanges([incoming], { keepLocal: true });
  expect(await library.listLocalPromptVersions()).toHaveLength(0);
  await expect(library.applyLocalSyncChanges([incoming, { ...incoming, id: 'broken', payload: { title: '坏记录', category_id: 'missing' } }])).rejects.toThrow();
  expect(await library.listLocalPromptVersions()).toHaveLength(0);
  await library.applyLocalSyncChanges([incoming]); await library.applyLocalSyncChanges([incoming]);
  expect(await library.listLocalPromptVersions()).toHaveLength(1);
  const w = mount(LocalVersions); w.element.open = true; await new Promise(resolve => setTimeout(resolve, 0)); await flushPromises();
  await w.get('li button').trigger('click'); await flushPromises();
  expect(w.get('pre').text()).toBe('本机正文');
  await w.get('section button').trigger('click'); await flushPromises();
  expect(w.get('section button').attributes('disabled')).toBeDefined();
  const rows = await library.listLocalPrompts(); expect(rows).toHaveLength(2);
  expect(rows.find(row => row.id === prompt.id).content).toBe('远端正文');
  const copy = rows.find(row => row.id !== prompt.id);
  expect(copy.content).toBe('本机正文'); expect(await listPromptAssets(copy.id)).toEqual([file]);
  w.unmount();
});
