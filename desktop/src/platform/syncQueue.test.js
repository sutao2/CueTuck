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

  it("replays collection kind and immutable member snapshots from the offline queue", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "acc" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setPublishTransport(async () => { throw new Error("offline"); });
    const members = [{ title: "成员", content: "原始正文", model: "Flux" }];
    await publishWithQueue({ sourceId: "collection", kind: "collection", title: "合集", members });
    members[0].content = "后来修改";
    let seen;
    setPublishTransport(async (payload) => { seen = payload; return { status: "pending" }; });
    await flushSyncQueue();
    expect(seen.kind).toBe("collection");
    expect(seen.members[0].content).toBe("原始正文");
    expect(await listSyncQueue()).toEqual([]);
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

  it("does not replay an old favorite after a newer successful delete", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async () => { throw new Error("offline"); });
    await favoriteWithQueue("sq-1", "PUT");
    const calls = [];
    setFavoriteTransport(async (request) => { calls.push(request.method); return {}; });
    await favoriteWithQueue("sq-1", "DELETE");
    expect(calls).toEqual(["DELETE"]);
    expect(await listSyncQueue()).toEqual([]);
  });

  it("does not lose concurrent offline operations for different items", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async () => { throw new Error("offline"); });
    await Promise.all([favoriteWithQueue("a", "PUT"), favoriteWithQueue("b", "PUT")]);
    expect((await listSyncQueue()).map((row) => row.id).sort()).toEqual(["a", "b"]);
  });

  it("stops flushing when the signed-in account changes", async () => {
    setSessionTransport(async ({ email }) => ({ email, access_token: email }));
    await loginSession({ email: "a@example.com", password: "test" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async () => { throw new Error("offline"); });
    await favoriteWithQueue("a", "PUT");
    await favoriteWithQueue("b", "PUT");
    const calls = [];
    setFavoriteTransport(async ({ id }) => {
      calls.push(id);
      await loginSession({ email: "b@example.com", password: "test" });
      return {};
    });
    await flushSyncQueue();
    expect(calls).toEqual(["a"]);
    expect(await listSyncQueue()).toEqual([{ kind: "favorite", id: "b", method: "PUT", email: "a@example.com" }]);
  });

  it("does not submit a superseded publication draft after the new snapshot succeeds", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setPublishTransport(async () => { throw new Error("offline"); });
    await publishWithQueue({ sourceId: "p", title: "旧版", content: "旧文" });
    const titles = [];
    setPublishTransport(async (payload) => { titles.push(payload.title); return { id: "pub" }; });
    await publishWithQueue({ sourceId: "p", title: "新版", content: "新文" });
    expect(titles).toEqual(["新版"]);
    expect(await listSyncQueue()).toEqual([]);
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
