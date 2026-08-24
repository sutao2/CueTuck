import { importDownloadedPrompt } from "./memoryLibrary.js";

const API_BASE = "http://127.0.0.1:8787";

let testList = null;
let testContent = null;

export function resetSquare() {
  testList = null;
  testContent = null;
}

export function setSquareTransport(transport) {
  testList = transport;
}

export function setSquareContentTransport(transport) {
  testContent = transport;
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
