import { importDownloadedPrompt } from "./library.js";
import { getSession } from "./session.js";

let testTransport = null;
let testContentTransport = null;
let testPublishTransport = null;
let testFavoriteTransport = null;

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

export function resetSquare() {
  testTransport = null;
  testContentTransport = null;
  testPublishTransport = null;
  testFavoriteTransport = null;
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

export async function listSquareItems({ sort = "推荐", query = "" } = {}) {
  if (testTransport) return testTransport({ sort, query });
  if (isTauri()) {
    return tauriInvoke("list_square_items", { sort, query });
  }
  try {
    const params = new URLSearchParams({ sort, q: query });
    const response = await fetch(`${apiBase()}/v1/square/items?${params}`);
    if (!response.ok) throw new Error("广场暂时不可用");
    const payload = await response.json();
    return payload.items ?? [];
  } catch {
    throw new Error("广场暂时不可用");
  }
}

async function fetchSquareContent(id) {
  if (testContentTransport) return testContentTransport(id);
  if (isTauri()) {
    return tauriInvoke("download_square_item", { id });
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
  if (isTauri() && !testContentTransport) {
    return tauriInvoke("download_square_item", { id });
  }
  const payload = await fetchSquareContent(id);
  return importDownloadedPrompt({
    title: payload.title,
    content: payload.content ?? "",
    remoteId: payload.id ?? id,
    author: payload.author,
  });
}

export async function createPublication({ sourceId, title, content } = {}) {
  const id = String(sourceId ?? "").trim();
  if (!id) throw new Error("未选择本地内容");
  if (testPublishTransport) return testPublishTransport({ sourceId: id, title, content });
  if (isTauri()) {
    return tauriInvoke("create_publication", {
      source_id: id,
      access_token: getSession().accessToken,
      title: title ?? null,
      content: content ?? null,
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
      body: JSON.stringify({ source_id: id, title, content }),
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
