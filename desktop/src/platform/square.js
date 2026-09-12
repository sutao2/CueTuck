import { applyLocalImport, importDownloadedPrompt, getLocalSetting, listLocalCategories, listLocalPrompts, appendDownloadedAssets } from "./library.js";
import { downloadReferenceImages } from './referenceAssets.js';
import { referenceImages } from '../lib/squareReference.js';
import { validateAssets } from './assets.js';
import { getSession } from "./session.js";
import { downloadPublishedAsset, validateReferences, validateCollectionAssets } from './privateMedia.js';

let testTransport = null;
let testContentTransport = null;
let testPublishTransport = null;
let testFavoriteTransport = null;
let testMineTransport = null;
let testStatsTransport = null;
let testCatalogTransport = null;
let testPageTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

import { apiBase } from '../../../shared/apiBase.js';

export function resetSquare() {
  testTransport = null;
  testContentTransport = null;
  testPublishTransport = null;
  testFavoriteTransport = null;
  testMineTransport = null;
  testStatsTransport = null;
  testCatalogTransport = null;
  testPageTransport = null;
}

export function setSquarePageTransport(transport) { testPageTransport = transport; }
export async function listSquarePage({ sort = '推荐', query = '', model = '', categoryId = null, offset = 0, signal } = {}) {
  signal?.throwIfAborted();
  let payload;
  if (testPageTransport) payload = await testPageTransport({ sort, query, model, categoryId, offset, signal });
  else if (testTransport || testFavoriteTransport) {
    // Legacy fixtures only; production must never fall back to the unbounded endpoint.
    const favorites = testFavoriteTransport && getSession().loggedIn ? await listFavorites() : [];
    const rows = sort === '收藏' ? favorites.filter(row => (!model || row.model === model) && (!query || row.title.toLowerCase().includes(query.toLowerCase()))) : await listSquareItems({ sort, query, model, categoryId });
    payload = { items: rows.slice(offset, offset + 48).map(row => ({ ...row, is_favorite: favorites.some(f => f.id === row.id) })), total: rows.length, next_offset: offset + 48 < rows.length ? offset + 48 : null };
  } else if (isTauri()) {
    const requestId = crypto.randomUUID();
    const cancel = () => { tauriInvoke('cancel_square_page', { request_id: requestId }).catch(() => {}); };
    signal?.addEventListener('abort', cancel, { once: true });
    try {
      payload = await tauriInvoke('list_square_page', { sort, query, model, category_id: categoryId, offset, request_id: requestId, access_token: getSession().accessToken || null });
    } finally { signal?.removeEventListener('abort', cancel); }
  } else {
    const params = new URLSearchParams({ sort, q: query, offset: String(offset), limit: '48' });
    if (model) params.set('model', model);
    if (categoryId) params.set('category_id', categoryId);
    const token = getSession().accessToken;
    const controller = new AbortController();
    const abort = () => controller.abort();
    signal?.addEventListener('abort', abort, { once: true });
    const timer = setTimeout(abort, 10_000);
    try {
      const response = await fetch(`${apiBase()}/v1/square/browse?${params}`, { signal: controller.signal, headers: token ? { Authorization: `Bearer ${token}` } : {} });
      if (!response.ok) throw new Error('广场暂时不可用');
      payload = await response.json();
    } finally { clearTimeout(timer); signal?.removeEventListener('abort', abort); }
  }
  signal?.throwIfAborted();
  if (!Array.isArray(payload?.items) || payload.items.length > 48 || !Number.isInteger(payload.total) || payload.total < 0
    || (payload.next_offset !== null && (!Number.isInteger(payload.next_offset) || payload.next_offset <= offset || !payload.items.length))) throw new Error('广场分页响应无效');
  return payload;
}

export function setSquareTransport(transport) {
  testTransport = transport;
}
export function setCatalogTransport(transport) { testCatalogTransport = transport; }
export async function fetchSquareCatalog() {
  let payload;
  if (testCatalogTransport) payload = await testCatalogTransport();
  else if (isTauri()) payload = await tauriInvoke('get_square_catalog');
  else {
    const response = await fetch(`${apiBase()}/v1/square/catalog`);
    if (!response.ok) throw new Error('广场分类与模型配置暂时不可用');
    payload = await response.json();
  }
  if (!Array.isArray(payload?.categories) || !Array.isArray(payload?.models)) throw new Error('广场字典响应无效');
  return payload;
}

export function setSquareContentTransport(transport) {
  testContentTransport = transport;
}

export function setPublishTransport(transport) {
  testPublishTransport = transport;
}

