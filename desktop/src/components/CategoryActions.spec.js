import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import * as lib from '../platform/library.js';
import { loginSession, resetMemorySession, setSessionTransport } from '../platform/session.js';
import { resetSquare, setPublishTransport } from '../platform/square.js';

let wrapper;
beforeEach(() => { lib.resetMemoryLibrary(); resetMemorySession(); resetSquare(); });
afterEach(() => { wrapper?.unmount(); vi.restoreAllMocks(); });
async function open() {
  wrapper = mount(WorkbenchShell, { attachTo: document.body });
  await flushPromises();
}

it('creates a root from all prompts, creates a child, and guards root deletion until children are removed', async () => {
  await open();
  await wrapper.get('[data-testid="add-category"]').trigger('click');
  expect(wrapper.get('[data-testid="category-parent"]').element.value).toBe('');
  await wrapper.get('[data-testid="new-category-name"]').setValue('项目');
  await wrapper.get('[data-testid="confirm-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('.tree-parent.active').text()).toContain('项目');
  const root = (await lib.listLocalCategories()).find(c => c.name === '项目');
  await wrapper.get('[data-testid="add-category"]').trigger('click');
  expect(wrapper.get('[data-testid="category-parent"]').element.value).toBe(root.id);
  await wrapper.get('[data-testid="new-category-name"]').setValue('发布');
  await wrapper.get('[data-testid="confirm-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('.tree-row.child.active').text()).toContain('发布');
  await wrapper.get('[aria-label="删除分类 项目"]').trigger('click');
  await wrapper.get('[data-testid="confirm-delete-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('[data-testid="category-error"]').text()).toContain('先删除');
  await wrapper.get('[aria-label="关闭分类窗口"]').trigger('click');
  for (const name of ['发布', '项目']) {
    await wrapper.get(`[aria-label="删除分类 ${name}"]`).trigger('click');
    await wrapper.get('[data-testid="confirm-delete-category"]').trigger('click');
    await flushPromises();
    expect(wrapper.find(`[aria-label="删除分类 ${name}"]`).exists()).toBe(false);
  }
});

it('hides custom root trees in the square and never publishes private category IDs', async () => {
  const root = await lib.createLocalCategory({ name: '私有项目' });
  const child = await lib.createLocalCategory({ name: '私有子类', parentId: root.id });
  const collection = await lib.createLocalCollection({ title: '合集', categoryId: root.id });
  const member = await lib.createLocalPrompt({ title: '成员', content: '内容', categoryId: child.id });
  await lib.addPromptToCollection(member.id, collection.id);
  setSessionTransport(async () => ({ email: 'dev@promptark.local', access_token: 'acc' }));
  await loginSession({ email: 'dev@promptark.local', password: 'devpass' });
  const publish = vi.fn(async () => ({ status: 'pending' }));
  setPublishTransport(publish);
  await open();
  expect(wrapper.get('.category-tree').text()).toContain('私有项目');
  await wrapper.get('[data-space="square"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('.category-tree').text()).not.toContain('私有');
  await wrapper.get('[data-testid="publish-prompt"]').trigger('click');
  await flushPromises();
  await wrapper.get('[data-testid="publish-source"]').setValue(collection.id);
  await wrapper.get('[data-testid="publish-submit"]').trigger('click');
  await flushPromises();
  expect(publish).toHaveBeenCalledWith(expect.objectContaining({ members: [expect.objectContaining({ category_id: null })] }));
  expect(publish.mock.calls[0][0]).not.toHaveProperty('categoryId');
});

it('creates from all prompts with parent choice, focuses the name, and rejects duplicates', async () => {
  await open();
  await wrapper.get('[data-testid="add-category"]').trigger('click');
  await flushPromises();
  const name = wrapper.get('[data-testid="new-category-name"]');
  expect(document.activeElement).toBe(name.element);
  expect(wrapper.get('[data-testid="confirm-category"]').element.disabled).toBe(true);
  await wrapper.get('[data-testid="category-parent"]').setValue('cat-office');
  await name.setValue(' 周报 ');
  await name.trigger('keydown', { key: 'Enter', isComposing: true });
  expect((await lib.listLocalCategories()).some(c => c.name === '周报')).toBe(false);
  await wrapper.get('[data-testid="confirm-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.find('.category-modal').exists()).toBe(false);
  expect(wrapper.get('.tree-row.child.active').text()).toContain('周报');
  await wrapper.get('[data-testid="add-category"]').trigger('click');
  expect(wrapper.get('[data-testid="category-parent"]').element.value).toBe('cat-office');
  await wrapper.get('[data-testid="new-category-name"]').setValue('周报');
  await wrapper.get('[data-testid="confirm-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('[data-testid="category-error"]').text()).toContain('同名');
  expect(wrapper.get('[data-testid="new-category-name"]').element.value).toBe('周报');
});

it('prevents double submission and closing while creating', async () => {
  let finish;
  const create = lib.createLocalCategory;
  const spy = vi.spyOn(lib, 'createLocalCategory').mockImplementation(args => new Promise(resolve => { finish = () => resolve(create(args)); }));
  await open();
  await wrapper.get('[data-testid="add-category"]').trigger('click');
  const name = wrapper.get('[data-testid="new-category-name"]');
  await name.setValue('Busy');
  await name.trigger('keydown', { key: 'Enter' });
  await name.trigger('keydown', { key: 'Enter' });
  await wrapper.get('.category-modal').trigger('keydown', { key: 'Escape' });
  expect(spy).toHaveBeenCalledTimes(1);
  expect(wrapper.get('[data-testid="confirm-category"]').element.disabled).toBe(true);
  finish();
  await flushPromises();
  expect(wrapper.find('.category-modal').exists()).toBe(false);
});

it('confirms deletion, keeps content, and switches the selected category to uncategorized', async () => {
  const cat = await lib.createLocalCategory({ name: '周报', parentId: 'cat-office' });
  const prompt = await lib.createLocalPrompt({ title: '保留正文', content: '保留', categoryId: cat.id });
  const collection = await lib.createLocalCollection({ title: '保留合集', categoryId: cat.id });
  await lib.addPromptToCollection(prompt.id, collection.id);
  await open();
  await wrapper.findAll('.tree-parent').find(b => b.text().includes('办公效率')).trigger('click');
  await wrapper.findAll('.tree-row.child').find(b => b.text().includes('周报')).trigger('click');
  await wrapper.get('[aria-label="删除分类 周报"]').trigger('click');
  const dialog = wrapper.get('[role="alertdialog"]');
  expect(dialog.text()).toContain('不会删除正文');
  await dialog.findAll('button').find(b => b.text() === '取消').trigger('click');
  expect((await lib.listLocalCategories()).some(c => c.id === cat.id)).toBe(true);
  await wrapper.get('[aria-label="删除分类 周报"]').trigger('click');
  await wrapper.get('[data-testid="confirm-delete-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.find('[aria-label="删除分类 周报"]').exists()).toBe(false);
  expect(wrapper.get('[data-testid="uncategorized"]').classes()).toContain('active');
  expect(wrapper.get('[data-testid="uncategorized"]').text()).toContain('2');
  expect(wrapper.findAll('.prompt-card')).toHaveLength(2);
  expect(await lib.listCollectionMembers(collection.id)).toHaveLength(1);
});

it('keeps a failed deletion visible and hides management actions in the square', async () => {
  const cat = await lib.createLocalCategory({ name: '周报', parentId: 'cat-office' });
  vi.spyOn(lib, 'deleteLocalCategory').mockRejectedValue(new Error('disk failure'));
  await open();
  expect(wrapper.find('[aria-label="删除分类 软件开发"]').exists()).toBe(false);
  await wrapper.get('[aria-label="删除分类 周报"]').trigger('click');
  await wrapper.get('[data-testid="confirm-delete-category"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('[data-testid="category-error"]').text()).toContain('disk failure');
  expect((await lib.listLocalCategories()).some(c => c.id === cat.id)).toBe(true);
  await wrapper.get('[aria-label="关闭分类窗口"]').trigger('click');
  await wrapper.get('[data-space="square"]').trigger('click');
  expect(wrapper.find('[data-testid="add-category"]').exists()).toBe(false);
  expect(wrapper.find('[aria-label="删除分类 周报"]').exists()).toBe(false);
});
