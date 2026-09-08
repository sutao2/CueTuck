import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import ContentManagement from './ContentManagement.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';
let w;
const row = { id: 'a', title: '作者标题', content: '不可改的原始正文', members: [], revision: 0, excerpt: '原摘要', category_id: null, model: null, recommended: false, sort_index: 0, visibility: 'online', history: [] };
beforeEach(() => { resetAdminApi(); vi.spyOn(window, 'confirm').mockReturnValue(true); });
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
async function open(write = vi.fn(), detail = row) {
  setAdminApiTransport(request => request.kind === 'content' ? { items: [detail], total: 26, categories: [] } : request.kind === 'contentDetail' ? { ...detail } : write(request));
  w = mount(ContentManagement); await flushPromises(); await w.get('[data-testid=content-edit]').trigger('click'); await flushPromises(); return w;
}
it('keeps source content read-only and requires a reason for visibility changes', async () => {
  const write = vi.fn().mockRejectedValue(new Error('冲突：请重新加载'));
  await open(write);
  expect(w.find('input[value="作者标题"]').exists()).toBe(false);
  expect(w.get('pre').text()).toBe(row.content);
  await w.get('[data-testid=content-visibility]').setValue('offline');
  expect(w.get('[data-testid=content-save]').element.disabled).toBe(true);
  await w.get('[data-testid=content-reason]').setValue('修正分类前暂停展示');
  await w.get('[data-testid=content-form]').trigger('submit'); await flushPromises();
  expect(write.mock.calls[0][0].config).toMatchObject({ revision: 0, visibility: 'offline', reason: '修正分类前暂停展示' });
  expect(write.mock.calls[0][0].config).not.toHaveProperty('content'); expect(write.mock.calls[0][0].config).not.toHaveProperty('title');
  expect(w.text()).toContain('冲突'); expect(w.vm.hasUnsavedChanges).toBe(true);
});
it('protects drafts and only restores trash to offline', async () => {
  await open(vi.fn(), { ...row, visibility: 'trashed' });
  expect(w.get('[data-testid=content-visibility]').text()).not.toContain('已上架');
  await w.get('[data-testid=content-excerpt]').setValue('新摘要');
  window.confirm.mockReturnValue(false);
  const close = w.findAll('button').find(b => b.text() === '关闭'); await close.trigger('click');
  expect(w.find('[aria-label="内容管理详情"]').exists()).toBe(true);
  expect(w.vm.hasUnsavedChanges).toBe(true);
});
it('blocks repeat saves and does not claim success for an unconfirmed response', async () => {
  let resolve; const write = vi.fn(() => new Promise(done => { resolve = done; }));
  await open(write); await w.get('[data-testid=content-excerpt]').setValue('新摘要');
  await w.get('[data-testid=content-form]').trigger('submit');
  expect(w.vm.isBusy).toBe(true); expect(write).toHaveBeenCalledTimes(1);
  resolve({}); await flushPromises(); expect(w.text()).toContain('服务端未确认保存'); expect(w.vm.hasUnsavedChanges).toBe(true);
});
it('keeps filters applied to pagination and handles details load failure', async () => {
  const transport = vi.fn(request => request.kind === 'content' ? { items: [row], total: 26 } : Promise.reject(new Error('详情失败')));
  setAdminApiTransport(transport); w=mount(ContentManagement); await flushPromises();
  await w.get('[data-testid=content-query]').setValue('尚未查询'); await w.get('[data-testid=content-next]').trigger('click'); await flushPromises();
  expect(transport.mock.lastCall[0].query).toMatchObject({ q: '', offset: 25 });
  await w.get('[data-testid=content-edit]').trigger('click'); await flushPromises();
  expect(w.text()).toContain('详情失败'); expect(w.find('[data-testid=content-save]').exists()).toBe(false);
});

it('saves display fields and reloads the confirmed revision without leaving a dirty draft', async () => {
  let current = { ...row };
  const writes = [];
  setAdminApiTransport(request => {
    if (request.kind === 'content') return { items: [current], total: 1, categories: [] };
    if (request.kind === 'contentDetail') return { ...current };
    writes.push(request.config); current = { ...current, ...request.config, revision: 1 };
    return { updated: true, revision: 1 };
  });
  w = mount(ContentManagement); await flushPromises();
  await w.get('[data-testid=content-edit]').trigger('click'); await flushPromises();
  await w.get('[data-testid=content-excerpt]').setValue('优化展示摘要');
  await w.get('[data-testid=content-form]').trigger('submit'); await flushPromises();
  expect(writes).toHaveLength(1); expect(writes[0]).toMatchObject({ excerpt: '优化展示摘要', revision: 0, model: null });
  expect(w.text()).toContain('变更已保存'); expect(w.text()).toContain('版本 1');
  expect(w.vm.hasUnsavedChanges).toBe(false); expect(w.get('pre').text()).toBe(row.content);
});
