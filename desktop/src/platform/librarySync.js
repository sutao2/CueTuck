import {
  getLocalSetting,
  exportLocalSyncChanges,
  applyLocalSyncChanges,
  timestampMillis,
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

export async function syncLocalLibraryNow() {
  const token = requireAccessToken();
  const assertSameSession = () => {
    if (getSession().accessToken !== token) throw new Error("登录状态已改变，请重新同步");
  };
  const snapshot = await exportLocalSyncChanges();
  const keepLocal = (await getLocalSetting("sync_conflict")) === "keep_local";
  const skipImages = await shouldSkipImageAssets();
  const remoteById = new Map();
  if (skipImages) {
    const existing = await listLibraryChanges({ since: "" });
    for (const item of existing.items ?? []) {
      if (item.kind === "collection") remoteById.set(item.id, item);
    }
  }
  const items = snapshot.map((item) => item.kind === "collection"
    ? { ...item, payload: collectionPayloadForPush({ ...item.payload, id: item.id }, skipImages, remoteById) }
    : item);
  assertSameSession();
  await putLibraryChanges(items);
  assertSameSession();
  const remote = await listLibraryChanges({ since: "" });
  assertSameSession();
  const localCollections = new Map(snapshot.filter((item) => item.kind === "collection").map((item) => [item.id, item]));
  const incoming = (remote.items ?? []).map((item) => item.kind === "collection" && skipImages
    ? { ...item, payload: collectionPayloadForPush({ ...item.payload, id: item.id }, true, localCollections) }
    : item);
  await applyLocalSyncChanges(incoming, { keepLocal });
  if (skipImages) {
    // Keep withheld local cover edits newer than the accepted text-only revision,
    // so the next Wi-Fi sync actually uploads them instead of losing an equal-time conflict.
    const remoteCollections = new Map((remote.items ?? []).filter((item) => item.kind === "collection").map((item) => [item.id, item]));
    const deferred = snapshot.filter((item) => {
      const server = remoteCollections.get(item.id);
      return item.kind === "collection" && server && timestampMillis(item.updated_at) >= timestampMillis(server.updated_at)
        && (item.payload.cover_json !== server.payload.cover_json || item.payload.cover_type !== server.payload.cover_type);
    }).map((item) => ({ ...item, updated_at: String(timestampMillis(item.updated_at) + 1) }));
    await applyLocalSyncChanges(deferred);
  }
  return remote;
}
