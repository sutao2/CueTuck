let testTransport = null;

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
}

export function setSquareTransport(transport) {
  testTransport = transport;
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
