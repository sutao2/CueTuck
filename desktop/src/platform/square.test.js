import { beforeEach, describe, expect, it } from "vitest";
import { createLocalPrompt, listLocalPrompts, resetMemoryLibrary } from "./library.js";
import {
  createPublication,
  downloadSquareItem,
  listSquareItems,
  resetSquare,
  setPublishTransport,
  setSquareContentTransport,
  setSquareTransport,
} from "./square.js";

describe("square client", () => {
  beforeEach(() => {
    resetSquare();
    resetMemoryLibrary();
  });

  it("returns injected items when online", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const rows = await listSquareItems({ sort: "推荐" });
    expect(rows).toHaveLength(1);
    expect(rows[0].title).toBe("自然光群像");
  });

  it("surfaces offline as a thrown error", async () => {
    setSquareTransport(async () => {
      throw new Error("广场暂时不可用");
    });
    await expect(listSquareItems()).rejects.toThrow("广场暂时不可用");
  });

  it("writes a local copy with source=downloaded", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    expect(row.title).toBe("自然光群像");
    const listed = await listLocalPrompts({ query: "自然光群像" });
    expect(listed).toHaveLength(1);
    expect(listed[0].source).toBe("downloaded");
  });

  it("submits a publication without changing the local copy", async () => {
    const created = await createLocalPrompt({ title: "本地源", content: "旧正文" });
    setPublishTransport(async ({ sourceId }) => ({ id: "pub-1", source_id: sourceId, status: "pending" }));
    const result = await createPublication({ sourceId: created.id });
    expect(result.status).toBe("pending");
    const listed = await listLocalPrompts({ query: "本地源" });
    expect(listed[0].content).toBe("旧正文");
  });
});
