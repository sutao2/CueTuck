import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import { resetMemoryLibrary, listLocalPrompts, deleteLocalPrompt } from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setSquareTransport, setSquareContentTransport } from '../platform/square.js';
let w;
beforeEach(() => {
  resetMemoryLibrary(); resetMemorySession(); resetSquare();
  setSquareTransport(async()=>[{id:'test-download',title:'下载测试',kind:'prompt'}]);
  setSquareContentTransport(async id=>({id,title:'下载测试',content:'正文'}));
});
afterEach(()=>w?.unmount());
async function openSquare() { await flushPromises(); await w.get('[data-space="square"]').trigger('click'); await flushPromises(); }
it('shows a visible toast, updates counts and restores downloaded state after remount', async () => {
  w=mount(WorkbenchShell); await openSquare();
  await w.get('[data-testid="download-square"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="download-notice"]').text()).toContain('已下载到本地');
  expect(w.get('[data-testid="download-square"]').text()).toBe('打开本地副本');
  expect(w.get('[data-testid="download-square"]').element.disabled).toBe(false);
  expect(w.emitted('library-changed').at(-1)).toEqual([1]);
  await w.get('.prompt-card').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="square-detail-download"]').text()).toBe('打开本地副本');
  expect(w.get('[data-testid="square-detail-download"]').element.disabled).toBe(false);
  await w.get('[data-testid="download-notice"] button').trigger('click'); await flushPromises();
  expect(w.get('[data-space="local"]').attributes('aria-selected')).toBe('true');
  w.unmount(); w=mount(WorkbenchShell); await openSquare();
  expect(w.get('[data-testid="download-square"]').text()).toBe('打开本地副本');
  const [row]=await listLocalPrompts(); await deleteLocalPrompt(row.id);
  await w.get('[data-space="local"]').trigger('click'); await flushPromises(); await openSquare();
  expect(w.get('[data-testid="download-square"]').element.disabled).toBe(false);
});
it('shows busy and error feedback, then permits retry without duplicate copies', async () => {
  let reject;
  const content=vi.fn().mockImplementationOnce(()=>new Promise((_,r)=>{reject=r;})).mockResolvedValue({id:'test-download',title:'下载测试',content:'正文'});
  setSquareContentTransport(content); w=mount(WorkbenchShell); await openSquare();
  await w.get('[data-testid="download-square"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="download-square"]').text()).toBe('下载中…');
  expect(w.get('[data-testid="download-square"]').element.disabled).toBe(true);
  reject(Error('连接失败')); await flushPromises();
  expect(w.get('[data-testid="download-notice"]').text()).toContain('下载失败');
  expect(w.get('[data-testid="download-square"]').element.disabled).toBe(false);
  await w.get('[data-testid="download-square"]').trigger('click'); await flushPromises();
  expect(await listLocalPrompts()).toHaveLength(1);
});
