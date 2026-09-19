import { mount, flushPromises } from '@vue/test-utils';
import { it, expect } from 'vitest';
import LocalTrash from './LocalTrash.vue';
import * as library from '../platform/library.js';
import { listPromptAssets } from '../platform/assets.js';

it('restores a deleted prompt with its original files without duplicating it', async () => {
  library.resetMemoryLibrary();
  const asset = { id: crypto.randomUUID(), name: 'note.txt', mime: 'text/plain', data: btoa('original') };
  const prompt = await library.createLocalPrompt({ title: '删除后恢复', content: '正文', assets: [asset] });
  await library.deleteLocalPrompt(prompt.id);
  const w = mount(LocalTrash);
  w.element.open = true; await w.trigger('toggle'); await flushPromises();
  expect(w.text()).toContain('删除后恢复');
  await w.get('li button').trigger('click'); await flushPromises();
  expect(w.text()).toContain('已恢复');
  expect(await library.listLocalPrompts()).toHaveLength(1);
  expect(await listPromptAssets(prompt.id)).toEqual([asset]);
  await expect(library.restoreDeletedLocalItem(prompt.id, 'prompt')).rejects.toThrow('已恢复');
  w.unmount();
});

it('keeps surviving collection members in their current locations on recovery', async () => {
  library.resetMemoryLibrary();
  const collection = await library.createLocalCollection({ title: '旧合集' });
  const prompt = await library.createLocalPrompt({ title: '独立内容', content: '正文' });
  await library.addPromptToCollection(prompt.id, collection.id);
  await library.deleteLocalCollection(collection.id);
  await library.restoreDeletedLocalItem(collection.id, 'collection');
  expect((await library.listLocalCollections())[0].member_count).toBe(0);
  expect((await library.listLocalPrompts())[0].collection_id).toBeNull();
});
