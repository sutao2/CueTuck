import { selectOption, selectComponent } from '../test/selectOption.js';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { webcrypto } from 'node:crypto';
import { mount, flushPromises } from '@vue/test-utils';
import WorkbenchShell from '../components/WorkbenchShell.vue';
import PublishedAttachments from '../components/PublishedAttachments.vue';
import SquareDetailModal from '../components/SquareDetailModal.vue';
import { assetHash } from './privateMedia.js';
import { memoryAssets } from './assets.js';
import { createLocalPrompt, createLocalCollection, addPromptToCollection, removePromptFromCollection, listCollectionMembers, listLocalCollections, listLocalPrompts, resetMemoryLibrary, setLocalSetting } from './library.js';
import { loginSession, resetMemorySession, setSessionTransport } from './session.js';
import { resetSquare, setDownloadStatsTransport, setSquareContentTransport, downloadSquareItem, setPublishTransport, setSquareTransport, setCatalogTransport, setFavoriteTransport } from './square.js';
import { publishWithQueue, listSyncQueue, flushSyncQueue } from './syncQueue.js';
let file, reference, wrapper;
beforeEach(async () => {
  vi.stubGlobal('crypto', webcrypto); resetMemoryLibrary(); resetMemorySession(); resetSquare(); setDownloadStatsTransport(async () => ({ download_count: 1 }));
  file = { id: crypto.randomUUID(), name: 'notes.txt', mime: 'text/plain', data: btoa('public notes') };
  reference = { id: file.id, media_id: `media.${crypto.randomUUID()}`, name: file.name, mime: file.mime, size: 12, sha256: await assetHash(file) };
  setSquareTransport(async () => []); setCatalogTransport(async () => ({ categories: [], models: [] })); setFavoriteTransport(async () => ({ items: [] }));
});
afterEach(() => { wrapper?.unmount(); wrapper = null; vi.unstubAllGlobals(); });
async function login() { setSessionTransport(async () => ({ email: 'a@example.com', access_token: 'token-a' })); await loginSession({ email: 'a@example.com', password: 'test' }); }
function content(refs = [reference]) { setSquareContentTransport(async id => ({ id, title: '公开附件', content: '正文', asset_refs: refs })); }

