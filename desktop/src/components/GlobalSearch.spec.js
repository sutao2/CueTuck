import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import GlobalSearch from './GlobalSearch.vue';
import { createLocalCollection, createLocalPrompt, resetMemoryLibrary, setLocalSetting } from '../platform/library.js';
import { resetSquare, setSquarePageTransport } from '../platform/square.js';

let w;
beforeEach(() => { resetMemoryLibrary(); resetSquare(); vi.useFakeTimers(); });
afterEach(() => { w?.unmount(); vi.useRealTimers(); });
async function query(value) {
  await w.get('input').setValue(value);
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
}
it('searches all local prompts and collections without contacting the square, bounds results and supports keyboard selection', async () => {
  const remote = vi.fn(); setSquarePageTransport(remote);
  await createLocalPrompt({ title: '目标提示词', content: '正文', categoryId: 'cat-software' });
  await createLocalCollection({ title: '目标合集', categoryId: 'cat-portrait' });
  w = mount(GlobalSearch);
  expect(w.text()).toContain('输入关键词开始搜索');
  await query('目标');
  expect(w.findAll('[role="option"]')).toHaveLength(2);
  expect(w.text()).toContain('目标合集');
  expect(remote).not.toHaveBeenCalled();
  await w.get('input').trigger('keydown', { key: 'ArrowDown' });
  expect(w.get('input').attributes('aria-activedescendant')).toBe('global-result-1');
  await w.get('input').trigger('keydown', { key: 'Enter', isComposing: true });
  expect(w.emitted('select')).toBeUndefined();
  await w.get('input').trigger('keydown', { key: 'Enter' });
  expect(w.emitted('select')[0][0]).toMatchObject({ scope: 'local', item: { title: '目标合集', kind: 'collection' } });
  for (let i = 0; i < 60; i++) await createLocalPrompt({ title: `目标 ${i}`, content: '长文'.repeat(10000) });
  await query('目标 ');
  expect(w.findAll('[role="option"]')).toHaveLength(48);
  expect(w.text()).toContain('请细化关键词');
  expect(w.get('.global-search-result-copy small').text().length).toBeLessThanOrEqual(240);
  await query('不存在'); expect(w.text()).toContain('没有找到匹配内容');
});
it('debounces composed input and explicitly searches the square without page filters', async () => {
  const remote = vi.fn(async () => ({ items: [{ id: 'remote', kind: 'prompt', title: '远程结果' }], total: 80, next_offset: 48 }));
  setSquarePageTransport(remote); w = mount(GlobalSearch);
  await w.get('[data-search-scope="square"]').trigger('click');
  await vi.advanceTimersByTimeAsync(300); expect(remote).not.toHaveBeenCalled();
  await w.get('input').trigger('compositionstart');
  await w.get('input').setValue('产品');
  await vi.advanceTimersByTimeAsync(300); expect(remote).not.toHaveBeenCalled();
  await w.get('input').trigger('compositionend');
  await vi.advanceTimersByTimeAsync(249); expect(remote).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1); await flushPromises();
  expect(remote).toHaveBeenCalledOnce();
  expect(remote.mock.calls[0][0]).toMatchObject({ query: '产品', categoryId: null, model: '', offset: 0 });
  expect(w.text()).toContain('找到 80 条');
  await w.get('[role="option"]').trigger('click');
  expect(w.emitted('select')[0][0]).toMatchObject({ scope: 'square', item: { id: 'remote' } });
});
it('honors disabled square access and retries a failed search', async () => {
  const remote = vi.fn().mockRejectedValueOnce(Error('离线')).mockResolvedValue({ items: [], total: 0, next_offset: null });
  setSquarePageTransport(remote); await setLocalSetting('square_access', '0');
  w = mount(GlobalSearch); await w.get('[data-search-scope="square"]').trigger('click'); await query('测试');
  expect(w.text()).toContain('广场访问已关闭'); expect(remote).not.toHaveBeenCalled();
  await setLocalSetting('square_access', '1'); await query('重试');
  expect(w.get('[role="alert"]').text()).toContain('离线');
  await w.get('[role="alert"] button').trigger('click'); await flushPromises();
  expect(w.text()).toContain('没有找到匹配内容'); expect(remote).toHaveBeenCalledTimes(2);
});
it('aborts old requests on scope changes and unmount, and ignores late responses', async () => {
  let resolveOld; const remote = vi.fn(() => new Promise(resolve => { resolveOld = resolve; }));
  setSquarePageTransport(remote); w = mount(GlobalSearch);
  await w.get('[data-search-scope="square"]').trigger('click'); await query('远程');
  const signal = remote.mock.calls[0][0].signal;
  await w.get('[data-search-scope="local"]').trigger('click'); expect(signal.aborted).toBe(true);
  resolveOld({ items: [{ id: 'old', title: '不该出现' }], total: 1, next_offset: null });
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
  expect(w.text()).not.toContain('不该出现'); expect(w.findAll('[role="option"]')).toHaveLength(0);
  await w.get('[data-search-scope="square"]').trigger('click'); await vi.advanceTimersByTimeAsync(250);
  const lastSignal = remote.mock.calls.at(-1)[0].signal; w.unmount(); expect(lastSignal.aborted).toBe(true);
});
