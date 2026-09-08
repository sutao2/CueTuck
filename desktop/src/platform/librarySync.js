import {
  getLocalSetting,
  exportLocalSyncChanges,
  applyLocalSyncChanges,
  timestampMillis,
} from "./library.js";
import { getSession } from "./session.js";
import { validateAssets, assetSize } from './assets.js';
import { uploadPrivateAsset, downloadPrivateAsset, assetHash, validateReferences, verifyAsset } from './privateMedia.js';

let testTransport = null;
let testNetworkType = null;
let syncing = false;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

import { apiBase } from '../../../shared/apiBase.js';

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

export async function syncLocalLibraryNow(options = {}) {
  if (syncing) throw new Error('同步正在进行，请稍候');
  syncing = true;
  try { return await runLibrarySync(options); }
  finally { syncing = false; }
}

async function runLibrarySync({ includeAssets = false, onProgress = () => {} } = {}) {
  const token = requireAccessToken();
  const assertSameSession = () => {
    if (getSession().accessToken !== token) throw new Error("登录状态已改变，请重新同步");
  };
  const keepLocal = (await getLocalSetting("sync_conflict")) === "keep_local";
  const skipImages = await shouldSkipImageAssets();
  const transferAssets = includeAssets && !skipImages;
  const snapshot = await exportLocalSyncChanges({ includeAssets: transferAssets });
  assertSameSession();
  const remoteById = new Map();
  if (skipImages || transferAssets) {
    const existing = await listLibraryChanges({ since: "" });
    assertSameSession();
    for (const item of existing.items ?? []) {
      remoteById.set(item.id, item);
    }
  }
  const items = snapshot.map((item) => item.kind === "collection"
    ? { ...item, payload: collectionPayloadForPush({ ...item.payload, id: item.id }, skipImages, remoteById) }
    : { ...item, payload: { ...item.payload } });
  for (let index = 0; index < items.length; index++) {
    const item = items[index];
    delete item.payload.assets;
    if (!transferAssets || item.kind !== 'prompt' || item.deleted_at) continue;
    const server = remoteById.get(item.id);
    // A remote tombstone must not be resurrected by an attachment-only change.
    if (server?.deleted_at && timestampMillis(server.updated_at) >= timestampMillis(item.updated_at)) continue;
    const refs = validateReferences(structuredClone(server?.payload.asset_refs ?? []));
    const assets = snapshot[index].payload.assets ?? [];
    validateAssets(assets);
    for (const asset of assets) {
      assertSameSession();
      const existing = refs.find(ref => ref.id === asset.id);
      if (existing) {
        // UUID identifies a local file; never silently replace a different file with the same UUID.
        if (existing.sha256 !== await assetHash(asset) || existing.name !== asset.name || existing.mime !== asset.mime || existing.size !== assetSize(asset)) throw new Error('两端附件标识冲突，请重新添加该附件后同步');
        continue;
      }
      if (refs.length >= 12 || refs.reduce((n, ref) => n + ref.size, assetSize(asset)) > 20 * 1024 * 1024) throw new Error('合并后附件超限，请先整理附件');
      onProgress(`上传附件：${asset.name}`);
      assertSameSession();
      const ref = await (testTransport?.upload ?? uploadPrivateAsset)(asset, token);
      assertSameSession();
      await verifyAsset(asset, ref);
      refs.push(ref);
    }
    if (refs.length && JSON.stringify(refs) !== JSON.stringify(server?.payload.asset_refs)) {
      const winner = server && timestampMillis(server.updated_at) > timestampMillis(item.updated_at) ? server : item;
      items[index] = { ...winner, payload: { ...winner.payload, asset_refs: refs }, updated_at: String(Math.max(timestampMillis(item.updated_at), timestampMillis(server?.updated_at)) + 1) };
    } else if (server?.payload.asset_refs) item.payload.asset_refs = refs;
  }
  assertSameSession();
  onProgress('同步提示词与分类…');
  await putLibraryChanges(items);
  assertSameSession();
  const remote = await listLibraryChanges({ since: "" });
  assertSameSession();
  const localCollections = new Map(snapshot.filter((item) => item.kind === "collection").map((item) => [item.id, item]));
  const incoming = (remote.items ?? []).map((item) => item.kind === "collection" && skipImages
    ? { ...item, payload: collectionPayloadForPush({ ...item.payload, id: item.id }, true, localCollections) }
    : { ...item, payload: { ...item.payload } });
  const localPrompts = new Map(snapshot.filter(item => item.kind === 'prompt').map(item => [item.id, item]));
  for (const item of incoming) {
    // Even a server response cannot opt itself into transferring raw file bytes.
    delete item.payload.assets;
    if (!transferAssets || item.kind !== 'prompt' || item.deleted_at || !item.payload.asset_refs) continue;
    const local = localPrompts.get(item.id);
    if (local && (keepLocal || timestampMillis(local.updated_at) > timestampMillis(item.updated_at))) continue;
    const assets = [];
    for (const ref of validateReferences(item.payload.asset_refs)) {
      assertSameSession();
      const cached = local?.payload.assets?.find(asset => asset.id === ref.id);
      onProgress(`校验附件：${ref.name}`);
      const asset = cached ?? await (testTransport?.download ?? downloadPrivateAsset)(ref, token);
      assertSameSession();
      assets.push(await verifyAsset(asset, ref));
    }
    item.payload.assets = assets;
  }
  assertSameSession();
  await applyLocalSyncChanges(incoming, { keepLocal, includeAssets: transferAssets });
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
  return { ...remote, attachmentsDeferred: includeAssets && skipImages };
}