export function setFavoriteTransport(transport) {
  testFavoriteTransport = transport;
}

export function setMineTransport(transport) {
  testMineTransport = transport;
}

export function setDownloadStatsTransport(transport) {
  testStatsTransport = transport;
}

export async function listSquareItems({ sort = "推荐", query = "", model = "", categoryId = null } = {}) {
  if (testTransport) return testTransport({ sort, query, model, ...(categoryId ? { categoryId } : {}) });
  if (isTauri()) {
    return tauriInvoke("list_square_items", { sort, query, model, category_id: categoryId, access_token: getSession().accessToken || null });
  }
  try {
    const params = new URLSearchParams({ sort, q: query });
    if (model) params.set("model", model);
    if (categoryId) params.set("category_id", categoryId);
    const token = getSession().accessToken;
    const response = await fetch(`${apiBase()}/v1/square/items?${params}`, { headers: token ? { Authorization: `Bearer ${token}` } : {} });
    if (!response.ok) throw new Error("广场暂时不可用");
    const payload = await response.json();
    return payload.items ?? [];
  } catch {
    throw new Error("广场暂时不可用");
  }
}

export async function fetchSquareContent(id) {
  if (testContentTransport) return testContentTransport(id);
  if (isTauri()) {
    return tauriInvoke("get_square_content", { id, access_token: getSession().accessToken || null });
  }
  try {
    const token = getSession().accessToken;
    const response = await fetch(`${apiBase()}/v1/square/items/${encodeURIComponent(id)}/content`, { headers: token ? { Authorization: `Bearer ${token}` } : {} });
    if (!response.ok) throw new Error("广场暂时不可用");
    return response.json();
  } catch {
    throw new Error("广场暂时不可用");
  }
}

const activeDownloads = new Map();
export async function completeSquareImages(id) {
  if (activeDownloads.has(id)) return activeDownloads.get(id);
  const task = (async () => {
    const existing = (await listLocalPrompts()).find(row => row.remote_id === id);
    if (!existing) throw Error('本地副本不存在，请重新下载');
    const payload = await fetchSquareContent(id);
    if (payload.kind === 'collection') throw Error('合集参考图不属于单条成员附件');
    const assets = await downloadReferenceImages(payload);
    if (!assets.length) throw Error('此条目没有可补全的参考图');
    return appendDownloadedAssets(existing.id, id, assets);
  })().finally(() => activeDownloads.delete(id));
  activeDownloads.set(id, task); return task;
}
export function downloadSquareItem(id) {
  if (activeDownloads.has(id)) return activeDownloads.get(id);
  const task = downloadNewSquareItem(id).finally(() => activeDownloads.delete(id));
  activeDownloads.set(id, task);
  return task;
}

async function downloadNewSquareItem(id) {
  const existing = (await listLocalPrompts()).find(row => row.remote_id === id);
  if (existing) return existing;
  const payload = await fetchSquareContent(id);
  const refs = validateReferences(payload.asset_refs ?? []);
  const localCategoryIds = new Set((await listLocalCategories()).map(category => category.id));
  const localCategory = id => localCategoryIds.has(id) ? id : null;
  let row;
  if (payload.kind === "collection") {
    validateCollectionAssets(payload.members, refs);
    const assets = new Map(), token = getSession().accessToken;
    for (const reference of refs) assets.set(reference.id, { ...await downloadPublishedAsset(id, reference, token), id: crypto.randomUUID() });
    const keepAuthor = (await getLocalSetting("keep_author_on_download")) === "1";
    row = await applyLocalImport(JSON.stringify({
      version: 2,
      collections: [{ id: "download", title: payload.title, category_id: localCategory(payload.category_id) }],
      prompts: payload.members.map((member) => ({
        title: member.title, content: member.content, category_id: localCategory(member.category_id), model: member.model,
        collection_id: "download", source: "downloaded", remote_id: payload.id ?? id,
        author: keepAuthor ? payload.author : null,
        assets: (member.asset_ids ?? []).map(id => assets.get(id)),
      })),
    }));
  } else if (refs.length || referenceImages(payload).length) {
    const assets = [];
    const token = getSession().accessToken;
    for (const reference of refs) assets.push({ ...await downloadPublishedAsset(id, reference, token), id: crypto.randomUUID() });
    assets.push(...await downloadReferenceImages(payload));
    validateAssets(assets);
    const keepAuthor = (await getLocalSetting('keep_author_on_download')) === '1';
    row = await applyLocalImport(JSON.stringify({ version: 2, prompts: [{
      title: payload.title, content: payload.content ?? '', category_id: localCategory(payload.category_id),
      model: payload.model, source: 'downloaded', remote_id: payload.id ?? id,
      author: keepAuthor ? payload.author : null, assets,
    }] }));
  } else {
    row = await importDownloadedPrompt({
      title: payload.title,
      content: payload.content ?? "",
      remoteId: payload.id ?? id,
      author: payload.author,
      categoryId: localCategory(payload.category_id),
      model: payload.model,
    });
  }
  void recordAnonymousDownload(id);
  return row;
}

