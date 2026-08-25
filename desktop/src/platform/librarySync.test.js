import { beforeEach, describe, expect, it } from "vitest";
import { createLocalPrompt, resetMemoryLibrary } from "./library.js";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";
import { resetLibrarySync, setLibrarySyncTransport, syncLocalLibraryNow } from "./librarySync.js";

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
});
