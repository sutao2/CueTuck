import { beforeEach, expect, it } from "vitest";
import { loadAccountLibrary, pushAccountPrompt, resetAccountLibrary, setAccountLibraryTransport } from "./accountLibrary.js";
import { listLocalPrompts, resetMemoryLibrary, updateLocalPrompt } from "./memoryLibrary.js";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";

beforeEach(() => { resetMemoryLibrary(); resetMemorySession(); resetAccountLibrary(); });

it("preserves desktop membership and metadata when editing an account prompt", async () => {
  let pushed;
  setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
  await loginSession({ email: "dev@promptark.local", password: "devpass" });
  setAccountLibraryTransport({
    get: async () => ({ items: [{ id: "p", kind: "prompt", updated_at: "1700000001", payload: {
      title: "原文", content: "内容", category_id: "custom", collection_id: "col", model: "Flux",
      source: "downloaded", remote_id: "sq", author: "作者", use_count: 7,
    } }] }),
    put: async (items) => { pushed = items[0]; return { items }; },
  });
  await loadAccountLibrary();
  const edited = updateLocalPrompt({ id: "p", title: "改后", content: "新正文" });
  await pushAccountPrompt(edited);
  expect(pushed.payload).toMatchObject({ title: "改后", collection_id: "col", category_id: "custom", model: "Flux", source: "downloaded", remote_id: "sq", author: "作者", use_count: 7 });
  expect(pushed.payload).not.toHaveProperty("original_source");
  expect(Number(pushed.updated_at)).toBeGreaterThan(1700000001000);
});

it("discards late account reads after switching accounts", async () => {
  setSessionTransport(async ({ email }) => ({ email, access_token: email }));
  await loginSession({ email: "a@local", password: "pass" });
  let resolveA;
  setAccountLibraryTransport({ get: () => new Promise((resolve) => { resolveA = resolve; }) });
  const a = loadAccountLibrary();
  const rejected = expect(a).rejects.toThrow("读取失败");
  await loginSession({ email: "b@local", password: "pass" });
  setAccountLibraryTransport({ get: async () => ({ items: [{ id: "b", kind: "prompt", payload: { title: "B私有", content: "B" } }] }) });
  await loadAccountLibrary();
  resolveA({ items: [{ id: "a", kind: "prompt", payload: { title: "A私有", content: "A" } }] });
  await rejected;
  expect(listLocalPrompts().map((row) => row.title)).toEqual(["B私有"]);
});
