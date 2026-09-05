import { afterEach, expect, it, vi } from "vitest";
import { createLocalPrompt, listLocalPrompts, addPromptToCollection, exportLocalSyncChanges, applyLocalSyncChanges } from "./library.js";
import { invokeCommand } from "./tauri.js";
import { downloadSquareItem, fetchSquareContent, resetSquare, setSquareContentTransport } from "./square.js";

const invoke = vi.hoisted(() => vi.fn(async () => []));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
afterEach(() => { delete window.__TAURI_INTERNALS__; invoke.mockClear(); resetSquare(); });

it("sends a whole downloaded collection to the native transactional import", async () => {
  window.__TAURI_INTERNALS__ = {};
  setSquareContentTransport(async () => ({ id: "remote", kind: "collection", title: "合集", members: [{ title: "成员", content: "正文", model: "Flux" }] }));
  await downloadSquareItem("remote");
  const call = invoke.mock.calls.find(([command]) => command === "apply_local_import");
  const payload = JSON.parse(call[1].json);
  expect(payload.collections).toHaveLength(1);
  expect(payload.prompts[0]).toMatchObject({ collection_id: payload.collections[0].id, source: "downloaded", model: "Flux" });
  expect(invoke.mock.calls.filter(([command]) => command === "apply_local_import")).toHaveLength(1);
});

it("reads native square detail without invoking the download write command", async () => {
  window.__TAURI_INTERNALS__ = {};
  await fetchSquareContent("sq-1");
  expect(invoke).toHaveBeenCalledExactlyOnceWith("get_square_content", { id: "sq-1" });
});

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

it("uses native snapshot commands including tombstones and keep-local", async () => {
  window.__TAURI_INTERNALS__ = {};
  const items = [{ id: "p", kind: "prompt", payload: { collection_id: "col" }, updated_at: "2", deleted_at: "2" }];
  await exportLocalSyncChanges();
  expect(invoke).toHaveBeenLastCalledWith("export_local_sync_changes", undefined);
  await applyLocalSyncChanges(items, { keepLocal: true });
  expect(invoke).toHaveBeenLastCalledWith("apply_local_sync_changes", { items, keepLocal: true });
});
