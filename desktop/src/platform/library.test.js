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
  createLocalCategory,
  listLocalCategories,
} from "./library.js";

describe("memory library", () => {
  beforeEach(() => {
    resetMemoryLibrary();
  });

  it("keeps created prompts when tauri is absent", async () => {
    await createLocalPrompt({ title: "测试", content: "正文" });
    const rows = await listLocalPrompts({ query: "测试" });
    expect(rows).toHaveLength(1);
    expect(rows[0].title).toBe("测试");
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
});
