import { beforeEach, expect, it } from 'vitest';
import * as lib from './library.js';

beforeEach(lib.resetMemoryLibrary);

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
