import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import { createLocalCollection, createLocalPrompt, resetMemoryLibrary } from '../platform/library.js';
import { resetSquare, setCatalogTransport, setSquarePageTransport, setSquareContentTransport } from '../platform/square.js';
import { resetMemorySession } from '../platform/session.js';
import { setSkillsTransportForTests } from '../platform/skills.js';
import { setSkillMarketTransport } from '../platform/skillMarket.js';
import { shortcutStatus } from '../platform/shortcut.js';

let w;
beforeEach(() => {
  resetMemoryLibrary(); resetSquare(); resetMemorySession(); vi.useFakeTimers();
  setCatalogTransport(async () => ({ categories: [], models: [] }));
  setSquarePageTransport(async () => ({ items: [], total: 0, next_offset: null }));
});
afterEach(() => { w?.unmount(); vi.useRealTimers(); setSkillsTransportForTests(null); setSkillMarketTransport(null); shortcutStatus.error = ''; });
async function start(host = 'macos') {
  w = mount(WorkbenchShell, { props: { host }, attachTo: document.body }); await flushPromises();
}
async function search(text) {
  await w.get('[data-testid="titlebar-search"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="global-search-input"]').setValue(text);
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
}
it('protects an unsaved prompt before opening shortcut repair from the footer', async () => {
  await start(); await w.get('.content-actions .primary-button').trigger('click');
  await w.get('[data-testid="prompt-editor"] input[data-dialog-autofocus]').setValue('未保存内容');
  shortcutStatus.error = '冲突'; await flushPromises();
  await w.get('[data-testid="shortcut-warning"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="settings-page"]').exists()).toBe(false);
  expect(w.get('[data-testid="discard-editor"]').text()).toContain('放弃未保存');
  await w.get('[data-testid="discard-editor"] .primary-button').trigger('click');
  expect(w.get('[data-testid="prompt-editor"] input[data-dialog-autofocus]').element.value).toBe('未保存内容');
});
it('opens a Skill search result with a single return action and restores prompt filters on Escape', async () => {
  const skill = { key: 'qa', path: '/qa/skill', name: 'QA Skill', description: '说明', root_id: 'qa' };
  const native = vi.fn(async request => request.action === 'snapshot'
    ? { skills: [skill], roots: [], backups: [], operations: [], sources: [], warnings: [] }
    : { package: { name: 'QA Skill', body: '# 正文', files: [], warnings: [], bytes: 10 }, modified: false });
  setSkillsTransportForTests(native);
  await start(); await w.get('.inline-search input').setValue('保留筛选');
  await search('QA'); await w.get('[data-search-scope="skills-local"]').trigger('click');
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
  await w.get('[role="option"]').trigger('click'); await flushPromises();
  const detail = w.get('[data-testid="skill-search-detail"]');
  expect(detail.text()).toContain('# 正文');
  expect(detail.findAll('button').filter(button => button.text().includes('返回'))).toHaveLength(1);
  expect(detail.get('.skills-back').element.disabled).toBe(false);
  await detail.trigger('keydown', { key: 'Escape', isComposing: true });
  expect(w.find('[data-testid="skill-search-detail"]').exists()).toBe(true);
  await detail.trigger('keydown', { key: 'Escape' }); await flushPromises();
  expect(w.find('[data-testid="skill-search-detail"]').exists()).toBe(false);
  expect(w.get('.inline-search input').element.value).toBe('保留筛选');
  expect(native.mock.calls.some(([request]) => request.action === 'install')).toBe(false);
});

