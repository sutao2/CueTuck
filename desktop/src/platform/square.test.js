import { beforeEach, describe, expect, it } from "vitest";
import { createLocalPrompt, listLocalPrompts, resetMemoryLibrary, setLocalSetting } from "./library.js";
import {
  createPublication,
  downloadSquareItem,
  listMyPublications,
  listSquareItems,
  putFavorite,
  resetSquare,
  setDownloadStatsTransport,
  setFavoriteTransport,
  setMineTransport,
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

  it("copies author onto the local row only when keep_author_on_download is on", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
      author: "林晚",
    }));
    await setLocalSetting("keep_author_on_download", "1");
    const kept = await downloadSquareItem("sq-1");
    expect(kept.author).toBe("林晚");
    expect(kept.content).toBe("清透蓝天下的多元人物群像。");

    await setLocalSetting("keep_author_on_download", "0");
    setSquareContentTransport(async (id) => ({
      id,
      title: "夜景街拍",
      content: "潮湿路面的霓虹倒影。",
      author: "林晚",
    }));
    const skipped = await downloadSquareItem("sq-2");
    expect(skipped.author).toBeFalsy();
    expect(skipped.content).toBe("潮湿路面的霓虹倒影。");
  });

  it("posts anonymous download stats without auth after a successful download when the setting is on", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const calls = [];
    setDownloadStatsTransport(async (request) => {
      calls.push(request);
    });
    await setLocalSetting("anonymous_download_stats", "1");
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    expect(calls).toHaveLength(1);
    expect(calls[0].id).toBe("sq-1");
    expect(calls[0].method).toBe("POST");
    expect(calls[0].path).toBe("/v1/square/items/sq-1/downloads");
    expect(calls[0].headers).toEqual({});
    expect(JSON.stringify(calls[0])).not.toMatch(/dev@promptark|自然光群像|清透蓝天|Authorization|authorization/);
  });

  it("does not post download stats when the setting is off", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const calls = [];
    setDownloadStatsTransport(async (request) => {
      calls.push(request);
    });
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    expect(calls).toEqual([]);
  });

  it("keeps the local download when stats post fails", async () => {
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    setDownloadStatsTransport(async () => {
      throw new Error("stats down");
    });
    await setLocalSetting("anonymous_download_stats", "1");
    const row = await downloadSquareItem("sq-1");
    expect(row.source).toBe("downloaded");
    const listed = await listLocalPrompts({ query: "自然光群像" });
    expect(listed).toHaveLength(1);
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

  it("lists my publications from the injected transport", async () => {
    setMineTransport(async () => [
      { id: "pub-1", source_id: "mem-1", status: "pending", title: "新稿" },
    ]);
    const rows = await listMyPublications();
    expect(rows).toEqual([
      { id: "pub-1", source_id: "mem-1", status: "pending", title: "新稿" },
    ]);
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
