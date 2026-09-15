import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import SettingsModal from './SettingsModal.vue';
import MyPublications from './MyPublications.vue';
import {setSkillMarketTransport} from '../platform/skillMarket.js';
import {setSkillsTransportForTests} from '../platform/skills.js';
import WorkbenchShell from './WorkbenchShell.vue';
import { resetMemoryLibrary, listLocalPrompts } from '../platform/library.js';
import { resetMemorySession, setSessionTransport, loginSession } from '../platform/session.js';
import * as square from '../platform/square.js';
import * as tauri from '../platform/tauri.js';
let w;
beforeEach(() => { resetMemoryLibrary(); resetMemorySession(); square.resetSquare(); });
afterEach(() => { w?.unmount();setSkillMarketTransport(null);setSkillsTransportForTests(null); vi.restoreAllMocks(); delete window.__TAURI_INTERNALS__; });
it('previews a dropped file and writes only after confirmation; rejects invalid files', async () => {
  w = mount(SettingsModal, { props: { initialPage: 'data' } }); await flushPromises();
  const file = new File(['{"prompts":[{"title":"文件导入","content":"原文"}]}'], 'prompts.json', { type: 'application/json' });
  await w.get('[data-testid=import-drop]').trigger('drop', { dataTransfer: { files: [file] } });
  await vi.waitFor(() => expect(w.find('[data-testid=import-preview]').exists()).toBe(true));
  expect(await listLocalPrompts()).toHaveLength(0);
  await w.findAll('button').find(b => b.text() === '确认导入').trigger('click'); await flushPromises();
  expect((await listLocalPrompts())[0].content).toBe('原文');
  await w.get('[data-testid=import-drop]').trigger('drop', { dataTransfer: { files: [new File(['broken'], 'invalid.json')] } });
  await vi.waitFor(() => expect(w.get('[data-testid=backup-error]').text()).toContain('读取文件失败'));
  expect(await listLocalPrompts()).toHaveLength(1);
});
it('keeps the restore path when the native file picker is cancelled', async () => {
  w = mount(SettingsModal); await flushPromises();
  window.__TAURI_INTERNALS__ = {};
  const choose = vi.spyOn(tauri, 'invokeCommand').mockResolvedValueOnce(null).mockResolvedValueOnce('/tmp/selected.sqlite');
  await w.get('[data-settings-page=data]').trigger('click');
  const path = w.get('input[placeholder="/path/to/promptark.sqlite"]');
  await path.setValue('/tmp/original.sqlite');
  await w.get('[data-testid=choose-restore]').trigger('click'); await flushPromises();
  expect(path.element.value).toBe('/tmp/original.sqlite');
  await w.get('[data-testid=choose-restore]').trigger('click'); await flushPromises();
  expect(path.element.value).toBe('/tmp/selected.sqlite');
  expect(choose.mock.calls.map(c => c[0])).toEqual(['choose_library_backup', 'choose_library_backup']);
});
it('isolates publication responses across accounts and offers a retry on failure', async () => {
  let resolve;
  vi.spyOn(square, 'listMyPublications').mockImplementationOnce(() => new Promise(r => { resolve = r; })).mockRejectedValueOnce(Error('离线')).mockResolvedValueOnce([{ id: 'b', title: 'B 的稿件', status: 'pending' }]);
  w = mount(MyPublications, { props: { session: { email: 'a', loggedIn: true } } });
  await w.setProps({ session: { email: 'b', loggedIn: true } }); await flushPromises();
  resolve([{ id: 'a', title: 'A 的稿件' }]); await flushPromises();
  expect(w.text()).not.toContain('A 的稿件');
  expect(w.text()).toContain('读取失败');
  await w.findAll('button').find(b => b.text() === '重试').trigger('click'); await flushPromises();
  expect(w.text()).toContain('B 的稿件');
});
it('protects settings drafts before opening the standalone publications page', async () => {
  setSessionTransport(async () => ({ email: 'a@example.test', access_token: 'token' }));
  await loginSession({ email: 'a@example.test', password: 'pass' });
  square.setMineTransport(async () => []);
  w = mount(WorkbenchShell); await flushPromises();
  await w.get('[data-testid=open-settings]').trigger('click'); await flushPromises();
  await w.get('[data-settings-page=models]').trigger('click'); await w.get('[data-testid=default-model]').setValue('草稿');
  await w.get('[data-settings-page=account]').trigger('click'); await w.get('[data-testid=settings-publications]').trigger('click');
  expect(w.find('[data-testid=publications-page]').exists()).toBe(false);
  await w.get('[data-testid=cancel-settings-action]').trigger('click');
  await w.get('[data-settings-page=models]').trigger('click');
  expect(w.get('[data-testid=default-model]').element.value).toBe('草稿');
});

