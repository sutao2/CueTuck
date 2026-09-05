import { beforeEach, describe, expect, it } from "vitest";
import { addPromptToCollection, applyLocalSyncChanges, createLocalCategory, createLocalCollection, createLocalPrompt, deleteLocalPrompt, exportLocalSyncChanges, getLocalSetting, listCollectionMembers, listLocalCategories, listLocalCollections, listLocalPrompts, resetMemoryLibrary, setLocalSetting, timestampMillis } from "./library.js";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";
import {
  resetLibrarySync,
  setLibrarySyncTransport,
  setNetworkType,
  syncLocalLibraryNow,
} from "./librarySync.js";

describe("library sync", () => {
  beforeEach(() => {
    resetMemoryLibrary();
    resetMemorySession();
    resetLibrarySync();
  });

  it("restores a second device including custom categories, collections, models and tombstones", async () => {
    const account = new Map();
    setLibrarySyncTransport({
      put: async (items) => {
        for (const item of items) {
          if (!account.has(item.id) || timestampMillis(item.updated_at) > timestampMillis(account.get(item.id).updated_at)) account.set(item.id, structuredClone(item));
        }
        return { items: [...account.values()] };
      },
      get: async () => ({ items: [...account.values()].reverse() }),
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const category = await createLocalCategory({ name: "自定义", parentId: "cat-image" });
    const collection = await createLocalCollection({ title: "合集", categoryId: category.id });
    const prompt = await createLocalPrompt({ title: "成员", content: "正文", categoryId: category.id, model: "Flux" });
    await addPromptToCollection(prompt.id, collection.id);
    await setLocalSetting("theme", "dark");
    await setLocalSetting("manual_proxy", "private");
    await syncLocalLibraryNow();
    expect(account.has("setting:manual_proxy")).toBe(false);
    resetMemoryLibrary();
    await syncLocalLibraryNow();
    expect((await listLocalCategories()).some((row) => row.id === category.id)).toBe(true);
    expect(await listCollectionMembers(collection.id)).toEqual([expect.objectContaining({ id: prompt.id, model: "Flux", category_id: category.id })]);
    expect((await listLocalCollections())[0].member_count).toBe(1);
    expect(await getLocalSetting("theme")).toBe("dark");
    await deleteLocalPrompt(prompt.id);
    await syncLocalLibraryNow();
    expect(account.get(prompt.id).deleted_at).toBeTruthy();
    resetMemoryLibrary();
    await syncLocalLibraryNow();
    expect(await listLocalPrompts()).toEqual([]);
    expect((await listLocalCollections())[0].member_count).toBe(0);
  });

  it("rolls back orphan references and compares legacy seconds numerically", async () => {
    const old = { id: "p", kind: "prompt", payload: { title: "旧" }, updated_at: "1700000000000" };
    await expect(applyLocalSyncChanges([old, { ...old, id: "bad", payload: { title: "坏", category_id: "missing" } }])).rejects.toThrow(/不存在/);
    expect(await listLocalPrompts()).toEqual([]);
    await applyLocalSyncChanges([old]);
    await applyLocalSyncChanges([{ ...old, payload: { title: "新" }, updated_at: "1700000001" }]);
    expect((await listLocalPrompts())[0].title).toBe("新");
    const snapshot = await exportLocalSyncChanges();
    expect(snapshot.find((row) => row.id === "p").updated_at).toBe("1700000001000");
  });

  it("does not apply an in-flight response after logout", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    setLibrarySyncTransport({
      put: async () => ({ items: [] }),
      get: async () => {
        resetMemorySession();
        return { items: [{ id: "private", kind: "prompt", payload: { title: "私有" }, updated_at: "1" }] };
      },
    });
    await expect(syncLocalLibraryNow()).rejects.toThrow(/登录状态/);
    expect(await listLocalPrompts()).toEqual([]);
  });

  it("retries deferred cover upload and download after returning to wifi", async () => {
    const account = new Map();
    setLibrarySyncTransport({
      put: async (items) => {
        for (const item of items) {
          if (!account.has(item.id) || timestampMillis(item.updated_at) > timestampMillis(account.get(item.id).updated_at)) account.set(item.id, structuredClone(item));
        }
        return { items: [...account.values()] };
      },
      get: async () => ({ items: [...account.values()] }),
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("sync_wifi_images", "1");
    setNetworkType("cellular");
    const collection = await createLocalCollection({ title: "封面", coverType: "single", coverUrls: ["local.png"] });
    await syncLocalLibraryNow();
    expect(account.get(collection.id).payload.cover_json).toBe("[]");
    expect((await listLocalCollections())[0].cover_json).toBe('["local.png"]');
    setNetworkType("wifi");
    await syncLocalLibraryNow();
    expect(account.get(collection.id).payload.cover_json).toBe('["local.png"]');
    resetMemoryLibrary();
    await setLocalSetting("sync_wifi_images", "1");
    setNetworkType("cellular");
    await syncLocalLibraryNow();
    expect((await listLocalCollections())[0].cover_json).toBe("[]");
    setNetworkType("wifi");
    await syncLocalLibraryNow();
    expect((await listLocalCollections())[0].cover_json).toBe('["local.png"]');
  });

  it("puts the local prompt onto the account library when signed in", async () => {
    const account = [];
    setLibrarySyncTransport({
      put: async (items) => {
        account.splice(0, account.length, ...items);
        return { items: account };
      },
      get: async () => ({ items: account }),
    });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await createLocalPrompt({ title: "本地仍在", content: "正文" });
    await syncLocalLibraryNow();
    expect(account.some((row) => row.payload?.title === "本地仍在")).toBe(true);
  });

  it("does not call the library API when signed out", async () => {
    let called = false;
    setLibrarySyncTransport({
      put: async () => {
        called = true;
        return { items: [] };
      },
      get: async () => {
        called = true;
        return { items: [] };
      },
    });
    await expect(syncLocalLibraryNow()).rejects.toThrow(/登录/);
    expect(called).toBe(false);
  });

  it("applies the remote body when the remote updated_at is newer", async () => {
    const { insertSyncedLocalPrompt, listLocalPrompts } = await import("./library.js");
    setLibrarySyncTransport({
      put: async (items) => ({ items }),
      get: async () => ({
        items: [
          {
            id: "p-1",
            kind: "prompt",
            payload: { title: "本地仍在", content: "远端正文" },
            updated_at: "2",
          },
        ],
      }),
    });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await insertSyncedLocalPrompt({
      id: "p-1",
      title: "本地仍在",
      content: "本机正文",
      updatedAt: "1",
    });
    await syncLocalLibraryNow();
    const rows = await listLocalPrompts({ query: "本地仍在" });
    expect(rows[0].content).toBe("远端正文");
  });

  it("keeps the local body when keep-local is chosen and remote updated_at is newer", async () => {
    const { insertSyncedLocalPrompt, listLocalPrompts } = await import("./library.js");
    await setLocalSetting("sync_conflict", "keep_local");
    setLibrarySyncTransport({
      put: async (items) => ({ items }),
      get: async () => ({
        items: [
          {
            id: "p-1",
            kind: "prompt",
            payload: { title: "本地仍在", content: "远端正文" },
            updated_at: "2",
          },
          {
            id: "p-2",
            kind: "prompt",
            payload: { title: "远端独有", content: "新条目" },
            updated_at: "3",
          },
        ],
      }),
    });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await insertSyncedLocalPrompt({
      id: "p-1",
      title: "本地仍在",
      content: "本机正文",
      updatedAt: "1",
    });
    await syncLocalLibraryNow();
    const local = await listLocalPrompts({ query: "本地仍在" });
    expect(local[0].content).toBe("本机正文");
    const remoteOnly = await listLocalPrompts({ query: "远端独有" });
    expect(remoteOnly[0].content).toBe("新条目");
  });

  it("skips collection covers when wifi-only is on and the network is not wifi", async () => {
    const putItems = [];
    setLibrarySyncTransport({
      put: async (items) => {
        putItems.splice(0, putItems.length, ...items);
        return { items };
      },
      get: async () => ({ items: [] }),
    });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("sync_wifi_images", "1");
    setNetworkType("unknown");
    await createLocalCollection({
      title: "人像灵感",
      coverType: "grid",
      coverUrls: ["one.jpg", "two.jpg"],
    });
    await createLocalPrompt({ title: "本地仍在", content: "正文" });
    await syncLocalLibraryNow();
    const collection = putItems.find((row) => row.kind === "collection");
    expect(collection?.payload?.title).toBe("人像灵感");
    expect(JSON.parse(collection?.payload?.cover_json || "[]")).toEqual([]);
    expect(putItems.some((row) => row.payload?.title === "本地仍在" && row.payload?.content === "正文")).toBe(true);
  });

  it("keeps remote covers when wifi-only skip would otherwise wipe them", async () => {
    let putItems = [];
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("sync_wifi_images", "1");
    setNetworkType("cellular");
    const created = await createLocalCollection({
      title: "新标题",
      coverType: "grid",
      coverUrls: ["local.jpg"],
    });
    const account = [
      {
        id: created.id,
        kind: "collection",
        payload: {
          title: "旧标题",
          cover_json: JSON.stringify(["remote.jpg"]),
          cover_type: "single",
        },
        updated_at: "1",
      },
    ];
    setLibrarySyncTransport({
      put: async (items) => {
        putItems = items;
        return { items };
      },
      get: async () => ({ items: account }),
    });
    await syncLocalLibraryNow();
    const collection = putItems.find((row) => row.id === created.id);
    expect(collection?.payload?.title).toBe("新标题");
    expect(JSON.parse(collection?.payload?.cover_json || "[]")).toEqual(["remote.jpg"]);
  });

  it("sends collection covers when wifi-only is on and the network is wifi", async () => {
    const putItems = [];
    setLibrarySyncTransport({
      put: async (items) => {
        putItems.splice(0, putItems.length, ...items);
        return { items };
      },
      get: async () => ({ items: [] }),
    });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("sync_wifi_images", "1");
    setNetworkType("wifi");
    await createLocalCollection({
      title: "人像灵感",
      coverType: "grid",
      coverUrls: ["one.jpg"],
    });
    await syncLocalLibraryNow();
    const collection = putItems.find((row) => row.kind === "collection");
    expect(JSON.parse(collection?.payload?.cover_json || "[]")).toEqual(["one.jpg"]);
  });

  it("sends collection covers when wifi-only is off even if the network is unknown", async () => {
    const putItems = [];
    setLibrarySyncTransport({
      put: async (items) => {
        putItems.splice(0, putItems.length, ...items);
        return { items };
      },
      get: async () => ({ items: [] }),
    });
    setSessionTransport(async () => ({
      email: "dev@promptark.local",
      access_token: "tok",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    setNetworkType("unknown");
    await createLocalCollection({
      title: "人像灵感",
      coverType: "grid",
      coverUrls: ["one.jpg"],
    });
    await syncLocalLibraryNow();
    const collection = putItems.find((row) => row.kind === "collection");
    expect(JSON.parse(collection?.payload?.cover_json || "[]")).toEqual(["one.jpg"]);
  });
});
