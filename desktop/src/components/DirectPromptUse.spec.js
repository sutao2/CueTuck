import { mount, flushPromises, enableAutoUnmount } from '@vue/test-utils';
import { beforeEach, afterEach, it, expect, vi } from 'vitest';
import Workbench from './WorkbenchShell.vue';
import Detail from './SquareDetailModal.vue';
import LocalDetail from './LocalPromptDetail.vue';
import * as library from '../platform/library.js';
import * as square from '../platform/square.js';
import * as translation from '../platform/translation.js';
import { resetMemorySession } from '../platform/session.js';
import { selectOption } from '../test/selectOption.js';

enableAutoUnmount(afterEach);
let writeText, stats;
const click = (w, label) => w.findAll('button').find(b => b.text() === label).trigger('click');
const original = 'Write for {{name}}.';
const translated = '为 {{name}} 写作。';
const item = () => ({ id: 'public', kind: 'prompt', title: '公开模板', content: original,
  translations: { zh: { status: 'ready', version: { source: { content: original }, content: translated } } } });
beforeEach(() => {
  library.resetMemoryLibrary(); resetMemorySession(); square.resetSquare();
  square.setCatalogTransport(async () => ({ categories: [], models: [] }));
  square.setSquarePageTransport(async () => ({ items: [{ id: 'public', kind: 'prompt', title: '公开模板', excerpt: '截断的摘要…', download_count: 7 }], total: 1, next_offset: null }));
  square.setSquareContentTransport(async () => item());
  stats = vi.fn(); square.setDownloadStatsTransport(stats);
  vi.spyOn(translation, 'localVersion').mockResolvedValue(null);
  writeText = vi.fn().mockResolvedValue();
  Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
});
afterEach(() => { vi.restoreAllMocks(); delete navigator.clipboard; document.body.innerHTML = ''; });
async function openSquare() {
  const w = mount(Workbench, { attachTo: document.body }); await flushPromises();
  await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  return w;
}

it('uses the complete ready Chinese template from a summary card without importing or counting a download', async () => {
  const record = vi.spyOn(library, 'recordLocalPromptUse');
  const w = await openSquare();
  expect(w.find('[data-testid="card-more"]').exists()).toBe(false);
  await w.get('[data-testid="use-square"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="use-modal"]').isVisible()).toBe(true);
  expect(w.get('[data-testid="square-detail"]').isVisible()).toBe(false);
  await w.get('[data-testid="use-value"]').setValue('小明');
  await w.get('[data-testid="use-next"]').trigger('click');
  expect(w.get('[data-testid="use-preview"]').text()).toBe('为 小明 写作。');
  await w.get('[data-testid="use-next"]').trigger('click'); await flushPromises();
  expect(writeText).toHaveBeenCalledExactlyOnceWith('为 小明 写作。');
  expect(record).not.toHaveBeenCalled(); expect(stats).not.toHaveBeenCalled();
  expect(await library.listLocalPrompts()).toHaveLength(0);
  expect(w.get('[data-testid="square-card-metrics"]').text()).toContain('7');
  expect(w.get('[data-testid="square-detail"]').isVisible()).toBe(true);
});

it('keeps English selected in details when using and returning from variable entry', async () => {
  const w = await openSquare(); await w.get('.prompt-title').trigger('click'); await flushPromises();
  await click(w.get('[data-testid="square-detail"]'), 'English · 原稿');
  await w.get('[data-testid="square-detail-use"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="use-value"]').setValue('Alice');
  await w.get('[data-testid="use-next"]').trigger('click');
  expect(w.get('[data-testid="use-preview"]').text()).toBe('Write for Alice.');
  await w.get('[data-testid="use-modal"] [aria-label="返回"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="square-detail-content"]').text()).toBe(original);
});

it('honors the original-language list preference', async () => {
  const w = await openSquare();
  await selectOption(w, 'language-filter', 'original'); await flushPromises();
  await w.get('[data-testid="use-square"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="use-next"]').trigger('click');
  expect(w.get('[data-testid="use-preview"]').text()).toBe(original);
});

