import { beforeEach, expect, it } from 'vitest';
import { resetMemoryLibrary, createLocalCollection, createCollectionPrompt, listLocalPrompts, listCollectionMembers, removePromptFromCollection, deleteLocalCollection, createLocalPrompt, addPromptToCollection } from './library.js';
beforeEach(resetMemoryLibrary);
it('keeps new collection prompts private to the collection until detached, while existing prompts remain standalone', async () => {
  const c = await createLocalCollection({ title: '合集' });
  const existing = await createLocalPrompt({ title: '已有', content: '正文' });
  await addPromptToCollection(existing.id, c.id);
  const member = await createCollectionPrompt({ collectionId: c.id, title: '专属', content: '正文' });
  expect((await listLocalPrompts()).map(p => p.id)).toEqual([existing.id]);
  expect(await listCollectionMembers(c.id)).toHaveLength(2);
  await removePromptFromCollection(member.id, c.id);
  expect(await listLocalPrompts()).toHaveLength(2);
  const second = await createCollectionPrompt({ collectionId: c.id, title: '保留', content: '不会丢失' });
  await deleteLocalCollection(c.id);
  expect((await listLocalPrompts()).find(p => p.id === second.id)).toMatchObject({ content: '不会丢失', source: 'local', collection_id: null });
});
it('rejects missing collections without leaving a standalone prompt', async () => {
  await expect(createCollectionPrompt({ collectionId: 'missing', title: '失败', content: '' })).rejects.toThrow('合集不存在');
  expect(await listLocalPrompts()).toHaveLength(0);
});
