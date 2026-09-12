import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import * as library from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setCatalogTransport, setSquareTransport } from '../platform/square.js';

let w;
beforeEach(() => {
  library.resetMemoryLibrary(); resetMemorySession(); resetSquare();
  setCatalogTransport(async () => ({ categories: [], models: [] }));
  setSquareTransport(async () => []);
});
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
async function shell() { w = mount(WorkbenchShell, { attachTo: document.body }); await flushPromises(); }
async function create() { await w.get('.content-actions .primary-button').trigger('click'); }
const editor = () => w.get('[data-testid="prompt-editor"]');

it('opens an in-workspace editor without a backdrop and preserves the library query and scroll on save', async () => {
  await library.createLocalPrompt({ title: 'SQL', content: 'existing' });
  await shell();
  await w.get('.inline-search input').setValue('SQL'); await flushPromises();
  w.get('[data-region="content"]').element.scrollTop = 120;
  await create();
  expect(editor().attributes('role')).toBe('region');
  expect(w.find('[aria-modal="true"]').exists()).toBe(false);
  expect(w.find('.modal-backdrop').exists()).toBe(false);
  expect(w.get('.sidebar').isVisible()).toBe(true);
  expect(w.get('[data-region="content"]').isVisible()).toBe(false);
  await editor().get('input').setValue('SQL new');
  await editor().get('textarea').setValue('new content');
  await editor().get('.modal-footer .primary-button').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="prompt-editor"]').exists()).toBe(false);
  expect(w.get('.inline-search input').element.value).toBe('SQL');
  expect(w.get('[data-region="content"]').element.scrollTop).toBe(120);
  expect((await library.listLocalPrompts()).map(p => p.title)).toContain('SQL new');
});

it('requires a decision before sidebar navigation discards an editor and cancels a declined destination', async () => {
  await shell(); await create();
  await editor().get('input').setValue('keep draft');
  await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="discard-editor"]').exists()).toBe(true);
  await editor().findAll('button').find(b => b.text() === '继续编辑').trigger('click');
  expect(editor().get('input').element.value).toBe('keep draft');
  await editor().get('.modal-footer .primary-button').trigger('click'); await flushPromises();
  expect(w.get('[data-space="local"]').attributes('aria-selected')).toBe('true');
  await create(); await editor().get('input').setValue('discard draft');
  await w.get('[data-space="square"]').trigger('click');
  await editor().findAll('button').find(b => b.text() === '放弃修改').trigger('click'); await flushPromises();
  expect(w.get('[data-space="square"]').attributes('aria-selected')).toBe('true');
  expect(w.find('[data-testid="prompt-editor"]').exists()).toBe(false);
});

it('returns from member editing to its collection and refreshes the edited member', async () => {
  const collection = await library.createLocalCollection({ title: 'My collection' });
  const prompt = await library.createLocalPrompt({ title: 'Member', content: 'body' });
  await library.addPromptToCollection(prompt.id, collection.id);
  await shell();
  await w.findAll('.prompt-card').find(card => card.text().includes('My collection')).trigger('click'); await flushPromises();
  await w.get('.member-title').trigger('click');
  await w.get('[data-testid=detail-edit]').trigger('click');
  expect(w.get('[data-testid="collection-detail"]').isVisible()).toBe(false);
  await editor().get('input').setValue('Updated member');
  await editor().get('.modal-footer .primary-button').trigger('click'); await flushPromises();
  expect(w.get('[data-testid=local-detail]').text()).toContain('Updated member');
  await w.get('[data-testid=local-detail] .page-back').trigger('click');
  expect(w.get('[data-testid="collection-detail"]').isVisible()).toBe(true);
  expect(w.get('.member-title').text()).toBe('Updated member');
  await w.get('[data-testid="edit-collection"]').trigger('click');
  await editor().get('.danger-button').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="delete-confirmation"]').text()).toContain('合集内的提示词会保留');
  await w.get('[data-testid="confirm-delete"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="collection-detail"]').exists()).toBe(false);
  expect(await library.listLocalPrompts()).toHaveLength(1);
  expect(w.get('[data-testid="delete-notice"]').text()).toContain('合集内的提示词已保留');
});

it('preserves settings drafts across login and guards a sidebar destination before leaving settings', async () => {
  await shell(); await w.get('[data-testid="open-settings"]').trigger('click'); await flushPromises();
  await w.get('[data-settings-page="models"]').trigger('click');
  await w.get('[data-testid="default-model"]').setValue('Draft model');
  await w.get('[data-settings-page="account"]').trigger('click');
  await w.get('[data-testid="settings-login"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="login-modal"]').attributes('role')).toBe('region');
  await w.get('[data-testid="login-modal"] .page-back').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="settings-page"]').isVisible()).toBe(true);
  await w.get('[data-settings-page="models"]').trigger('click');
  expect(w.get('[data-testid="default-model"]').element.value).toBe('Draft model');
  await w.get('[data-settings-page="account"]').trigger('click');
  await w.get('[data-testid="settings-login"]').trigger('click');
  await w.get('[data-space="square"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="cancel-settings-action"]').exists()).toBe(true);
  await w.get('[data-testid="cancel-settings-action"]').trigger('click');
  await w.get('[data-settings-page="models"]').trigger('click');
  expect(w.get('[data-testid="default-model"]').element.value).toBe('Draft model');
});

it('blocks sidebar navigation while saving and keeps failed drafts editable', async () => {
  await shell(); await create(); await editor().get('input').setValue('Keep me');
  let rejectSave;
  vi.spyOn(library, 'createLocalPrompt').mockImplementation(() => new Promise((_, reject) => { rejectSave = reject; }));
  await editor().get('.modal-footer .primary-button').trigger('click');
  await w.get('[data-space="square"]').trigger('click');
  expect(w.get('[data-space="local"]').attributes('aria-selected')).toBe('true');
  rejectSave(new Error('test failure')); await flushPromises();
  expect(editor().get('input').element.value).toBe('Keep me');
  expect(editor().text()).toContain('保存失败');
});

it('keeps the second page and scroll after editing and returning from reading', async () => {
  for (let i = 0; i < 50; i++) await library.createLocalPrompt({ title: `条目 ${i}`, content: '原正文' });
  await shell();
  await w.get('[aria-label=提示词分页]').findAll('button')[1].trigger('click');
  w.get('[data-region=content]').element.scrollTop = 120;
  await w.findAll('.prompt-title')[0].trigger('click');
  await w.get('[data-testid=detail-edit]').trigger('click');
  await editor().get('textarea').setValue('修改正文');
  await editor().get('.modal-footer .primary-button').trigger('click'); await flushPromises();
  await w.get('[data-testid=local-detail] .page-back').trigger('click');
  expect(w.get('[aria-label=提示词分页]').text()).toContain('第 2 / 2 页');
  expect(w.get('[data-region=content]').element.scrollTop).toBe(120);
});
