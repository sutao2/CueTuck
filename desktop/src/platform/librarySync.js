import {
  getLocalSetting,
  insertSyncedLocalPrompt,
  listLocalCategories,
  listLocalCollections,
  listLocalPrompts,
} from "./library.js";
import { getSession } from "./session.js";

let testTransport = null;
let testNetworkType = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

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
  testNetworkType = null;
}

export function setLibrarySyncTransport(transport) {
  testTransport = transport;
}

export function setNetworkType(type) {
  testNetworkType = type;
}

export function detectNetworkType() {
  if (testNetworkType != null) return testNetworkType;
  const connection =
    typeof navigator !== "undefined"
      ? navigator.connection || navigator.mozConnection || navigator.webkitConnection
      : null;
  const type = connection?.type;
  if (typeof type === "string" && type.trim()) return type.trim().toLowerCase();
  return "unknown";
}

function collectionPayloadForPush(row, skipImages, remoteById) {
  if (!skipImages) return row;
  const remotePayload = remoteById.get(row.id)?.payload ?? {};
  return {
    ...row,
    cover_json: remotePayload.cover_json ?? "[]",
    cover_type: remotePayload.cover_type ?? "none",
  };
}

async function shouldSkipImageAssets() {
  if ((await getLocalSetting("sync_wifi_images")) !== "1") return false;
  return detectNetworkType() !== "wifi";
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
  const keepLocal = (await getLocalSetting("sync_conflict")) === "keep_local";
  const existingIds = keepLocal
    ? new Set((await listLocalPrompts({ query: "" })).map((row) => row.id))
    : null;
  for (const item of items) {
    if (item.kind !== "prompt" || item.deleted_at) continue;
    if (keepLocal && existingIds.has(item.id)) continue;
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
  const skipImages = await shouldSkipImageAssets();
  const remoteById = new Map();
  if (skipImages) {
    const existing = await listLibraryChanges({ since: "" });
    for (const item of existing.items ?? []) {
      if (item.kind === "collection") remoteById.set(item.id, item);
    }
  }
  const items = [
    ...prompts.map((row) => asChange("prompt", row)),
    ...collections.map((row) => asChange("collection", collectionPayloadForPush(row, skipImages, remoteById))),
    ...categories.map((row) => asChange("category", row)),
  ];
  await putLibraryChanges(items);
  const remote = await listLibraryChanges({ since: "" });
  await applyRemotePromptChanges(remote.items ?? []);
  return remote;
}
