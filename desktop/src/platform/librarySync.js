import {
  insertSyncedLocalPrompt,
  listLocalCategories,
  listLocalCollections,
  listLocalPrompts,
} from "./library.js";
import { getSession } from "./session.js";

let testTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function tauriInvoke(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}

function apiBase() {
  return "http://127.0.0.1:8787";
}

function requireAccessToken() {
  const token = getSession().accessToken;
  if (!token) throw new Error("同步需要登录");
  return token;
}

function asChange(kind, row) {
  return {
    id: row.id,
    kind,
    payload: { ...row },
    updated_at: String(row.updated_at ?? "0"),
  };
}

export function resetLibrarySync() {
  testTransport = null;
}

export function setLibrarySyncTransport(transport) {
  testTransport = transport;
}

export async function putLibraryChanges(items) {
  if (testTransport?.put) return testTransport.put(items);
  const token = requireAccessToken();
  if (isTauri()) {
    return tauriInvoke("put_library_changes", { access_token: token, items });
  }
  const response = await fetch(`${apiBase()}/v1/library/changes`, {
    method: "PUT",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ items }),
  });
  if (!response.ok) throw new Error("同步失败");
  return response.json();
}

export async function listLibraryChanges({ since = "" } = {}) {
  if (testTransport?.get) return testTransport.get({ since });
  const token = requireAccessToken();
  if (isTauri()) {
    return tauriInvoke("list_library_changes", { access_token: token, since });
  }
  const params = new URLSearchParams({ since });
  const response = await fetch(`${apiBase()}/v1/library/changes?${params}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) throw new Error("同步失败");
  return response.json();
}

async function applyRemotePromptChanges(items) {
  const local = await listLocalPrompts({ query: "" });
  const ids = new Set(local.map((row) => row.id));
  for (const item of items) {
    if (item.kind !== "prompt" || item.deleted_at || ids.has(item.id)) continue;
    await insertSyncedLocalPrompt({
      id: item.id,
      title: item.payload?.title ?? "",
      content: item.payload?.content ?? "",
      categoryId: item.payload?.category_id ?? null,
      updatedAt: item.updated_at,
    });
  }
}

export async function syncLocalLibraryNow() {
  requireAccessToken();
  const [prompts, collections, categories] = await Promise.all([
    listLocalPrompts({ query: "" }),
    listLocalCollections({ query: "" }),
    listLocalCategories(),
  ]);
  const items = [
    ...prompts.map((row) => asChange("prompt", row)),
    ...collections.map((row) => asChange("collection", row)),
    ...categories.map((row) => asChange("category", row)),
  ];
  await putLibraryChanges(items);
  const remote = await listLibraryChanges({ since: "" });
  await applyRemotePromptChanges(remote.items ?? []);
  return remote;
}
