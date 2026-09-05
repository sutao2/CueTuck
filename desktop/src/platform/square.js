import { importDownloadedPrompt, getLocalSetting } from "./library.js";
import { getSession } from "./session.js";

let testTransport = null;
let testContentTransport = null;
let testPublishTransport = null;
let testFavoriteTransport = null;
let testMineTransport = null;
let testStatsTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

function apiBase() {
  return "http://127.0.0.1:8787";
}

export function resetSquare() {
  testTransport = null;
  testContentTransport = null;
  testPublishTransport = null;
  testFavoriteTransport = null;
  testMineTransport = null;
  testStatsTransport = null;
}

export function setSquareTransport(transport) {
  testTransport = transport;
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
    return tauriInvoke("list_square_items", { sort, query, model, category_id: categoryId });
  }
  try {
    const params = new URLSearchParams({ sort, q: query });
    if (model) params.set("model", model);
    if (categoryId) params.set("category_id", categoryId);
    const response = await fetch(`${apiBase()}/v1/square/items?${params}`);
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
    return tauriInvoke("get_square_content", { id });
  }
  try {
    const response = await fetch(`${apiBase()}/v1/square/items/${encodeURIComponent(id)}/content`);
    if (!response.ok) throw new Error("广场暂时不可用");
    return response.json();
  } catch {
    throw new Error("广场暂时不可用");
  }
}

export async function downloadSquareItem(id) {
  let row;
  if (isTauri() && !testContentTransport) {
    row = await tauriInvoke("download_square_item", { id });
  } else {
    const payload = await fetchSquareContent(id);
    row = await importDownloadedPrompt({
      title: payload.title,
      content: payload.content ?? "",
      remoteId: payload.id ?? id,
      author: payload.author,
      categoryId: payload.category_id,
      model: payload.model,
    });
  }
  await recordAnonymousDownload(id);
  return row;
}

async function recordAnonymousDownload(id) {
  if ((await getLocalSetting("anonymous_download_stats")) !== "1") return;
  try {
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

export async function createPublication({ sourceId, title, content, categoryId, model } = {}) {
  const id = String(sourceId ?? "").trim();
  if (!id) throw new Error("未选择本地内容");
  if (testPublishTransport) return testPublishTransport({ sourceId: id, title, content, ...(categoryId ? { categoryId } : {}), ...(model ? { model } : {}) });
  if (isTauri()) {
    return tauriInvoke("create_publication", {
      source_id: id,
      access_token: getSession().accessToken,
      title: title ?? null,
      content: content ?? null,
      category_id: categoryId ?? null,
      model: model ?? null,
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
      body: JSON.stringify({ source_id: id, title, content, category_id: categoryId, model }),
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
