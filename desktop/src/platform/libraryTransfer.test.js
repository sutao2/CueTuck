import { beforeEach, expect, it } from "vitest";
import { addPromptToCollection, applyLocalImport, createLocalCategory, createLocalCollection, createLocalPrompt, exportLocalLibrary, listLocalCategories, listLocalCollections, listLocalPrompts, previewImportJson, resetMemoryLibrary } from "./library.js";

beforeEach(resetMemoryLibrary);

it("imports copies with categories, covers, models and membership without overwriting originals", async () => {
  const category = await createLocalCategory({ name: "自定义图片", parentId: "cat-image" });
  const collection = await createLocalCollection({ title: "合集", categoryId: category.id, coverType: "single", coverUrls: ["one.png"] });
  const prompt = await createLocalPrompt({ title: "成员", content: "正文", model: "Flux", categoryId: category.id });
  await addPromptToCollection(prompt.id, collection.id);
  await applyLocalImport(await exportLocalLibrary());
  const prompts = await listLocalPrompts();
  expect(prompts).toHaveLength(2);
  const copy = prompts.find((row) => row.id !== prompt.id);
  expect(copy.model).toBe("Flux");
  expect(copy.category_id).not.toBe(category.id);
  expect(copy.collection_id).not.toBe(collection.id);
  expect((await listLocalCollections()).find((row) => row.id === copy.collection_id).cover_json).toBe('["one.png"]');
  expect((await listLocalCategories()).find((row) => row.id === copy.category_id).name).toBe("自定义图片");
});

it("rejects an invalid import as a whole instead of leaving partial rows", async () => {
  await expect(applyLocalImport(JSON.stringify({ prompts: [{ title: "不应写入", content: "a" }, { title: "", content: "b" }] }))).rejects.toThrow();
  expect(await listLocalPrompts()).toHaveLength(0);
  expect(() => previewImportJson('{"prompts":null}')).toThrow();
  await expect(applyLocalImport('{"prompts":[{"title":"坏关联","collection_id":"missing"}]}')).rejects.toThrow();
  expect(await listLocalPrompts()).toHaveLength(0);
});

it("accepts legacy title/content files and rejects duplicate ids", async () => {
  await applyLocalImport('{"prompts":[{"title":"旧格式","content":"旧正文"}]}');
  expect((await listLocalPrompts())[0].content).toBe("旧正文");
  expect(() => previewImportJson('{"prompts":[{"id":"p","title":"一"},{"id":"p","title":"二"}]}')).toThrow();
});
