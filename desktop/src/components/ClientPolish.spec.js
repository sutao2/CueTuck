import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import CreatePromptModal from './CreatePromptModal.vue';
import * as library from '../platform/library.js';
import * as session from '../platform/session.js';
import { resetSquare, setCatalogTransport, setSquareTransport } from '../platform/square.js';

let w;
beforeEach(() => {
  library.resetMemoryLibrary(); session.resetMemorySession(); resetSquare();
  setCatalogTransport(async () => ({ categories: [], models: [] }));
  setSquareTransport(async () => []);
});
afterEach(() => { w?.unmount(); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); document.body.classList.remove('theme-dark'); });
async function shell() { w = mount(WorkbenchShell, { attachTo: document.body }); await flushPromises(); return w; }

it('routes the signed-in account to existing account settings, then returns to login after logout', async () => {
  session.setSessionTransport(async () => ({ access_token: 'test-access', email: 'tester@example.test' }));
  await session.loginSession({ email: 'tester@example.test', password: 'test-only' });
  await library.createLocalPrompt({ title: '保留内容', content: '本地正文' });
  await shell();
  await w.get('[data-testid="open-login"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="login-modal"]').exists()).toBe(false);
  expect(w.get('[data-testid="current-account"]').text()).toBe('tester@example.test');
  await w.get('[data-testid="settings-logout"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="current-account"]').text()).toBe('未登录');
  expect((await library.listLocalPrompts())[0].title).toBe('保留内容');
  await w.get('[data-testid="settings-login"]').trigger('click');
  expect(w.find('[data-testid="login-email"]').exists()).toBe(true);
});

it('keeps account information and shows logout failure while preventing duplicate attempts', async () => {
  session.setSessionTransport(async () => ({ access_token: 'test-access', email: 'tester@example.test' }));
  await session.loginSession({ email: 'tester@example.test', password: 'test-only' });
  let reject;
  const logout = vi.spyOn(session, 'logoutSession').mockImplementation(() => new Promise((_, fail) => { reject = fail; }));
  await shell(); await w.get('[data-testid="open-login"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="settings-logout"]').trigger('click');
  expect(w.get('[data-testid="settings-logout"]').attributes('disabled')).toBeDefined();
  expect(logout).toHaveBeenCalledTimes(1);
  reject(new Error('退出失败，请重试')); await flushPromises();
  expect(w.text()).toContain('退出失败，请重试');
  expect(w.get('[data-testid="current-account"]').text()).toBe('tester@example.test');
});

it('persists layout only after loading and restores it on remount', async () => {
  const saved = JSON.stringify({width:320,collapsed:true,view:'list'});
  await library.setLocalSetting('workbench_layout', saved);
  await shell();
  expect(await library.getLocalSetting('workbench_layout')).toBe(saved);
  expect(w.get('.sidebar').isVisible()).toBe(false);
  expect(w.attributes('style')).toContain('320px');
  expect(w.get('[title="列表视图"]').attributes('aria-pressed')).toBe('true');
  await w.get('[data-testid="toggle-sidebar"]').trigger('click');
  await w.get('[role="separator"]').trigger('keydown', {key:'ArrowRight'}); await flushPromises();
  w.unmount(); await shell();
  expect(w.get('.sidebar').isVisible()).toBe(true);
  expect(w.get('[role="separator"]').attributes('aria-valuenow')).toBe('330');
});

it.each(['broken json', '{"width":9999,"collapsed":"yes","view":"bad"}'])('ignores malformed layout: %s', async saved => {
  await library.setLocalSetting('workbench_layout', saved); await shell();
  expect(w.get('[role="separator"]').attributes('aria-valuenow')).toBe('260');
  expect(w.get('.sidebar').isVisible()).toBe(true);
});

it('separates category expansion from filtering', async () => {
  await shell();
  const group = w.findAll('.tree-group')[0];
  const name = group.get('.tree-parent'), arrow = group.get('.tree-expand');
  const opened = arrow.attributes('aria-expanded');
  await name.trigger('click'); await flushPromises();
  expect(name.classes()).toContain('active');
  expect(arrow.attributes('aria-expanded')).toBe(opened);
  await w.get('.category-tree > .tree-row').trigger('click'); await flushPromises();
  await arrow.trigger('click');
  expect(arrow.attributes('aria-expanded')).not.toBe(opened);
  expect(name.classes()).not.toContain('active');
});

