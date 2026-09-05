import { beforeEach, describe, expect, it } from "vitest";
import {
  addPromptToCollection,
  createLocalCollection,
  createLocalPrompt,
  deleteLocalPrompt,
  listCollectionMembers,
  listLocalPrompts,
  previewImportJson,
  resetMemoryLibrary,
  backupLocalLibrary,
  restoreLocalLibrary,
  exportLibraryZip,
  openLibraryDir,
  recordLocalPromptUse,
  clearLocalPromptUse,
  createLocalCategory,
  listLocalCategories,
  listLocalCollections,
  updateLocalCollection,
  deleteLocalCollection,
  removePromptFromCollection,
} from "./library.js";

describe("memory library", () => {
  it("filters uncategorized prompts without including categorized rows", async () => {
    resetMemoryLibrary();
    await createLocalPrompt({ title: "无分类", content: "正文" });
    await createLocalPrompt({ title: "图片", content: "正文", categoryId: "cat-image-0" });
    expect((await listLocalPrompts({ categoryId: "__uncategorized__" })).map((row) => row.title)).toEqual(["无分类"]);
  });
  beforeEach(() => {
    resetMemoryLibrary();
  });

  it("keeps created prompts when tauri is absent", async () => {
    await createLocalPrompt({ title: "测试", content: "正文" });
    const rows = await listLocalPrompts({ query: "测试" });
    expect(rows).toHaveLength(1);
    expect(rows[0].title).toBe("测试");
  });

  it("persists the selected model on create and update", async () => {
    const created = await createLocalPrompt({ title: "模型片", content: "正文", model: "Flux" });
    expect(created.model).toBe("Flux");
    const { updateLocalPrompt } = await import("./library.js");
    const updated = await updateLocalPrompt({
      id: created.id,
      title: "模型片",
      content: "正文",
      model: "GPT-5",
    });
    expect(updated.model).toBe("GPT-5");
  });

  it("hides deleted prompts from default search", async () => {
    const created = await createLocalPrompt({ title: "过期模板", content: "x" });
    await deleteLocalPrompt(created.id);
    expect(await listLocalPrompts({ query: "过期" })).toHaveLength(0);
  });

  it("stores cover refs on a grid collection", async () => {
    const created = await createLocalCollection({
      title: "人像灵感",
      coverType: "grid",
      coverUrls: ["one.jpg", "two.jpg", "three.jpg"],
    });
    expect(created.cover_type).toBe("grid");
    expect(JSON.parse(created.cover_json)).toEqual(["one.jpg", "two.jpg", "three.jpg"]);
  });

  it("adds a prompt to a collection", async () => {
    const collection = await createLocalCollection({ title: "人像灵感" });
    const prompt = await createLocalPrompt({ title: "提示词B", content: "正文" });
    await addPromptToCollection(prompt.id, collection.id);
    const members = await listCollectionMembers(collection.id);
    expect(members).toHaveLength(1);
    expect(members[0].title).toBe("提示词B");
  });

  it("updates both collection counts on move and keeps members when deleting a collection", async () => {
    const a = await createLocalCollection({ title: "A" });
    const b = await createLocalCollection({ title: "B" });
    const prompt = await createLocalPrompt({ title: "成员", content: "保留" });
    await expect(addPromptToCollection(prompt.id, "missing")).rejects.toThrow(/不存在/);
    await addPromptToCollection(prompt.id, a.id);
    await addPromptToCollection(prompt.id, b.id);
    const counts = new Map((await listLocalCollections()).map((row) => [row.id, row.member_count]));
    expect(counts.get(a.id)).toBe(0);
    expect(counts.get(b.id)).toBe(1);
    await removePromptFromCollection(prompt.id, b.id);
    expect((await listLocalPrompts())[0].collection_id).toBeNull();
    await updateLocalCollection({ id: b.id, title: "新合集", categoryId: "cat-image", coverType: "single", coverUrls: ["one.png"] });
    expect((await listLocalCollections({ query: "新合集" }))[0].cover_json).toBe('["one.png"]');
    await addPromptToCollection(prompt.id, b.id);
    await deleteLocalCollection(b.id);
    expect(await listLocalCollections({ query: "新合集" })).toHaveLength(0);
    expect((await listLocalPrompts())[0]).toMatchObject({ content: "保留", collection_id: null });
    await expect(addPromptToCollection(prompt.id, b.id)).rejects.toThrow(/不存在/);
  });

  it("previews import without writing", async () => {
    await createLocalPrompt({ title: "已有", content: "x" });
    const preview = previewImportJson(
      JSON.stringify({ prompts: [{ title: "一", content: "a" }, { title: "二", content: "b" }] }),
    );
    expect(preview.prompt_count).toBe(2);
    expect(await listLocalPrompts({ query: "" })).toHaveLength(1);
  });

  it("rejects sqlite file backup in the browser memory library", async () => {
    await createLocalPrompt({ title: "已有", content: "x" });
    await expect(backupLocalLibrary()).rejects.toThrow("仅桌面窗口支持库文件备份");
    await expect(restoreLocalLibrary("/tmp/fake.sqlite")).rejects.toThrow(
      "仅桌面窗口支持库文件备份",
    );
    expect(await listLocalPrompts({ query: "" })).toHaveLength(1);
  });

  it("adds a user child category under a parent", async () => {
    const created = await createLocalCategory({ name: "周报", parentId: "cat-office" });
    expect(created.is_system).toBe(false);
    const officeKids = (await listLocalCategories()).filter((row) => row.parent_id === "cat-office");
    expect(officeKids.some((row) => row.name === "周报")).toBe(true);
  });

  it("rejects a third-level category", async () => {
    await expect(
      createLocalCategory({ name: "再下一层", parentId: "cat-software-1" }),
    ).rejects.toThrow("小分类下不能再创建子分类");
  });

  it("exports a zip payload without dropping memory prompts", async () => {
    await createLocalPrompt({ title: "条目A", content: "正文" });
    const zip = await exportLibraryZip();
    expect(JSON.parse(zip).prompts).toHaveLength(1);
    expect(await openLibraryDir()).toBe("memory-library");
    expect(await listLocalPrompts({ query: "" })).toHaveLength(1);
  });

  it("records last_used_at when a prompt is used", async () => {
    const created = await createLocalPrompt({ title: "刚用过", content: "x" });
    expect(created.last_used_at).toBeFalsy();
    const used = await recordLocalPromptUse(created.id);
    expect(used.use_count).toBe(1);
    expect(used.last_used_at).toBeTruthy();
  });

  it("clears use counts without deleting prompt content", async () => {
    const created = await createLocalPrompt({ title: "条目A", content: "中文 English" });
    await recordLocalPromptUse(created.id);
    expect((await listLocalPrompts({ query: "" }))[0].use_count).toBe(1);
    await clearLocalPromptUse();
    const rows = await listLocalPrompts({ query: "" });
    expect(rows).toHaveLength(1);
    expect(rows[0].content).toBe("中文 English");
    expect(rows[0].use_count).toBe(0);
  });
});
