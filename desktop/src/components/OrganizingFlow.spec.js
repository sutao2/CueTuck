import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import * as library from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
let w;
beforeEach(() => { library.resetMemoryLibrary(); resetMemorySession(); });
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
it('moves only selected categories and retries only failed rows', async () => {
  const a = await library.createLocalPrompt({ title: 'A', content: '正文 A' });
  const b = await library.createLocalPrompt({ title: 'B', content: '正文 B' });
  const c = await library.createLocalPrompt({ title: 'C', content: '正文 C' });
  w = mount(WorkbenchShell); await flushPromises();
  await w.get('[data-testid=select-prompts]').trigger('click');
  await w.get(`[data-select-prompt="${a.id}"]`).setValue(true); await w.get(`[data-select-prompt="${b.id}"]`).setValue(true);
  await w.get('[data-testid=batch-category]').setValue('cat-image');
  const original = library.moveLocalPromptCategory;
  let first = true;
  const move = vi.spyOn(library, 'moveLocalPromptCategory').mockImplementation(async (id, category) => {
    if (id === b.id && first) { first = false; throw Error('磁盘忙'); }
    return original(id, category);
  });
  await w.get('[data-testid=apply-batch]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=batch-result]').text()).toContain('失败 1 条');
  expect(w.get(`[data-select-prompt="${b.id}"]`).element.checked).toBe(true);
  expect(w.get(`[data-select-prompt="${a.id}"]`).element.checked).toBe(false);
  await w.get('[data-testid=apply-batch]').trigger('click'); await flushPromises();
  expect(move.mock.calls.filter(([id]) => id === a.id)).toHaveLength(1);
  const rows = await library.listLocalPrompts();
  expect(rows.find(p => p.id === c.id).category_id).toBeNull();
  expect(rows.find(p => p.id === a.id).content).toBe('正文 A');
});
it('searches and adds multiple collection members while preserving existing text', async () => {
  const collection = await library.createLocalCollection({ title: '合集' });
  const a = await library.createLocalPrompt({ title: '成员 A', content: '正文 A' });
  const b = await library.createLocalPrompt({ title: '成员 B', content: '正文 B' });
  w = mount(WorkbenchShell); await flushPromises();
  await w.findAll('.prompt-title').find(t => t.text() === '合集').trigger('click'); await flushPromises();
  await w.get(`[data-member-choice="${a.id}"]`).setValue(true);
  await w.get('.member-picker input[type=search]').setValue('成员 B');
  expect(w.find(`[data-member-choice="${a.id}"]`).exists()).toBe(false);
  await w.get(`[data-member-choice="${b.id}"]`).setValue(true);
  await w.get('[data-testid=collection-detail] .primary-button').trigger('click'); await flushPromises();
  const members = await library.listCollectionMembers(collection.id);
  expect(members).toHaveLength(2);
  expect(members.map(p => p.content)).toEqual(expect.arrayContaining(['正文 A', '正文 B']));
});
