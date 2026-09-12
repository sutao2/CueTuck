import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import * as library from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';

let w;
beforeEach(() => { library.resetMemoryLibrary(); resetMemorySession(); });
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
async function setup() {
  await library.createLocalPrompt({ title: '待删模板', content: '正文' });
  w = mount(WorkbenchShell); await flushPromises();
}
async function requestDelete() {
  await w.get('.prompt-card').trigger('contextmenu');
  await w.get('[data-action="delete"]').trigger('click');
}
it('confirms list deletion without a modal and reports success and count changes', async () => {
  await setup(); await requestDelete();
  expect(w.get('[data-testid="delete-confirmation"]').text()).toContain('待删模板');
  expect(w.find('[aria-modal="true"]').exists()).toBe(false);
  expect(await library.listLocalPrompts()).toHaveLength(1);
  await w.get('[data-testid="delete-confirmation"] button').trigger('click');
  expect(w.find('[data-testid="delete-confirmation"]').exists()).toBe(false);
  expect(await library.listLocalPrompts()).toHaveLength(1);
  await requestDelete(); await w.get('[data-testid="confirm-delete"]').trigger('click'); await flushPromises();
  expect(await library.listLocalPrompts()).toHaveLength(0);
  expect(w.get('[data-testid="delete-notice"]').text()).toContain('已删除「待删模板」');
  expect(w.emitted('library-changed').at(-1)).toEqual([0]);
});
it('shows errors from list deletion and permits retry with a single busy request', async () => {
  await setup();
  const original = library.deleteLocalPrompt;
  const remove = vi.spyOn(library, 'deleteLocalPrompt').mockRejectedValueOnce(Error('磁盘不可写'));
  await requestDelete(); await w.get('[data-testid="confirm-delete"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="delete-notice"]').text()).toContain('删除失败：磁盘不可写');
  expect(await library.listLocalPrompts()).toHaveLength(1);
  let finish;
  remove.mockImplementationOnce(id => new Promise(resolve => { finish = async () => { await original(id); resolve(); }; }));
  await w.get('[data-testid="confirm-delete"]').trigger('click');
  expect(w.get('[data-testid="confirm-delete"]').element.disabled).toBe(true);
  await w.get('[data-testid="confirm-delete"]').trigger('click');
  expect(remove).toHaveBeenCalledTimes(2);
  await finish(); await flushPromises();
  expect(w.find('[data-testid="delete-confirmation"]').exists()).toBe(false);
  expect(await library.listLocalPrompts()).toHaveLength(0);
});
it('does not misreport successful deletion when refreshing the list fails', async () => {
  await setup(); await requestDelete();
  const read = library.listLocalPrompts;
  vi.spyOn(library, 'listLocalPrompts').mockRejectedValueOnce(Error('读取失败'));
  await w.get('[data-testid="confirm-delete"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="delete-notice"]').text()).toContain('已删除，但刷新失败');
  expect(w.find('[data-testid="delete-confirmation"]').exists()).toBe(false);
  expect(await read()).toHaveLength(0);
});
it('keeps editor changes when deletion is cancelled', async () => {
  await setup(); await w.get('.prompt-card').trigger('click'); await flushPromises();
  await w.get('[data-testid=detail-edit]').trigger('click');
  const editor = w.get('[data-testid="prompt-editor"]');
  await editor.get('textarea').setValue('未保存修改');
  await editor.get('.danger-button').trigger('click');
  await w.get('[data-testid="delete-confirmation"] button').trigger('click');
  expect(editor.get('textarea').element.value).toBe('未保存修改');
  expect(await library.listLocalPrompts()).toHaveLength(1);
});