it('atomically downloads collection files to their respective members and retries without duplicates', async () => {
  const second = { ...reference, id: crypto.randomUUID(), name: 'second.txt' };
  setSquareContentTransport(async id => ({ id, title: '文件合集', kind: 'collection', asset_refs: [reference,second], members: [
    { title: '成员一', content: '正文一', asset_ids: [reference.id] }, { title: '成员二', content: '正文二', asset_ids: [second.id] },
  ] }));
  const fetcher = vi.fn().mockResolvedValueOnce(new Response('public notes')).mockResolvedValueOnce(new Response('failed', { status: 404 }));
  vi.stubGlobal('fetch', fetcher);
  await expect(downloadSquareItem('collection-test')).rejects.toThrow('404');
  expect(await listLocalPrompts()).toHaveLength(0);
  expect(await listLocalCollections()).toHaveLength(0);
  fetcher.mockImplementation(async () => new Response('public notes'));
  await Promise.all([downloadSquareItem('collection-test'), downloadSquareItem('collection-test')]);
  const rows = await listLocalPrompts(), collections = await listLocalCollections(), collection = collections[0];
  expect(rows).toHaveLength(2); expect(collections).toHaveLength(1);
  const members = await listCollectionMembers(collection.id);
  for (const [title,name] of [['成员一','notes.txt'],['成员二','second.txt']]) {
    const assets = memoryAssets(members.find(member => member.title === title).id);
    expect(assets).toHaveLength(1); expect(assets[0]).toMatchObject({ name, data: file.data });
    expect([reference.id,second.id]).not.toContain(assets[0].id);
  }
  await downloadSquareItem('collection-test'); expect(fetcher).toHaveBeenCalledTimes(4);
});
it('rejects unassigned, duplicate and unknown collection file links before fetching or writing', async () => {
  const fetcher = vi.fn(); vi.stubGlobal('fetch',fetcher);
  for (const asset_ids of [[],[reference.id,reference.id],['unknown'],'bad']) {
    setSquareContentTransport(async id => ({ id, title: 'bad', kind: 'collection', asset_refs: [reference], members: [{ title: 'member',content: 'body',asset_ids }] }));
    await expect(downloadSquareItem('invalid-collection')).rejects.toThrow('附件');
  }
  expect(fetcher).not.toHaveBeenCalled(); expect(await listLocalPrompts()).toHaveLength(0);
});
async function collectionSource() {
  await login(); const collection = await createLocalCollection({ title: '成员附件合集' });
  const first = await createLocalPrompt({ title: '成员一', content: '正文一', assets: [file] });
  const second = await createLocalPrompt({ title: '成员二', content: '正文二', assets: [{ ...file,name:'private.txt' }] });
  await addPromptToCollection(first.id,collection.id); await addPromptToCollection(second.id,collection.id);
  wrapper = mount(WorkbenchShell); await flushPromises();
  await wrapper.get('[data-space="square"]').trigger('click'); await flushPromises();
  await wrapper.get('[data-testid="publish-prompt"]').trigger('click'); await flushPromises();
  await selectOption(wrapper, 'publish-source', collection.id); await flushPromises();
  return { collection,first,second };
}
it('publishes only selected collection files with member links, preserving selection after upload failure', async () => {
  const publish = vi.fn(async () => ({ id:'pub',status:'pending' })); setPublishTransport(publish);
  const { first,second } = await collectionSource();
  const choices = wrapper.findAll('[data-testid="publish-asset"]'); expect(choices).toHaveLength(2);
  expect(choices.every(choice => !choice.element.checked)).toBe(true);
  expect(wrapper.text()).toContain('所属提示词：成员一');
  expect(new Set(choices.map(choice => choice.element.value)).size).toBe(2);
  const choice = wrapper.findAll('.publication-file').find(label => label.text().includes('notes.txt')).get('input'); await choice.setValue(true);
  const fetcher = vi.fn(async () => new Response('fail',{status:503})); vi.stubGlobal('fetch',fetcher);
  await submitPreview(wrapper); await flushPromises();
  expect(publish).not.toHaveBeenCalled(); expect(choice.element.checked).toBe(true);
  fetcher.mockImplementation(async () => Response.json({ ...reference,id:reference.media_id }));
  await submitPreview(wrapper);
  await vi.waitFor(() => expect(publish).toHaveBeenCalledTimes(1));
  const sent = publish.mock.calls[0][0]; expect(sent.assetRefs).toEqual([{...reference,id:choice.element.value}]);
  expect(sent.members.find(member => member.title === '成员一').asset_ids).toEqual([choice.element.value]);
  expect(sent.members.find(member => member.title === '成员二').asset_ids).toBeUndefined();
  expect(sent.members.every(member => !member.id && !member.assets)).toBe(true);
  expect(JSON.stringify(sent)).not.toContain('private.txt'); expect(JSON.stringify(sent)).not.toContain(file.data);
  expect(memoryAssets(first.id)).toHaveLength(1); expect(memoryAssets(second.id)).toHaveLength(1);
});
it('refuses to publish a selected file after its member was removed', async () => {
  const publish = vi.fn(), fetcher = vi.fn(); setPublishTransport(publish); vi.stubGlobal('fetch',fetcher);
  const { collection,first } = await collectionSource();
  await wrapper.findAll('.publication-file').find(label => label.text().includes('notes.txt')).get('input').setValue(true);
  await removePromptFromCollection(first.id,collection.id);
  await submitPreview(wrapper); await flushPromises();
  expect(wrapper.text()).toContain('成员已移出合集'); expect(fetcher).not.toHaveBeenCalled(); expect(publish).not.toHaveBeenCalled();
});
it('shows collection files under the matching member without automatically loading bytes', () => {
  const fetcher = vi.fn(); vi.stubGlobal('fetch',fetcher);
  wrapper = mount(SquareDetailModal,{props:{item:{id:'pub',title:'合集',kind:'collection',asset_refs:[reference],members:[
    {title:'成员一',content:'正文',asset_ids:[reference.id]}, {title:'成员二',content:'正文'},
  ]}}});
  const articles = wrapper.findAll('[data-testid="square-detail-member"]');
  expect(articles[0].text()).toContain('notes.txt'); expect(articles[1].text()).not.toContain('notes.txt');
  expect(fetcher).not.toHaveBeenCalled();
});

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
  await selectOption(wrapper, 'publish-source', first.id); await flushPromises();
  expect(wrapper.get('[data-testid="publish-asset"]').element.checked).toBe(false); expect(fetcher).not.toHaveBeenCalled();
  await wrapper.get('[data-testid="publish-asset"]').setValue(true);
  await selectOption(wrapper, 'publish-source', second.id); await flushPromises();
  await selectOption(wrapper, 'publish-source', first.id); await flushPromises();
  expect(wrapper.get('[data-testid="publish-asset"]').element.checked).toBe(false);
  await wrapper.get('[data-testid="publish-asset"]').setValue(true);
  await submitPreview(wrapper); await flushPromises();
  expect(wrapper.text()).toContain('发布失败'); expect(wrapper.get('[data-testid="publish-asset"]').element.checked).toBe(true); expect(publish).not.toHaveBeenCalled();
  fetcher.mockImplementation(async () => Response.json({ ...reference, id: reference.media_id }));
  await submitPreview(wrapper);
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
it('shows local image previews and explicitly selects images without exposing other files', async () => {
  await login();
  const image={id:crypto.randomUUID(),name:'animation.gif',mime:'image/gif',data:btoa('GIF89a-example')};
  const prompt=await createLocalPrompt({title:'Image publication',content:'body',assets:[image,file]});
  wrapper=mount(WorkbenchShell); await flushPromises();
  await wrapper.get('[data-space="square"]').trigger('click'); await flushPromises();
  await wrapper.get('[data-testid="publish-prompt"]').trigger('click'); await flushPromises();
  await selectOption(wrapper,'publish-source',prompt.id); await flushPromises();
  expect(wrapper.get('.publication-image-preview').attributes('src')).toBe('data:image/gif;base64,'+image.data);
  expect(wrapper.text()).toContain('本次发布不会包含本地图片');
  await wrapper.get('[data-testid="select-publish-images"]').trigger('click');
  const choices=wrapper.findAll('[data-testid="publish-asset"]');
  expect(choices.filter(c=>c.element.checked)).toHaveLength(1);
  expect(choices.find(c=>c.element.checked).element.value).toBe(image.id);
  expect(wrapper.text()).toContain('已选择 1 张图片');
});

