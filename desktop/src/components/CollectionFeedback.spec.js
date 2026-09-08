import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import * as library from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setCatalogTransport, setSquareTransport } from '../platform/square.js';

let w;
beforeEach(() => {
  library.resetMemoryLibrary(); resetMemorySession(); resetSquare();
  setCatalogTransport(async () => ({ categories: [], models: [] }));
  setSquareTransport(async () => []);
});
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
const detail = () => w.get('[data-testid=collection-detail]');
const add = () => detail().findAll('button').find(button => button.text() === '加入合集');
async function open(title = 'A') {
  await w.findAll('.prompt-card').find(card => card.text().includes(title)).trigger('click');
  await flushPromises();
}
async function setup() {
  const collection = await library.createLocalCollection({ title: 'A' });
  await library.createLocalCollection({ title: 'B' });
  const prompt = await library.createLocalPrompt({ title: 'member', content: 'preserved' });
  w = mount(WorkbenchShell, { attachTo: document.body }); await flushPromises();
  return { collection, prompt };
}

it.each(['same', 'different', 'old failure'])('ignores stale collection reads after returning and reopening: %s', async mode => {
  await setup(); let resolve, reject;
  vi.spyOn(library, 'listCollectionMembers').mockImplementationOnce(() => new Promise((yes, no) => { resolve = yes; reject = no; }));
  await open();
  expect(detail().text()).toContain('正在读取合集');
  expect(detail().find('.collection-empty').exists()).toBe(false);
  expect(detail().get('[data-testid=edit-collection]').element.disabled).toBe(true);
  await detail().get('.page-back').trigger('click'); await flushPromises();
  await open(mode === 'different' ? 'B' : 'A');
  if (mode === 'old failure') reject(Error('stale error'));
  else resolve([{ id: 'stale', title: 'stale member' }]);
  await flushPromises();
  expect(detail().text()).not.toContain('stale');
  expect(detail().get('.collection-empty').exists()).toBe(true);
  expect(detail().get('h2').text()).toBe(mode === 'different' ? 'B' : 'A');
});

it('offers a read-only retry instead of presenting a failed read as an empty collection', async () => {
  const { collection, prompt } = await setup();
  await library.addPromptToCollection(prompt.id, collection.id);
  const write = vi.spyOn(library, 'addPromptToCollection');
  vi.spyOn(library, 'listCollectionMembers').mockRejectedValueOnce(Error('disk read failed'));
  await open();
  expect(detail().text()).toContain('读取失败');
  expect(detail().find('.collection-empty').exists()).toBe(false);
  await detail().get('[data-testid=retry-collection-load]').trigger('click'); await flushPromises();
  expect(detail().get('.member-title').text()).toBe('member');
  expect(write).not.toHaveBeenCalled();
});

it('retains the selected member on a failed write and guards repeated submission and navigation', async () => {
  const { prompt } = await setup(); await open();
  await detail().get('select').setValue(prompt.id); let fail;
  const write = vi.spyOn(library, 'addPromptToCollection').mockImplementationOnce(() => new Promise((_, reject) => { fail = reject; }));
  await add().trigger('click'); await add().trigger('click');
  await detail().get('.page-back').trigger('click');
  await w.get('[data-space=square]').trigger('click');
  expect(detail().exists()).toBe(true); expect(write).toHaveBeenCalledTimes(1);
  fail(Error('disk full')); await flushPromises();
  expect(detail().text()).toContain('加入失败');
  expect(detail().get('select').element.value).toBe(prompt.id);
  await add().trigger('click'); await flushPromises();
  expect(write).toHaveBeenCalledTimes(2);
  expect(detail().get('.member-title').text()).toBe('member');
  expect(detail().get('select').element.value).toBe('');
});

it.each(['add', 'remove'])('reports durable %s success when refresh fails; retry never repeats the write', async action => {
  const { collection, prompt } = await setup();
  if (action === 'remove') await library.addPromptToCollection(prompt.id, collection.id);
  await open();
  const write = vi.spyOn(library, action === 'add' ? 'addPromptToCollection' : 'removePromptFromCollection');
  vi.spyOn(library, 'listCollectionMembers').mockRejectedValueOnce(Error('refresh failed'));
  if (action === 'add') { await detail().get('select').setValue(prompt.id); await add().trigger('click'); }
  else await detail().get('[data-testid=remove-member]').trigger('click');
  await flushPromises();
  expect(w.get('[data-testid=collection-notice]').text()).toContain(action === 'add' ? '已加入合集，但刷新失败' : '已移出合集，但刷新失败');
  expect(detail().find('.member-title').exists()).toBe(false);
  await w.get('[data-testid=retry-operation-refresh]').trigger('click'); await flushPromises();
  expect(write).toHaveBeenCalledTimes(1);
  expect(detail().findAll('.member-title')).toHaveLength(action === 'add' ? 1 : 0);
  expect((await library.listLocalPrompts())[0].content).toBe('preserved');
});

it('does not let a delayed metadata refresh replace a reopened collection', async () => {
  const { collection, prompt } = await setup(); await open();
  vi.spyOn(library, 'listCollectionMembers').mockRejectedValueOnce(Error('read failed'));
  await detail().get('select').setValue(prompt.id); await add().trigger('click'); await flushPromises();
  let resolve;
  vi.spyOn(library, 'listLocalCollections').mockImplementationOnce(() => new Promise(yes => { resolve = yes; }));
  await w.get('[data-testid=retry-operation-refresh]').trigger('click');
  await detail().get('.page-back').trigger('click'); await flushPromises(); await open();
  resolve([{ ...collection, title: 'stale title' }]); await flushPromises();
  expect(detail().get('h2').text()).toBe('A');
  expect(detail().get('.member-title').text()).toBe('member');
});
