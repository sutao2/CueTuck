import { beforeEach, describe, expect, it } from "vitest";
import { createLocalCollection, createLocalPrompt, resetMemoryLibrary, setLocalSetting } from "./library.js";
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