async function submitPreview(wrapper) {
 const preview=wrapper.find('[data-testid="publish-submit"]');
 if(preview.exists()){await preview.trigger('click');await flushPromises();}
 const confirm=wrapper.find('[data-testid="publish-confirm"]');
 if(confirm.exists()){await confirm.trigger('click');await flushPromises();}
}

it('publishes ordered collection covers independently of private member attachments and restores them on download', async () => {
  const { collection } = await collectionSource();
  const { updateLocalCollection } = await import('./library.js');
  const png = { id: crypto.randomUUID(), name: 'cover-1.png', mime: 'image/png', data: btoa('\x89PNG\r\n\x1a\nfixture') };
  const dataUrl = `data:${png.mime};base64,${png.data}`;
  await updateLocalCollection({ id: collection.id, title: collection.title, coverType: 'grid', coverUrls: [dataUrl] });
  wrapper.unmount(); wrapper = mount(WorkbenchShell); await flushPromises();
  await wrapper.get('[data-space="square"]').trigger('click'); await flushPromises();
  await wrapper.get('[data-testid="publish-prompt"]').trigger('click'); await flushPromises();
  await selectOption(wrapper, 'publish-source', collection.id); await flushPromises();
  expect(wrapper.get('[data-testid="publish-cover"]').element.checked).toBe(true);
  const pngRef = { ...reference, id: png.id, name: png.name, mime: png.mime, size: atob(png.data).length, sha256: await assetHash(png) };
  const fetcher = vi.fn(async () => Response.json({ ...pngRef, id: pngRef.media_id })); vi.stubGlobal('fetch', fetcher);
  const publish = vi.fn(async () => ({ id: 'covered', status: 'pending' })); setPublishTransport(publish);
  await submitPreview(wrapper); await vi.waitFor(() => expect(publish).toHaveBeenCalledTimes(1));
  const sent = publish.mock.calls[0][0];
  expect(sent.cover.layout).toBe('grid'); expect(sent.cover.asset_ids).toEqual(sent.assetRefs.map(a => a.id));
  expect(sent.assetRefs).toHaveLength(1); expect(sent.members.every(m => !m.asset_ids)).toBe(true);
  expect(JSON.stringify(sent)).not.toContain('private.txt'); expect(JSON.stringify(sent)).not.toContain('base64');
  setSquareContentTransport(async id => ({ id, title: '下载封面', kind: 'collection', cover: sent.cover, asset_refs: sent.assetRefs, members: sent.members }));
  fetcher.mockImplementation(async () => new Response(Uint8Array.from(atob(png.data), c => c.charCodeAt(0))));
  await downloadSquareItem('covered');
  const downloaded = (await listLocalCollections()).find(c => c.title === '下载封面');
  expect(downloaded).toMatchObject({ cover_type: 'grid', cover_json: JSON.stringify([dataUrl]) });
  expect((await listCollectionMembers(downloaded.id)).every(m => memoryAssets(m.id).length === 0)).toBe(true);
});

it('keeps the collection cover manifest when replaying a queued publication', async () => {
  await login(); await setLocalSetting('auto_sync_queue', '1');
  const ref = { ...reference, name:'cover.png', mime:'image/png' };
  const cover = { layout:'single',asset_ids:[ref.id] };
  setPublishTransport(async () => { throw Error('offline'); });
  await publishWithQueue({sourceId:'covered-queue',title:'封面',kind:'collection',members:[{title:'成员',content:'正文'}],assetRefs:[ref],cover});
  const sent=vi.fn(async()=>({id:'queued'}));setPublishTransport(sent);
  await flushSyncQueue();
  expect(sent.mock.calls[0][0].cover).toEqual(cover);
  expect(await listSyncQueue()).toHaveLength(0);
});
