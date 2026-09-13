import { selectOption, selectComponent } from '../test/selectOption.js';
import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, it, expect } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import { resetMemoryLibrary, createLocalPrompt, createLocalCollection } from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setSquareTransport, setCatalogTransport } from '../platform/square.js';
let w;
beforeEach(() => { resetMemoryLibrary(); resetMemorySession(); resetSquare(); setCatalogTransport(async () => ({ categories: [], models: [] })); setSquareTransport(async () => []); });
afterEach(() => w?.unmount());
async function shell() { w = mount(WorkbenchShell); await flushPromises(); }
it('does not invent square tab counts and uses the selected sort title', async () => {
  await shell(); await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  expect(w.findAll('.filter-tabs small')).toHaveLength(0);
  await w.get('[data-sort="最新"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=results-heading]').text()).toBe('最新发布');
});
it('keeps loading until the latest request settles and shows only one retryable error', async () => {
  const pending = []; setSquareTransport(() => new Promise((resolve, reject) => pending.push({resolve, reject})));
  await shell(); await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="browse-loading"]').exists()).toBe(true);
  expect(w.find('.empty-state').exists()).toBe(false);
  await w.get('[data-sort="最新"]').trigger('click'); await flushPromises();
  pending[0].resolve([{id:'old',kind:'prompt',title:'Old'}]); await flushPromises();
  expect(w.find('[data-testid="browse-loading"]').exists()).toBe(true);
  expect(w.find('.prompt-card').exists()).toBe(false);
  pending[1].reject(new Error('offline')); await flushPromises();
  expect(w.find('[data-testid="browse-loading"]').exists()).toBe(false);
  expect(w.findAll('[data-testid="square-offline"]')).toHaveLength(1);
  expect(w.find('.empty-state').exists()).toBe(false);
  await w.get('[data-testid="retry-square"]').trigger('click'); await flushPromises();
  pending[2].resolve([{id:'new',kind:'prompt',title:'New'}]); await flushPromises();
  expect(w.get('.prompt-card').text()).toContain('New');
});
it('counts the selected model and clears filters without changing the current tab or data', async () => {
  await createLocalPrompt({ title:'One',content:'body',model:'Flux' });
  await createLocalPrompt({ title:'Two',content:'body',model:'GPT' });
  await shell(); await selectOption(w, 'model-filter', 'Flux');
  expect(w.get('[data-sort="全部"] small').text()).toBe('1');
  expect(w.findAll('.prompt-card')).toHaveLength(1);
  await w.get('[data-sort="收藏"]').trigger('click');
  await w.get('[data-testid="clear-filters"]').trigger('click'); await flushPromises();
  expect(w.get('[data-sort="收藏"]').attributes('aria-selected')).toBe('true');
  expect(w.get('[data-sort="全部"] small').text()).toBe('2');
  expect(w.find('.active-filters').exists()).toBe(false);
});
it('offers a collection open action and a recovery action for empty searches', async () => {
  await createLocalCollection({ title:'资料合集' }); await shell();
  expect(w.get('.card-footer button').text()).toBe('打开合集');
  await w.get('input[type="search"]').setValue('not-found'); await flushPromises();
  expect(w.get('[data-testid=results-heading]').text()).toBe('搜索结果');
  expect(w.get('.empty-state button').text()).toBe('清除筛选');
  await w.get('.empty-state button').trigger('click'); await flushPromises();
  expect(w.get('.prompt-card').text()).toContain('资料合集');
});
