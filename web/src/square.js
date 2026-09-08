import { apiBase } from "../../shared/apiBase.js";
import { importDownloadedCollection, importDownloadedPrompt } from "./memoryLibrary.js";

const API_BASE = apiBase();

let testList = null;
let testContent = null;
let testFavorite = null;
let testCatalog = null;

export function resetSquare() {
  testList = null;
  testContent = null;
  testFavorite = null;
  testCatalog = null;
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

export function setCatalogTransport(transport) { testCatalog = transport; }
export async function fetchSquareCatalog() {
  if (testCatalog) return testCatalog();
  const response = await fetch(`${API_BASE}/v1/square/catalog`);
  if (!response.ok) throw new Error('广场分类配置暂时不可用');
  const payload = await response.json();
  if (!Array.isArray(payload.categories) || !Array.isArray(payload.models)) throw new Error('广场字典响应无效');
  return payload;
}
export async function listSquareItems({ categoryId = '', model = '' } = {}) {
  if (testList) return testList({ categoryId, model });
  const response = await fetch(`${API_BASE}/v1/square/items?${new URLSearchParams({category_id:categoryId,model})}`);
  if (!response.ok) throw new Error("广场暂时不可用");
  const payload = await response.json();
  return payload.items ?? [];
}

export async function downloadSquareItem(id) {
  const payload = testContent
    ? await testContent(id)
    : await fetchSquareContent(id);
  if (payload.kind === "collection") return importDownloadedCollection(payload);
  return importDownloadedPrompt({
    title: payload.title,
    content: payload.content ?? "",
    remoteId: payload.id ?? id,
    categoryId: payload.category_id,
    model: payload.model,
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
