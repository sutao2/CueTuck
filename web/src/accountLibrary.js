import { getSession } from "./session.js";
import { listLocalPrompts, replacePromptsFromAccount } from "./memoryLibrary.js";

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
    return replacePromptsFromAccount(payload.items ?? []);
  } catch {
    return listLocalPrompts();
  }
}

export async function pushAccountPrompt(row) {
  const token = getSession().accessToken;
  if (!token || !row?.id) return;
  const item = {
    id: row.id,
    kind: "prompt",
    payload: { title: row.title, content: row.content },
    updated_at: String(row.updated_at ?? Date.now()),
  };
  if (testTransport?.put) {
    return testTransport.put([item]);
  }
  const response = await fetch(`${API_BASE}/v1/library/changes`, {
    method: "PUT",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ items: [item] }),
  });
  if (!response.ok) throw new Error("同步失败");
  return response.json();
}
