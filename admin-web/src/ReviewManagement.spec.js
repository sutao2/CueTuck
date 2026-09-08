import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import ReviewManagement from './ReviewManagement.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';
let wrapper;
const item = (id = 'a') => ({ id, source_id: id, title: `投稿 ${id}`, status: 'pending', content: '正文' });
it('shows selected files and requires confirmation before approving a file publication', async () => {
  const transport = vi.fn(request => request.kind === 'list' ? { items: [{ ...item(), asset_refs: [{ id: 'file', name: 'notes.txt', mime: 'text/plain', size: 12 }] }] } : { status: 'approved' });
  const w = await open(transport);
  expect(w.text()).toContain('notes.txt'); expect(w.text()).toContain('文本自动审核不验证文件安全');
  await w.get('[data-testid=review-approve]').trigger('click'); await flushPromises();
  expect(transport).toHaveBeenCalledTimes(1);
  const confirm = vi.spyOn(window, 'confirm');
  await w.get('[data-testid=review-confirm]').trigger('click'); await flushPromises();
  expect(transport).toHaveBeenCalledTimes(2); expect(confirm).not.toHaveBeenCalled();
  expect(w.find('[data-testid=review-approve]').exists()).toBe(false);
});
beforeEach(() => resetAdminApi());
afterEach(() => { wrapper?.unmount(); vi.restoreAllMocks(); });
async function open(transport) { setAdminApiTransport(transport); wrapper = mount(ReviewManagement); await flushPromises(); return wrapper; }
it('requires a reason, retains it on failure, and sends trimmed reason on retry', async () => {
  const write = vi.fn().mockRejectedValueOnce(new Error('保存失败')).mockResolvedValue({ status: 'rejected' });
  const w = await open(request => request.kind === 'list' ? { items: [item()] } : write(request));
  await w.get('[data-testid=review-reject]').trigger('click');
  expect(w.get('[data-testid=review-confirm]').element.disabled).toBe(true);
  await w.get('[data-testid=review-reason]').setValue('  请完善说明  ');
  expect(w.vm.hasUnsavedChanges).toBe(true);
  await w.get('[data-testid=review-confirm]').trigger('click'); await flushPromises();
  expect(w.text()).toContain('保存失败'); expect(w.get('textarea').element.value).toContain('请完善说明');
  await w.get('[data-testid=review-confirm]').trigger('click'); await flushPromises();
  expect(write.mock.calls[1][0].reason).toBe('请完善说明'); expect(w.vm.hasUnsavedChanges).toBe(false);
  expect(w.find('[data-testid=review-reject]').exists()).toBe(false);
});
it('confirms a batch and displays individual failures instead of claiming all succeeded', async () => {
  const w = await open(request => request.kind === 'list' ? { items: [item('a'), item('b')] } : { results: [{ id: 'a', ok: true, status: 200 }, { id: 'b', ok: false, status: 409, message: '已被处理' }] });
  await w.get('.review-batch input').setValue(true);
  await w.get('[data-testid=review-batch-approve]').trigger('click');
  expect(w.text()).toContain('批量通过 · 2 条');
  await w.get('[data-testid=review-confirm]').trigger('click'); await flushPromises();
  expect(w.text()).toContain('成功 1 条，失败 1 条'); expect(w.text()).toContain('已被处理（409）');
  expect(w.findAll('[data-testid=review-approve]')).toHaveLength(1);
});
it('paginates using applied filters, not unsubmitted drafts, and shows historical missing time', async () => {
  const transport = vi.fn(() => ({ items: [item()], total: 51 }));
  const w = await open(transport);
  await w.get('[data-testid=review-search]').setValue('尚未查询');
  await w.get('[data-testid=reviews-next]').trigger('click'); await flushPromises();
  expect(transport.mock.lastCall[0].query).toMatchObject({ q: '', offset: 25, limit: 25 });
  expect(w.text()).toContain('历史时间未记录');
});
it('protects a rejection draft when refreshing and blocks duplicate batch submission', async () => {
  let finish;
  const transport = vi.fn(request => request.kind === 'list' ? { items: [item()] } : new Promise(resolve => { finish = resolve; }));
  const w = await open(transport);
  await w.get('[data-testid=review-reject]').trigger('click');
  await w.get('textarea').setValue('草稿');
  vi.spyOn(window, 'confirm').mockReturnValue(false);
  await w.get('form').trigger('submit'); await flushPromises();
  expect(transport).toHaveBeenCalledTimes(1); expect(w.get('textarea').element.value).toBe('草稿');
  await w.get('[data-testid=review-confirm]').trigger('click');
  expect(w.vm.isBusy).toBe(true); expect(w.get('[data-testid=review-approve]').element.disabled).toBe(true);
  finish({ status: 'rejected' }); await flushPromises(); expect(w.vm.isBusy).toBe(false);
});
it('does not remove records for an incomplete batch response', async () => {
  const w = await open(request => request.kind === 'list' ? { items: [item('a'),item('b')] } : { results: [{ id: 'a', ok: true }] });
  await w.get('.review-batch input').setValue(true); await w.get('[data-testid=review-batch-approve]').trigger('click');
  await w.get('[data-testid=review-confirm]').trigger('click'); await flushPromises();
  expect(w.text()).toContain('批量响应不完整'); expect(w.findAll('[data-testid=review-approve]')).toHaveLength(2);
});
