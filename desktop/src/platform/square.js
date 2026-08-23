import { importDownloadedPrompt } from "./library.js";

let testTransport = null;
let testContentTransport = null;

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
}

export function setSquareTransport(transport) {
  testTransport = transport;
}

export function setSquareContentTransport(transport) {
  testContentTransport = transport;
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
  });
}