it('keeps failed automatic copying retryable without counting a download', async () => {
  square.setSquareContentTransport(async () => ({ id: 'public', title: '完整文本', content: '完整正文', kind: 'prompt' }));
  writeText.mockRejectedValueOnce(Error('denied'));
  const w = await openSquare();
  await w.get('[data-testid="use-square"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="use-modal"]').text()).toContain('复制失败');
  expect(w.get('[data-testid="use-preview"]').text()).toBe('完整正文');
  await w.get('[data-testid="use-next"]').trigger('click'); await flushPromises();
  expect(writeText).toHaveBeenCalledTimes(2); expect(stats).not.toHaveBeenCalled();
  expect(w.find('[data-testid="use-modal"]').exists()).toBe(false);
});

it('does not copy an empty remote template or a response arriving after unmount', async () => {
  square.setSquareContentTransport(async () => ({ id: 'public', content: ' ', kind: 'prompt' }));
  const w = await openSquare();
  await w.get('[data-testid="use-square"]').trigger('click'); await flushPromises();
  expect(w.text()).toContain('内容为空'); expect(writeText).not.toHaveBeenCalled();
  await w.get('[data-testid="square-detail"] [aria-label="返回"]').trigger('click');
  let finish; square.setSquareContentTransport(() => new Promise(resolve => { finish = resolve; }));
  await w.get('[data-testid="use-square"]').trigger('click'); w.unmount();
  finish({ id: 'public', content: '迟到正文', kind: 'prompt' }); await flushPromises();
  expect(writeText).not.toHaveBeenCalled();
});

it('does not start use when a request fails or returns after the detail was closed', async () => {
  const fetch = vi.fn().mockRejectedValueOnce(Error('离线'));
  square.setSquareContentTransport(fetch);
  const w = await openSquare();
  await w.get('[data-testid="use-square"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="square-detail"]').text()).toContain('离线');
  await w.get('[data-testid="square-detail"] [aria-label="返回"]').trigger('click');
  let finish; fetch.mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  await w.get('[data-testid="use-square"]').trigger('click');
  await w.get('[data-testid="square-detail"] [aria-label="返回"]').trigger('click');
  finish(item()); await flushPromises();
  expect(w.find('[data-testid="use-modal"]').exists()).toBe(false);
  expect(writeText).not.toHaveBeenCalled();
});

it('uses each collection member in its displayed language and blocks empty members', async () => {
  const w = mount(Detail, { props: { item: { id: 'collection', kind: 'collection', title: '合集', members: [{ title: '成员', content: original }, { title: '空白', content: '' }], translations: { zh: { status: 'ready', version: { members: [{ content: translated }, { content: '' }] } } } } } });
  await flushPromises();
  const buttons = w.findAll('[data-testid="square-member-use"]');
  await buttons[0].trigger('click');
  expect(w.emitted('use').at(-1)[0]).toMatchObject({ content: translated, remote: true, asset_count: 0 });
  expect(buttons[1].element.disabled).toBe(true);
});

it('uses the visible local translation without overwriting the saved original', async () => {
  vi.spyOn(translation, 'localVersion').mockImplementation(async (_, target) => target === 'zh' ? { text: translated } : null);
  const w = mount(LocalDetail, { props: { prompt: { id: 'local', title: '本地', content: original } } });
  await flushPromises(); await click(w, '中文 · 已就绪');
  await click(w, '使用提示词');
  expect(w.emitted('use').at(-1)[0].content).toBe(translated);
  expect(w.props('prompt').content).toBe(original);
});

it('has one local use action, copies immediately and labels duplication clearly', async () => {
  await library.createLocalPrompt({ title: '本地', content: '纯文本' });
  const w = mount(Workbench, { attachTo: document.body }); await flushPromises();
  const card = w.get('.prompt-card');
  expect(card.findAll('button').filter(b => b.text() === '复制')).toHaveLength(0);
  await click(card, '使用'); await flushPromises();
  expect(writeText).toHaveBeenCalledExactlyOnceWith('纯文本');
  expect((await library.listLocalPrompts())[0].use_count).toBe(1);
  await card.get('[data-testid="card-more"]').trigger('click');
  expect(w.get('[data-action="duplicate"]').text()).toBe('创建本地副本');
});
