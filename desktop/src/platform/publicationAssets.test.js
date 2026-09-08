import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { webcrypto } from 'node:crypto';
import { mount, flushPromises } from '@vue/test-utils';
import WorkbenchShell from '../components/WorkbenchShell.vue';
import PublishedAttachments from '../components/PublishedAttachments.vue';
import { assetHash } from './privateMedia.js';
import { memoryAssets } from './assets.js';
import { createLocalPrompt, listLocalPrompts, resetMemoryLibrary, setLocalSetting } from './library.js';
import { loginSession, resetMemorySession, setSessionTransport } from './session.js';
import { resetSquare, setSquareContentTransport, downloadSquareItem, setPublishTransport, setSquareTransport, setCatalogTransport, setFavoriteTransport } from './square.js';
import { publishWithQueue, listSyncQueue, flushSyncQueue } from './syncQueue.js';
let file, reference, wrapper;
beforeEach(async () => {
  vi.stubGlobal('crypto', webcrypto); resetMemoryLibrary(); resetMemorySession(); resetSquare();
  file = { id: crypto.randomUUID(), name: 'notes.txt', mime: 'text/plain', data: btoa('public notes') };
  reference = { id: file.id, media_id: `media.${crypto.randomUUID()}`, name: file.name, mime: file.mime, size: 12, sha256: await assetHash(file) };
  setSquareTransport(async () => []); setCatalogTransport(async () => ({ categories: [], models: [] })); setFavoriteTransport(async () => ({ items: [] }));
});
afterEach(() => { wrapper?.unmount(); wrapper = null; vi.unstubAllGlobals(); });
async function login() { setSessionTransport(async () => ({ email: 'a@example.com', access_token: 'token-a' })); await loginSession({ email: 'a@example.com', password: 'test' }); }
function content(refs = [reference]) { setSquareContentTransport(async id => ({ id, title: '公开附件', content: '正文', asset_refs: refs })); }

it('verifies all files before importing, retries after corrupt bytes and prevents duplicates', async () => {
  content(); const fetcher = vi.fn(async () => new Response('broken notes')); vi.stubGlobal('fetch', fetcher);
  await expect(downloadSquareItem('pub-test')).rejects.toThrow('校验失败');
  expect(await listLocalPrompts()).toHaveLength(0);
  fetcher.mockImplementation(async () => new Response('public notes'));
  await Promise.all([downloadSquareItem('pub-test'), downloadSquareItem('pub-test')]);
  const rows = await listLocalPrompts(); expect(rows).toHaveLength(1);
  expect(memoryAssets(rows[0].id)[0]).toMatchObject({ name: file.name, data: file.data });
  expect(memoryAssets(rows[0].id)[0].id).not.toBe(file.id);
  await downloadSquareItem('pub-test'); expect(fetcher).toHaveBeenCalledTimes(2);
  expect(fetcher.mock.lastCall[0]).toBe(`http://127.0.0.1:8787/v1/square/items/pub-test/assets/${file.id}`);
});
it('does not leave an imported prompt when the second attachment fails', async () => {
  content([reference, { ...reference, id: crypto.randomUUID() }]);
  vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(new Response('public notes')).mockResolvedValueOnce(new Response('denied', { status: 404 })));
  await expect(downloadSquareItem('pub-test')).rejects.toThrow('404'); expect(await listLocalPrompts()).toHaveLength(0);
});
it('rejects malformed references instead of silently downloading text only', async () => {
  content({ fake: true }); await expect(downloadSquareItem('pub-test')).rejects.toThrow('引用'); expect(await listLocalPrompts()).toHaveLength(0);
});
it('preserves selected references in offline publication retries without file bytes', async () => {
  await login(); await setLocalSetting('auto_sync_queue', '1'); setPublishTransport(async () => { throw new Error('offline'); });
  await publishWithQueue({ sourceId: 'local', title: '正文', assetRefs: [reference] });
  const jobs = await listSyncQueue(); expect(jobs[0].assetRefs).toEqual([reference]); expect(JSON.stringify(jobs)).not.toContain(file.data);
  const publish = vi.fn(async () => ({ id: 'pub', status: 'pending' })); setPublishTransport(publish); await flushSyncQueue();
  expect(publish.mock.calls[0][0].assetRefs).toEqual([reference]); expect(await listSyncQueue()).toHaveLength(0);
});
it('defaults to no public files, clears choices on source changes and retains choices on upload failure', async () => {
  await login(); const first = await createLocalPrompt({ title: '有文件', content: '正文', assets: [file] });
  const second = await createLocalPrompt({ title: '没有文件', content: '正文' });
  const publish = vi.fn(async () => ({ id: 'pub', status: 'pending' })); setPublishTransport(publish);
  const fetcher = vi.fn(async () => new Response('failed', { status: 503 })); vi.stubGlobal('fetch', fetcher);
  wrapper = mount(WorkbenchShell); await flushPromises();
  await wrapper.get('[data-space="square"]').trigger('click'); await flushPromises();
  await wrapper.get('[data-testid="publish-prompt"]').trigger('click'); await flushPromises();
  await wrapper.get('[data-testid="publish-source"]').setValue(first.id); await flushPromises();
  expect(wrapper.get('[data-testid="publish-asset"]').element.checked).toBe(false); expect(fetcher).not.toHaveBeenCalled();
  await wrapper.get('[data-testid="publish-asset"]').setValue(true);
  await wrapper.get('[data-testid="publish-source"]').setValue(second.id); await flushPromises();
  await wrapper.get('[data-testid="publish-source"]').setValue(first.id); await flushPromises();
  expect(wrapper.get('[data-testid="publish-asset"]').element.checked).toBe(false);
  await wrapper.get('[data-testid="publish-asset"]').setValue(true);
  await wrapper.get('[data-testid="publish-submit"]').trigger('click'); await flushPromises();
  expect(wrapper.text()).toContain('发布失败'); expect(wrapper.get('[data-testid="publish-asset"]').element.checked).toBe(true); expect(publish).not.toHaveBeenCalled();
  fetcher.mockImplementation(async () => Response.json({ ...reference, id: reference.media_id }));
  await wrapper.get('[data-testid="publish-submit"]').trigger('click');
  await vi.waitFor(() => expect(publish).toHaveBeenCalledTimes(1));
  expect(publish.mock.calls[0][0].assetRefs).toEqual([reference]);
});
it('only loads previews on explicit request and renders text as inert text', async () => {
  const fetcher = vi.fn(async () => new Response('public notes')); vi.stubGlobal('fetch', fetcher);
  wrapper = mount(PublishedAttachments, { props: { itemId: 'pub-test', references: [reference] } });
  expect(fetcher).not.toHaveBeenCalled(); await wrapper.get('.file-row button').trigger('click');
  await vi.waitFor(() => expect(wrapper.get('pre').text()).toBe('public notes'));
  await wrapper.setProps({ itemId: 'another' }); expect(wrapper.find('pre').exists()).toBe(false);
});