it('debounces remote searches, waits for composition, caches dictionaries and clears immediately', async () => {
  const catalog = vi.fn(async () => ({categories:[],models:[]}));
  const list = vi.fn(async () => []);
  setCatalogTransport(catalog); setSquareTransport(list);
  await shell(); await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  vi.useFakeTimers();
  const input = w.get('.inline-search input');
  await input.setValue('s'); await input.setValue('sql');
  expect(list).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(250);
  expect(list).toHaveBeenCalledTimes(2); expect(list.mock.lastCall[0].query).toBe('sql');
  expect(catalog).toHaveBeenCalledTimes(1);
  await input.trigger('compositionstart'); await input.setValue('中');
  await vi.advanceTimersByTimeAsync(300); expect(list).toHaveBeenCalledTimes(2);
  await input.trigger('compositionend'); await vi.advanceTimersByTimeAsync(250);
  expect(list.mock.lastCall[0].query).toBe('中');
  await input.setValue(''); await vi.advanceTimersByTimeAsync(0);
  expect(list.mock.lastCall[0].query).toBe('');
  await input.setValue('cancel this');
  await w.get('[data-space="local"]').trigger('click');
  const calls = list.mock.calls.length;
  await vi.advanceTimersByTimeAsync(300); expect(list).toHaveBeenCalledTimes(calls);
});

it('reacts to system theme changes only in system mode and removes its listener', async () => {
  const media = {matches:false, addEventListener:vi.fn(),removeEventListener:vi.fn()};
  vi.stubGlobal('matchMedia', () => media);
  await library.setLocalSetting('theme', 'system'); await shell();
  const change = media.addEventListener.mock.calls[0][1];
  media.matches = true; change(); await flushPromises();
  expect(document.body.classList.contains('theme-dark')).toBe(true);
  await w.get('[title="切换浅色主题"]').trigger('click'); await flushPromises();
  expect(document.body.classList.contains('theme-dark')).toBe(false);
  change(); expect(document.body.classList.contains('theme-dark')).toBe(false);
  w.unmount(); expect(media.removeEventListener).toHaveBeenCalledWith('change',change);
});

it('ignores an old remote response as soon as a newer query is typed', async () => {
  let resolveOld;
  setSquareTransport(async ({query}) => query === 'old' ? new Promise(resolve => { resolveOld = resolve; }) : []);
  await shell(); await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  vi.useFakeTimers();
  const input = w.get('.inline-search input');
  await input.setValue('old'); await vi.advanceTimersByTimeAsync(250);
  await input.setValue('new');
  resolveOld([{id:'old-result',title:'过时搜索结果',kind:'prompt'}]);
  await vi.advanceTimersByTimeAsync(0);
  expect(w.text()).not.toContain('过时搜索结果');
  await vi.advanceTimersByTimeAsync(250);
  expect(w.text()).not.toContain('过时搜索结果');
});

it.each(['button','cancel','escape'])('protects dirty editor content on %s return', async method => {
  w = mount(CreatePromptModal, {attachTo:document.body});
  await w.get('input').setValue('未保存标题'); await w.get('textarea').setValue('保留正文');
  if (method === 'button') await w.get('.page-back').trigger('click');
  else if (method === 'cancel') await w.get('.modal-footer .ghost-button').trigger('click');
  else await w.get('textarea').trigger('keydown',{key:'Escape'});
  expect(w.emitted('cancel')).toBeUndefined();
  expect(w.get('[data-testid="discard-editor"]').text()).toContain('放弃未保存的修改');
  await w.findAll('button').find(b=>b.text()==='继续编辑').trigger('click');
  expect(w.get('input').element.value).toBe('未保存标题');
  expect(w.get('textarea').element.value).toBe('保留正文');
  await w.get('.page-back').trigger('click');
  await w.findAll('button').find(b=>b.text()==='放弃修改').trigger('click');
  expect(w.emitted('cancel')).toHaveLength(1);
});

it('closes an unchanged editor and prevents closing or deleting during save', async () => {
  w = mount(CreatePromptModal); await w.get('.page-back').trigger('click');
  expect(w.emitted('cancel')).toHaveLength(1); w.unmount();
  w = mount(CreatePromptModal,{props:{busy:true,prompt:{id:'test',title:'保存中',content:'正文'}}});
  await w.get('.page-back').trigger('click');
  expect(w.emitted('cancel')).toBeUndefined();
  expect(w.get('.danger-button').attributes('disabled')).toBeDefined();
  await w.setProps({busy:false,error:'保存失败'});
  expect(w.get('input').element.value).toBe('保存中');
});

it('restores the account on mount and offers retry after a temporary failure', async () => {
  const restore = vi.spyOn(session, 'restoreSession').mockRejectedValueOnce(new Error('offline')).mockImplementationOnce(async () => {
    session.setSessionTransport(async () => ({access_token:'acc.restored',email:'restored@example.test'}));
    await session.loginSession({email:'restored@example.test',password:'test'});
    return session.getSession();
  });
  await shell();
  expect(restore).toHaveBeenCalledTimes(1);
  expect(w.text()).toContain('登录暂未恢复');
  await w.get('[data-testid="retry-session"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="open-login"]').text()).toContain('restored@example.test');
  expect(w.find('[data-testid="retry-session"]').exists()).toBe(false);
});
