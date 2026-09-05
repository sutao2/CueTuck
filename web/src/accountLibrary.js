import { getSession } from "./session.js";
import { activateMemoryLibrary, listLocalCollections, listLocalPrompts, replacePromptsFromAccount } from "./memoryLibrary.js";

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:8787";

let testTransport = null;

export function resetAccountLibrary() {
  testTransport = null;
}

export function setAccountLibraryTransport(transport) {
  testTransport = transport;
}

export async function loadAccountLibrary() {
  const token = getSession().accessToken;
  activateMemoryLibrary(getSession().email);
  if (!token) return listLocalPrompts();
  try {
    let payload;
    if (testTransport?.get) {
      payload = await testTransport.get();
    } else {
      const response = await fetch(`${API_BASE}/v1/library/changes?since=`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!response.ok) throw new Error("同步失败");
      payload = await response.json();
    }
    if (getSession().accessToken !== token) throw new Error("账号已改变，已忽略旧请求");
    return replacePromptsFromAccount(payload.items ?? []);
  } catch {
    throw new Error("账号库读取失败，当前标签页内容已保留");
  }
}

export async function pushAccountPrompt(row) {
  const token = getSession().accessToken;
  if (!token || !row?.id) return;
  const item = {
    id: row.id,
    kind: "prompt",
    payload: { ...row, source: row.original_source ?? row.source ?? "local" },
    updated_at: String(row.updated_at ?? Date.now()),
  };
  delete item.payload.original_source;
  delete item.payload.sync_pending;
  const collection = listLocalCollections().find((value) => value.id === row.collection_id);
  const items = collection ? [{ id: collection.id, kind: "collection", payload: collection, updated_at: collection.updated_at }, item] : [item];
  if (testTransport?.put) {
    return testTransport.put(items);
  }
  const response = await fetch(`${API_BASE}/v1/library/changes`, {
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
