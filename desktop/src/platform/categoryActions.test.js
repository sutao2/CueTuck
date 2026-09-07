import { beforeEach, expect, it } from 'vitest';
import * as lib from './library.js';

beforeEach(lib.resetMemoryLibrary);

it('creates custom roots and children, protects nonempty roots and retains direct content on deletion', async () => {
  const root = await lib.createLocalCategory({ name: ' 我的项目 ' });
  expect(root).toMatchObject({ name: '我的项目', parent_id: null, is_system: false });
  await expect(lib.createLocalCategory({ name: '我的项目' })).rejects.toThrow('同名');
  await expect(lib.createLocalCategory({ name: ' 软件开发 ' })).rejects.toThrow('同名');
  const child = await lib.createLocalCategory({ name: '发布', parentId: root.id });
  await expect(lib.createLocalCategory({ name: '三级', parentId: child.id })).rejects.toThrow('不能');
  await lib.createLocalPrompt({ title: '根内容', content: '保留', categoryId: root.id });
  await lib.createLocalPrompt({ title: '子内容', content: '保留', categoryId: child.id });
  expect(await lib.listLocalPrompts({ categoryId: root.id })).toHaveLength(2);
  await expect(lib.deleteLocalCategory(root.id)).rejects.toThrow('先删除');
  await lib.deleteLocalCategory(child.id);
  await lib.deleteLocalCategory(root.id);
  expect((await lib.listLocalPrompts()).every(row => row.category_id === null)).toBe(true);
  await expect(lib.createLocalCategory({ name: '失效父级', parentId: root.id })).rejects.toThrow('不存在');
});

it('round trips custom roots in reversed sync and import order and rejects invalid trees atomically', async () => {
  const root = await lib.createLocalCategory({ name: '项目' });
  const child = await lib.createLocalCategory({ name: '发布', parentId: root.id });
  const collection = await lib.createLocalCollection({ title: '合集', categoryId: root.id });
  const prompt = await lib.createLocalPrompt({ title: '内容', content: '保留', categoryId: child.id });
  await lib.addPromptToCollection(prompt.id, collection.id);
  const snapshot = (await lib.exportLocalSyncChanges()).reverse();
  const file = JSON.parse(await lib.exportLocalLibrary());
  file.categories.reverse();
  lib.resetMemoryLibrary();
  await lib.applyLocalSyncChanges(snapshot);
  expect((await lib.listLocalCategories()).find(c => c.id === child.id).parent_id).toBe(root.id);
  await lib.applyLocalImport(JSON.stringify(file));
  const copy = (await lib.listLocalPrompts()).find(p => p.id !== prompt.id);
  const categories = await lib.listLocalCategories();
  const copiedChild = categories.find(c => c.id === copy.category_id);
  expect(copiedChild.id).not.toBe(child.id);
  expect(copiedChild.parent_id).not.toBe(root.id);
  expect(categories.find(c => c.id === copiedChild.parent_id)).toMatchObject({ name: '项目', parent_id: null });
  expect((await lib.listLocalCollections()).find(c => c.id === copy.collection_id).category_id).toBe(copiedChild.parent_id);
  const before = await lib.exportLocalSyncChanges();
  const stamp = String(Date.now() + 10000);
  await expect(lib.applyLocalSyncChanges([{ id: root.id, kind: 'category', payload: { name: '项目', parent_id: 'cat-office' }, updated_at: stamp }])).rejects.toThrow('层级');
  expect(await lib.exportLocalSyncChanges()).toEqual(before);
  file.categories.push({ id: 'third', name: '三级', parent_id: child.id });
  expect(() => lib.previewImportJson(JSON.stringify(file))).toThrow('两级');
  await expect(lib.applyLocalImport(JSON.stringify(file))).rejects.toThrow('两级');
  expect(await lib.exportLocalSyncChanges()).toEqual(before);
  await lib.deleteLocalCategory(child.id);
  await lib.deleteLocalCategory(root.id);
  const deleted = await lib.exportLocalSyncChanges();
  lib.resetMemoryLibrary();
  await lib.applyLocalSyncChanges(deleted.reverse());
  expect((await lib.listLocalCategories()).some(c => c.id === root.id || c.id === child.id)).toBe(false);
});

it('rejects blank and duplicate sibling names while allowing different parents', async () => {
  await expect(lib.createLocalCategory({ name: ' ', parentId: 'cat-office' })).rejects.toThrow('不能为空');
  await lib.createLocalCategory({ name: ' 周报 ', parentId: 'cat-office' });
  await expect(lib.createLocalCategory({ name: '周报', parentId: 'cat-office' })).rejects.toThrow('同名');
  await expect(lib.createLocalCategory({ name: '周报', parentId: 'cat-image' })).resolves.toMatchObject({ name: '周报' });
});

it('soft deletes custom categories without deleting content or collection membership and syncs the tombstone', async () => {
  const category = await lib.createLocalCategory({ name: '周报', parentId: 'cat-office' });
  const collection = await lib.createLocalCollection({ title: '合集', categoryId: category.id });
  const prompt = await lib.createLocalPrompt({ title: '正文', content: '保留', categoryId: category.id });
  await lib.addPromptToCollection(prompt.id, collection.id);
  const before = await lib.exportLocalSyncChanges();
  await lib.deleteLocalCategory(category.id);
  expect((await lib.listLocalCategories()).some(c => c.id === category.id)).toBe(false);
  expect((await lib.listLocalPrompts())[0]).toMatchObject({ content: '保留', category_id: null, collection_id: collection.id });
  expect((await lib.listLocalCollections())[0].category_id).toBeNull();
  expect(await lib.listCollectionMembers(collection.id)).toHaveLength(1);
  const after = await lib.exportLocalSyncChanges();
  expect(after.find(c => c.id === category.id).deleted_at).toBeTruthy();
  expect(JSON.parse(await lib.exportLocalLibrary()).categories.some(c => c.id === category.id)).toBe(false);
  lib.resetMemoryLibrary();
  await lib.applyLocalSyncChanges(before);
  // A remote category tombstone alone must also detach local content.
  await lib.applyLocalSyncChanges(after.filter(c => c.kind === 'category'));
  expect((await lib.listLocalCategories()).some(c => c.id === category.id)).toBe(false);
  expect((await lib.listLocalPrompts())[0].category_id).toBeNull();
  expect((await lib.listLocalCollections())[0].category_id).toBeNull();
  await lib.applyLocalSyncChanges(before);
  expect((await lib.listLocalCategories()).some(c => c.id === category.id)).toBe(false);
  await expect(lib.createLocalCategory({ name: '周报', parentId: 'cat-office' })).resolves.toBeTruthy();
});

it('refuses system categories and missing IDs without changing data', async () => {
  const before = await lib.listLocalCategories();
  await expect(lib.deleteLocalCategory('cat-office')).rejects.toThrow('系统');
  await expect(lib.deleteLocalCategory('cat-software-0')).rejects.toThrow('系统');
  await expect(lib.deleteLocalCategory('missing')).rejects.toThrow('不存在');
  expect(await lib.listLocalCategories()).toEqual(before);
});
