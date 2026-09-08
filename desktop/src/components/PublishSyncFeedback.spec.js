import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import SettingsModal from './SettingsModal.vue';
import * as library from '../platform/library.js';
import * as sync from '../platform/librarySync.js';
import * as queue from '../platform/syncQueue.js';
import { getSession, loginSession, resetMemorySession, setSessionTransport } from '../platform/session.js';
import { resetSquare, setCatalogTransport, setSquareTransport, setPublishTransport } from '../platform/square.js';

let w;
beforeEach(async () => {
  library.resetMemoryLibrary(); resetMemorySession(); resetSquare();
  setCatalogTransport(async () => ({ categories: [], models: [] })); setSquareTransport(async () => []);
  setSessionTransport(async ({ email }) => ({ email, access_token: email }));
  await loginSession({ email: 'a@example.test', password: 'test' });
});
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
async function shell() {
  w = mount(WorkbenchShell); await flushPromises();
  await w.get('[data-space=square]').trigger('click'); await flushPromises();
}
async function publish() { await w.findAll('button').find(b => b.text().includes('发布提示词')).trigger('click'); await flushPromises(); }
async function settings() { w = mount(SettingsModal, { props: { session: getSession(), initialPage: 'sync' } }); await flushPromises(); }

it('opens the publish page immediately, ignores late source reads after return, and retries failures', async () => {
  await library.createLocalPrompt({ title: 'source', content: 'body' }); await shell();
  let resolve;
  vi.spyOn(library, 'listLocalCollections').mockImplementationOnce(() => new Promise(yes => { resolve = yes; }));
  await publish();
  expect(w.get('[data-testid=publish-resume]').text()).toContain('正在读取本地内容');
  expect(w.get('[data-testid=publish-submit]').element.disabled).toBe(true);
  await w.get('[data-testid=publish-resume] .page-back').trigger('click');
  vi.spyOn(library, 'listLocalPrompts').mockRejectedValueOnce(Error('disk failed'));
  await publish();
  resolve([{ id: 'stale', title: 'stale source' }]); await flushPromises();
  expect(w.get('[data-testid=publish-resume]').text()).toContain('本地内容读取失败');
  expect(w.text()).not.toContain('stale source');
  await w.get('[data-testid=retry-publish-sources]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=publish-source]').text()).toContain('source');
});

it('preserves selection on rejected publication and keeps success visible after leaving the page', async () => {
  const source = await library.createLocalPrompt({ title: 'source', content: 'body' }); await shell(); await publish();
  await w.get('[data-testid=publish-source]').setValue(source.id); await flushPromises();
  setPublishTransport(async () => { throw Error('offline'); });
  await w.get('[data-testid=publish-submit]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=publish-source]').element.value).toBe(source.id);
  expect(w.get('[data-testid=publish-resume]').text()).toContain('发布失败');
  setPublishTransport(async () => ({ status: 'pending' }));
  await w.get('[data-testid=publish-submit]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid=publish-resume]').exists()).toBe(false);
  expect(w.get('[data-testid=publish-notice]').text()).toContain('已提交审核');
});

it('does not report an already submitted publication as failed when local queue cleanup fails', async () => {
  const send = vi.fn().mockResolvedValue({ status: 'pending' }); setPublishTransport(send);
  vi.spyOn(library, 'setLocalSetting').mockRejectedValueOnce(Error('disk full'));
  const result = await queue.publishWithQueue({ sourceId: 'p', title: 'sent', content: 'body' });
  expect(result).toMatchObject({ queued: false, queueWarning: 'disk full' }); expect(send).toHaveBeenCalledTimes(1);
});

it('reports own pending jobs only and retries the queue without syncing the library twice', async () => {
  const librarySync = vi.spyOn(sync, 'syncLocalLibraryNow').mockResolvedValue({});
  const flush = vi.spyOn(queue, 'flushSyncQueue').mockResolvedValueOnce([{ email: 'a@example.test' }, { email: 'b@example.test' }]).mockResolvedValueOnce([{ email: 'b@example.test' }]);
  await settings(); await w.get('[data-testid=sync-now]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=sync-note]').text()).toContain('个人库已同步；队列仍有 1 项未送达');
  await w.get('[data-testid=retry-sync-queue]').trigger('click'); await flushPromises();
  expect(librarySync).toHaveBeenCalledTimes(1); expect(flush).toHaveBeenCalledTimes(2);
  expect(w.get('[data-testid=sync-note]').text()).toContain('队列已处理完成');
});

it('blocks closing settings and repeat clicks while syncing, and rejects old-account success', async () => {
  let resolve;
  const librarySync = vi.spyOn(sync, 'syncLocalLibraryNow').mockImplementation(() => new Promise(yes => { resolve = yes; }));
  const flush = vi.spyOn(queue, 'flushSyncQueue').mockResolvedValue([]);
  await settings(); await w.get('[data-testid=sync-now]').trigger('click'); await w.get('[data-testid=sync-now]').trigger('click');
  w.vm.requestClose(); expect(w.emitted('cancel')).toBeUndefined(); expect(librarySync).toHaveBeenCalledTimes(1);
  await loginSession({ email: 'b@example.test', password: 'test' }); await w.setProps({ session: getSession() });
  resolve({}); await flushPromises(); expect(flush).not.toHaveBeenCalled();
  expect(w.get('[data-testid=sync-note]').text()).toContain('登录状态已改变');
});

it('keeps successful library feedback when queue persistence throws', async () => {
  vi.spyOn(sync, 'syncLocalLibraryNow').mockResolvedValue({}); vi.spyOn(queue, 'flushSyncQueue').mockRejectedValue(Error('queue unavailable'));
  await settings(); await w.get('[data-testid=sync-now]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=sync-note]').text()).toContain('个人库已同步；队列处理失败');
  expect(w.find('[data-testid=retry-sync-queue]').exists()).toBe(true);
});