async function recordAnonymousDownload(id) {
  try {
    if ((await getLocalSetting("anonymous_download_stats")) !== "1") return;
    if (testStatsTransport) {
      await testStatsTransport({
        id,
        method: "POST",
        path: `/v1/square/items/${id}/downloads`,
        headers: {},
      });
      return;
    }
    if (isTauri()) {
      await tauriInvoke("record_square_download", { id });
      return;
    }
    await fetch(`${apiBase()}/v1/square/items/${encodeURIComponent(id)}/downloads`, {
      method: "POST",
    });
  } catch {
    /* 统计失败不得阻断下载 */
  }
}

export async function createPublication({ sourceId, title, content, categoryId, model, kind, members, assetRefs } = {}) {
  const id = String(sourceId ?? "").trim();
  if (!id) throw new Error("未选择本地内容");
  if (assetRefs) validateReferences(assetRefs);
  if (kind === 'collection') validateCollectionAssets(members, assetRefs ?? []);
  if (testPublishTransport) return testPublishTransport({ sourceId: id, title, content, ...(categoryId ? { categoryId } : {}), ...(model ? { model } : {}), ...(kind ? { kind } : {}), ...(members ? { members } : {}), ...(assetRefs ? { assetRefs } : {}) });
  if (isTauri()) {
    return tauriInvoke("create_publication", {
      source_id: id,
      access_token: getSession().accessToken,
      title: title ?? null,
      content: content ?? null,
      category_id: categoryId ?? null,
      model: model ?? null,
      kind: kind ?? "prompt",
      members: members ?? [],
      asset_refs: assetRefs ?? [],
    });
  }
  const token = getSession().accessToken;
  if (!token) throw new Error("发布需要登录");
  try {
    const response = await fetch(`${apiBase()}/v1/publications`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ source_id: id, title, content, category_id: categoryId, model, kind, members, ...(assetRefs ? { asset_refs: assetRefs } : {}) }),
    });
    if (!response.ok) throw new Error("发布失败");
    return response.json();
  } catch {
    throw new Error("发布失败");
  }
}

async function favoriteRequest(method, id) {
  const token = getSession().accessToken;
  if (!token) throw new Error("收藏需要登录");
  if (testFavoriteTransport) {
    return testFavoriteTransport({ method, id });
  }
  if (isTauri()) {
    if (method === "GET") {
      return tauriInvoke("list_favorites", { access_token: token });
    }
    const command = method === "PUT" ? "put_favorite" : "delete_favorite";
    return tauriInvoke(command, { id, access_token: token });
  }
  const path = method === "GET" ? "/v1/favorites" : `/v1/favorites/${encodeURIComponent(id)}`;
  try {
    const response = await fetch(`${apiBase()}${path}`, {
      method,
      headers: { Authorization: `Bearer ${token}` },
    });
    if (method === "DELETE") {
      if (!response.ok) throw new Error("取消收藏失败");
      return { ok: true };
    }
    if (!response.ok) throw new Error("收藏失败");
    return response.json();
  } catch (error) {
    if (error instanceof Error && (error.message === "收藏失败" || error.message === "取消收藏失败")) {
      throw error;
    }
    throw new Error("收藏失败");
  }
}

export function putFavorite(id) {
  return favoriteRequest("PUT", id);
}

export function deleteFavorite(id) {
  return favoriteRequest("DELETE", id);
}

export async function listFavorites() {
  const payload = await favoriteRequest("GET");
  return payload.items ?? payload ?? [];
}

export async function listMyPublications() {
  if (testMineTransport) return testMineTransport();
  const token = getSession().accessToken;
  if (!token) throw new Error("查看发布需要登录");
  if (isTauri()) {
    const payload = await tauriInvoke("list_my_publications", { access_token: token });
    return payload.items ?? payload ?? [];
  }
  try {
    const response = await fetch(`${apiBase()}/v1/publications/mine`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!response.ok) throw new Error("发布列表暂时不可用");
    const payload = await response.json();
    return payload.items ?? [];
  } catch {
    throw new Error("发布列表暂时不可用");
  }
}
