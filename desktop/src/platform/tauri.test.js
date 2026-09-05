import { afterEach, expect, it, vi } from "vitest";
import { createLocalPrompt, listLocalPrompts, addPromptToCollection } from "./library.js";
import { invokeCommand } from "./tauri.js";

const invoke = vi.hoisted(() => vi.fn(async () => []));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
afterEach(() => { delete window.__TAURI_INTERNALS__; invoke.mockClear(); });

it("sends category and collection arguments in the native command schema", async () => {
  window.__TAURI_INTERNALS__ = {};
  await createLocalPrompt({ title: "人像", content: "光影", categoryId: "cat-image-0" });
  expect(invoke).toHaveBeenLastCalledWith("create_local_prompt", {
    title: "人像", content: "光影", categoryId: "cat-image-0", model: null,
  });
  await listLocalPrompts({ categoryId: "cat-image" });
  expect(invoke).toHaveBeenLastCalledWith("list_local_prompts", { query: "", categoryId: "cat-image" });
  await addPromptToCollection("p", "c");
  expect(invoke).toHaveBeenLastCalledWith("add_prompt_to_local_collection", { promptId: "p", collectionId: "c" });
});

it("converts auth arguments without rewriting nested sync payloads", async () => {
  const items = [{ payload: { category_id: "cat-image-0" } }];
  await invokeCommand("put_library_changes", { access_token: "test", items });
  expect(invoke).toHaveBeenLastCalledWith("put_library_changes", { accessToken: "test", items });
});