it('summarizes own metrics and sorts by recorded downloads or current favorites', async () => {
  square.setMineTransport(async () => [
    { id: 'new', title: '最新作品', download_count: 3, favorite_count: 9 },
    { id: 'hot', title: '下载最多作品', download_count: 1200, favorite_count: 2 },
  ]);
  w = mount(MyPublications, { props: { session: { email: 'a', loggedIn: true } } });
  await flushPromises();
  expect(w.get('[data-testid=publication-metrics]').text()).toContain('1,203');
  expect(w.get('[data-testid=publication-metrics]').text()).toContain('11');
  const titles = () => w.findAll('.publication-heading strong').map(n => n.text());
  expect(titles()).toEqual(['最新作品', '下载最多作品']);
  await w.get('[aria-label="作品排序"]').setValue('download_count');
  expect(titles()[0]).toBe('下载最多作品');
  await w.get('[aria-label="作品排序"]').setValue('favorite_count');
  expect(titles()[0]).toBe('最新作品');
});

it('does not invent zero totals for missing counters or failed requests', async () => {
  square.setMineTransport(async () => [{ id: 'legacy', title: '旧服务作品' }]);
  w = mount(MyPublications, { props: { session: { email: 'a', loggedIn: true } } });
  await flushPromises();
  expect(w.findAll('.publication-metrics strong').map(n => n.text())).toEqual(['1', '—', '—']);
  expect(w.get('.publication-counters').text()).toContain('记录下载 —');
  square.setMineTransport(async () => { throw new Error('offline'); });
  await w.findAll('button').find(b => b.text() === '刷新').trigger('click'); await flushPromises();
  expect(w.find('[data-testid=publication-metrics]').exists()).toBe(false);
  expect(w.text()).toContain('读取失败');
});

it('shows server metrics on square cards and explains the hot ordering without recording a view', async () => {
  const stats = vi.fn();
  square.setDownloadStatsTransport(stats);
  square.setCatalogTransport(async () => ({ categories: [], models: [] }));
  square.setSquarePageTransport(async () => ({ items: [{ id: 'counted', kind: 'prompt', title: '统计作品', download_count: 1234, favorite_count: 7 }, { id: 'legacy', kind: 'prompt', title: '旧条目' }], total: 2, next_offset: null }));
  w = mount(WorkbenchShell); await flushPromises();
  await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  const counters = w.findAll('[data-testid=square-card-metrics]');
  expect(counters[0].text()).toContain('1,234');
  expect(counters[0].text()).toContain('7');
  expect(counters[1].text()).toContain('—');
  await w.get('[data-sort="热门"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=square-sort-note]').text()).toContain('按已记录下载量从高到低');
  expect(stats).not.toHaveBeenCalled();
});

it('uses the sidebar publications entry for both prompts and Skills',async()=>{
 setSessionTransport(async()=>({email:'author@test',access_token:'test'}));await loginSession({email:'author@test',password:'test'});square.setMineTransport(async()=>[]);square.setCatalogTransport(async()=>({categories:[],models:[]}));square.setSquareTransport(async()=>[]);
 setSkillsTransportForTests(async()=>({roots:[],skills:[],backups:[],operations:[],warnings:[],sources:[]}));const market=vi.fn(async()=>({items:[],total:0,category_counts:{}}));setSkillMarketTransport(market);
 w=mount(WorkbenchShell);await flushPromises();await w.get('[data-space="skills-square"]').trigger('click');await flushPromises();expect(w.findAll('[data-testid="open-publications"]')).toHaveLength(1);expect(w.get('.community-skills').findAll('button').some(b=>b.text()==='我的发布')).toBe(false);
 await w.get('[data-testid="open-publications"]').trigger('click');await flushPromises();const nav=w.get('[aria-label="发布内容类型"]');expect(nav.findAll('button').find(b=>b.text()==='Skills').attributes('aria-current')).toBe('page');expect(market.mock.calls.some(([action])=>action==='mine')).toBe(true);
 await nav.findAll('button').find(b=>b.text()==='提示词').trigger('click');await flushPromises();expect(w.get('[data-testid="publication-metrics"]').exists()).toBe(true);
});
it('isolates Skill publications across accounts and blocks leaving while a request is pending',async()=>{
 let resolve;setSkillMarketTransport(vi.fn().mockImplementationOnce(()=>new Promise(r=>resolve=r)).mockResolvedValue({items:[],total:0,category_counts:{}}));
 w=mount(MyPublications,{props:{session:{email:'a@test',loggedIn:true},initialKind:'skills'}});await flushPromises();expect(w.get('.page-back').attributes('disabled')).toBeDefined();
 await w.setProps({session:{email:'b@test',loggedIn:true}});await flushPromises();resolve({items:[{id:'a',title:'A 的私有 Skill'}],total:1,category_counts:{}});await flushPromises();expect(w.text()).not.toContain('A 的私有 Skill');expect(w.text()).toContain('你还没有发布 Skill');expect(w.get('.page-back').attributes('disabled')).toBeUndefined();
});
