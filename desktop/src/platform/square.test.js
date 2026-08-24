import { beforeEach, describe, expect, it } from "vitest";
import { createLocalPrompt, listLocalPrompts, resetMemoryLibrary } from "./library.js";
import {
  createPublication,
  downloadSquareItem,
  listSquareItems,
  putFavorite,
  resetSquare,
  setFavoriteTransport,
  setPublishTransport,
  setSquareContentTransport,
  setSquareTransport,
} from "./square.js";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";

describe("square client", () => {
  beforeEach(() => {
    resetSquare();
    resetMemoryLibrary();
    resetMemorySession();
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
    const calls = [];
    setPublishTransport(async (payload) => {
      calls.push(payload);
      return { id: "pub-1", source_id: payload.sourceId, status: "pending" };
    });
    const result = await createPublication({
      sourceId: created.id,
      title: created.title,
      content: created.content,
    });
    expect(result.status).toBe("pending");
    expect(calls[0]).toEqual({ sourceId: created.id, title: "本地源", content: "旧正文" });
    const listed = await listLocalPrompts({ query: "本地源" });
    expect(listed[0].content).toBe("旧正文");
  });

  it("puts a favorite without writing a local copy", async () => {
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const calls = [];
    setFavoriteTransport(async (request) => {
      calls.push(request);
      return { id: request.id };
    });
    await putFavorite("sq-1");
    expect(calls).toEqual([{ method: "PUT", id: "sq-1" }]);
    expect(await listLocalPrompts({ query: "" })).toHaveLength(0);
  });
});
