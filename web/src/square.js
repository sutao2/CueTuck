import { importDownloadedPrompt } from "./memoryLibrary.js";

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:8787";

let testList = null;
let testContent = null;
let testFavorite = null;

export function resetSquare() {
  testList = null;
  testContent = null;
  testFavorite = null;
}

export function setSquareTransport(transport) {
  testList = transport;
}

export function setSquareContentTransport(transport) {
  testContent = transport;
}

export function setFavoriteTransport(transport) {
  testFavorite = transport;
}

export async function listSquareItems() {
  if (testList) return testList();
  const response = await fetch(`${API_BASE}/v1/square/items`);
  if (!response.ok) throw new Error("广场暂时不可用");
  const payload = await response.json();
  return payload.items ?? [];
}

export async function downloadSquareItem(id) {
  const payload = testContent
    ? await testContent(id)
    : await fetchSquareContent(id);
  return importDownloadedPrompt({
    title: payload.title,
    content: payload.content ?? "",
    remoteId: payload.id ?? id,
  });
}

async function fetchSquareContent(id) {
  const response = await fetch(`${API_BASE}/v1/square/items/${encodeURIComponent(id)}/content`);
  if (!response.ok) throw new Error("广场暂时不可用");
  return response.json();
}

export async function putFavorite(id, accessToken) {
  if (testFavorite) return testFavorite({ method: "PUT", id });
  const response = await fetch(`${API_BASE}/v1/favorites/${encodeURIComponent(id)}`, {
    method: "PUT",
    headers: { authorization: `Bearer ${accessToken}` },
  });
  if (!response.ok) throw new Error("收藏失败");
  try {
    return await response.json();
  } catch {
    return { id };
  }
}