it('opens a community Skill result as a detail without downloading a package', async () => {
  setSkillsTransportForTests(async () => ({ skills: [], roots: [], backups: [], operations: [], sources: [], warnings: [] }));
  const market = vi.fn(async action => action === 'browse'
    ? { items: [{ id: 'community', title: '社区目标', description: '用途' }], total: 1 }
    : { id: 'community', title: '社区目标', body: '社区完整正文', files: [], status: 'approved', license: 'MIT' });
  setSkillMarketTransport(market);
  await start(); await search('社区'); await w.get('[data-search-scope="skills-square"]').trigger('click');
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
  await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="skill-search-detail"]').text()).toContain('社区完整正文');
  expect(market.mock.calls.map(([action]) => action)).toEqual(['browse', 'detail']);
});
it.each([['macos', 'metaKey', '⌘K'], ['windows', 'ctrlKey', 'Ctrl K']])('opens global search on %s and restores focus and page query on Escape', async (host, modifier, label) => {
  await start(host); const input = w.get('.inline-search input'); await input.setValue('页面查询'); input.element.focus();
  await input.trigger('keydown', { key: 'k', [modifier]: true, isComposing: true });
  expect(w.find('[role="dialog"]').exists()).toBe(false);
  await input.trigger('keydown', { key: 'k', [modifier]: true }); await flushPromises();
  expect(w.get('[data-testid="titlebar-search"]').text()).toContain(label);
  expect(document.activeElement).toBe(w.get('[data-testid="global-search-input"]').element);
  await w.get('[data-testid="global-search-input"]').trigger('keydown', { key: 'f', [modifier]: true });
  expect(document.activeElement).not.toBe(input.element);
  await w.get('[data-testid="global-search-input"]').trigger('keydown', { key: 'Escape' }); await flushPromises();
  expect(w.find('[role="dialog"]').exists()).toBe(false); expect(input.element.value).toBe('页面查询');
  expect(document.activeElement).toBe(input.element);
});
it('finds a prompt outside the selected category and restores the underlying filters after returning', async () => {
  await createLocalPrompt({ title: '跨分类目标', content: '正文', categoryId: 'cat-software' });
  await start(); await w.get('[data-testid="uncategorized"]').trigger('click');
  await w.get('.inline-search input').setValue('当前筛选'); await vi.advanceTimersByTimeAsync(300); await flushPromises();
  expect(w.findAll('.prompt-card')).toHaveLength(0);
  await search('跨分类'); await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="local-detail"]').text()).toContain('跨分类目标');
  await w.get('[data-testid="local-detail"] .page-back').trigger('click'); await flushPromises();
  expect(w.get('.inline-search input').element.value).toBe('当前筛选');
  expect(w.get('[data-testid="uncategorized"]').classes()).toContain('active'); expect(w.findAll('.prompt-card')).toHaveLength(0);
});
it('opens a local collection from a square page and a square result from a local page', async () => {
  await createLocalCollection({ title: '跨空间合集' }); await start();
  await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  await search('跨空间'); await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="local-detail"]').exists()).toBe(false);
  expect(w.get('[data-testid="collection-detail"]').text()).toContain('跨空间合集');
  await w.get('[data-space="local"]').trigger('click'); await flushPromises();
  setSquarePageTransport(async () => ({ items: [{ id: 'remote', title: '远程命中', kind: 'prompt', is_favorite: true }], total: 1, next_offset: null }));
  setSquareContentTransport(async () => ({ content: '远程完整正文' }));
  await search('远程'); await w.get('[data-search-scope="square"]').trigger('click');
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
  await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="square-detail"]').text()).toContain('远程完整正文');
  expect(w.get('[data-testid="square-detail"] .page-back').text()).toBe('← 返回本地列表');
  expect(w.get('[data-testid="square-detail"]').text()).toContain('已收藏');
  expect(w.get('[data-space="local"]').attributes('aria-selected')).toBe('true');
});
it('keeps editor drafts when cancelling search or declining result navigation', async () => {
  await createLocalPrompt({ title: '可打开的目标', content: '正文' }); await start();
  await w.get('.content-actions .primary-button').trigger('click'); await flushPromises();
  const title = w.get('[data-testid="prompt-editor"] input[data-dialog-autofocus]'); await title.setValue('还没保存的草稿');
  await search('目标'); await w.get('[aria-label="关闭全局搜索"]').trigger('click');
  expect(title.element.value).toBe('还没保存的草稿'); expect(w.find('[data-testid="discard-editor"]').exists()).toBe(false);
  await search('目标'); await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="discard-editor"]').text()).toContain('放弃未保存');
  expect(w.get('[data-testid="titlebar-search"]').attributes('disabled')).toBeDefined();
  await w.get('[data-testid="discard-editor"] .primary-button').trigger('click');
  expect(w.get('[data-testid="prompt-editor"] input[data-dialog-autofocus]').element.value).toBe('还没保存的草稿');
  await search('目标'); await w.get('[role="option"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="discard-editor"] .danger-button').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="local-detail"]').text()).toContain('可打开的目标');
});
it('protects settings drafts when choosing a global result', async () => {
  await createLocalPrompt({ title: '设置跳转目标', content: '正文' }); await start();
  await w.get('[data-testid="open-settings"]').trigger('click'); await flushPromises();
  await w.get('[data-settings-page="data"]').trigger('click');
  await w.get('textarea').setValue('未保存导入文本');
  await w.trigger('keydown', { key: 'k', metaKey: true }); await flushPromises();
  await w.get('[data-testid="global-search-input"]').setValue('设置跳转'); await vi.advanceTimersByTimeAsync(250); await flushPromises();
  await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.get('[aria-modal="true"]').text()).toContain('放弃未保存');
  await w.get('[data-testid="cancel-settings-action"]').trigger('click');
  expect(w.get('textarea').element.value).toBe('未保存导入文本');
});

it('recognizes an existing downloaded copy when searching the square before visiting it', async () => {
  const local = await createLocalPrompt({ title: '已下载目标', content: '本地正文', source: 'downloaded' });
  local.remote_id = 'downloaded-remote';
  setSquarePageTransport(async () => ({ items: [{ id: 'downloaded-remote', title: '已下载目标', kind: 'prompt' }], total: 1, next_offset: null }));
  setSquareContentTransport(async () => ({ content: '广场正文' }));
  await start(); await search('已下载'); await w.get('[data-search-scope="square"]').trigger('click');
  await vi.advanceTimersByTimeAsync(250); await flushPromises();
  await w.get('[role="option"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="square-detail-download"]').text()).toBe('打开本地副本');
});
