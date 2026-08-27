import { beforeEach, describe, expect, it } from "vitest";
import { createLocalPrompt, listLocalPrompts, resetMemoryLibrary, setLocalSetting } from "./library.js";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";
import {
  resetSquare,
  setFavoriteTransport,
  setPublishTransport,
} from "./square.js";
import {
  favoriteWithQueue,
  flushSyncQueue,
  listSyncQueue,
  publishWithQueue,
} from "./syncQueue.js";

describe("sync queue", () => {
  beforeEach(() => {
    resetMemoryLibrary();
    resetMemorySession();
    resetSquare();
  });

  it("queues a favorite when auto-sync is on and the request fails", async () => {
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async () => {
      throw new Error("收藏失败");
    });
    const result = await favoriteWithQueue("sq-1", "PUT");
    expect(result.queued).toBe(true);
    expect(await listSyncQueue()).toEqual([
      { kind: "favorite", method: "PUT", id: "sq-1", email: "dev@promptark.local" },
    ]);
    expect(await listLocalPrompts({ query: "" })).toHaveLength(0);
  });

  it("does not queue a favorite when auto-sync is off", async () => {
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    setFavoriteTransport(async () => {
      throw new Error("收藏失败");
    });
    await expect(favoriteWithQueue("sq-1", "PUT")).rejects.toThrow("收藏失败");
    expect(await listSyncQueue()).toEqual([]);
  });

  it("flushes a queued favorite when the transport recovers", async () => {
    const calls = [];
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async () => {
      throw new Error("收藏失败");
    });
    await favoriteWithQueue("sq-1", "PUT");
    setFavoriteTransport(async (request) => {
      calls.push(request);
      return { id: request.id };
    });
    await flushSyncQueue();
    expect(calls).toEqual([{ method: "PUT", id: "sq-1" }]);
    expect(await listSyncQueue()).toEqual([]);
  });

  it("keeps the later favorite write for the same id", async () => {
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async () => {
      throw new Error("收藏失败");
    });
    await favoriteWithQueue("sq-1", "PUT");
    await favoriteWithQueue("sq-1", "DELETE");
    expect(await listSyncQueue()).toEqual([
      { kind: "favorite", method: "DELETE", id: "sq-1", email: "dev@promptark.local" },
    ]);
  });

  it("queues a publication snapshot when auto-sync is on and the request fails", async () => {
    const created = await createLocalPrompt({ title: "本地源", content: "旧正文" });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setPublishTransport(async () => {
      throw new Error("发布失败");
    });
    const result = await publishWithQueue({
      sourceId: created.id,
      title: created.title,
      content: created.content,
    });
    expect(result.queued).toBe(true);
    expect(await listSyncQueue()).toEqual([
      {
        kind: "publish",
        sourceId: created.id,
        title: "本地源",
        content: "旧正文",
        email: "dev@promptark.local",
      },
    ]);
    const listed = await listLocalPrompts({ query: "本地源" });
    expect(listed[0].content).toBe("旧正文");
  });

  it("flushes a queued publication when the transport recovers", async () => {
    const created = await createLocalPrompt({ title: "本地源", content: "旧正文" });
    const calls = [];
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setPublishTransport(async () => {
      throw new Error("发布失败");
    });
    await publishWithQueue({
      sourceId: created.id,
      title: created.title,
      content: created.content,
    });
    setPublishTransport(async (payload) => {
      calls.push(payload);
      return { id: "pub-1", source_id: payload.sourceId, status: "pending" };
    });
    await flushSyncQueue();
    expect(calls).toEqual([{ sourceId: created.id, title: "本地源", content: "旧正文" }]);
    expect(await listSyncQueue()).toEqual([]);
  });
});
