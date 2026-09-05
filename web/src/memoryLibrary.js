let prompts = [];
let collections = [];
let activeAccount = null;
const libraries = new Map();

export function resetMemoryLibrary() {
  prompts = [];
  collections = [];
  activeAccount = null;
  libraries.clear();
}

export function activateMemoryLibrary(email) {
  const next = email || null;
  if (next === activeAccount) return;
  libraries.set(activeAccount, { prompts, collections });
  const saved = libraries.get(next);
  prompts = saved?.prompts ?? [];
  collections = saved?.collections ?? [];
  activeAccount = next;
}

export function createLocalPrompt({ title, content, source = "local", categoryId = null, model = null } = {}) {
  const row = {
    id: `mem-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    title: String(title ?? "").trim(),
    content: content ?? "",
    source,
    category_id: categoryId,
    model,
    updated_at: String(Date.now()),
  };
  prompts = [row, ...prompts];
  return row;
}

export function importDownloadedPrompt({ title, content, remoteId = null, categoryId = null, model = null } = {}) {
  const row = createLocalPrompt({
    title,
    content,
    source: "downloaded",
    categoryId,
    model,
  });
  row.remote_id = remoteId;
  return row;
}

export function listLocalPrompts() {
  return [...prompts];
}

export function listLocalCollections() {
  return collections.map((row) => ({ ...row, member_count: prompts.filter((prompt) => prompt.collection_id === row.id).length }));
}

export function importDownloadedCollection(payload) {
  if (typeof payload.title !== "string" || !payload.title.trim() || !Array.isArray(payload.members) || !payload.members.length) throw new Error("该合集缺少成员快照，暂时无法下载");
  const collectionId = crypto.randomUUID();
  const timestamp = String(Date.now());
  const members = payload.members.map((member) => {
    if (typeof member?.title !== "string" || !member.title.trim() || typeof member.content !== "string" || !member.content.trim()) throw new Error("合集成员快照不完整");
    return { id: crypto.randomUUID(), title: member.title, content: member.content,
      category_id: member.category_id ?? null, model: member.model ?? null,
      collection_id: collectionId, source: "downloaded", remote_id: payload.id, updated_at: timestamp };
  });
  const collection = { id: collectionId, title: payload.title, category_id: payload.category_id ?? null, cover_type: "none", cover_json: "[]", updated_at: timestamp };
  collections = [collection, ...collections];
  prompts = [...members, ...prompts];
  return collection;
}

export function replacePromptsFromAccount(items) {
  const pending = prompts.filter((row) => row.sync_pending);
  const pendingCollections = collections.filter((row) => pending.some((prompt) => prompt.collection_id === row.id));
  collections = (Array.isArray(items) ? items : [])
    .filter((item) => item?.kind === "collection" && !item.deleted_at)
    .map((item) => ({ ...item.payload, id: item.id, updated_at: String(item.updated_at ?? "0") }));
  prompts = (Array.isArray(items) ? items : [])
    .filter((item) => item?.kind === "prompt" && !item.deleted_at)
    .map((item) => ({
      ...item.payload,
      id: item.id,
      title: String(item.payload?.title ?? "").trim(),
      content: item.payload?.content ?? "",
      source: "account",
      original_source: item.payload?.source ?? "local",
      category_id: item.payload?.category_id ?? null,
      model: item.payload?.model ?? null,
      updated_at: String(item.updated_at ?? "0"),
    }));
  prompts = [...pending, ...prompts.filter((row) => !pending.some((draft) => draft.id === row.id))];
  collections = [...pendingCollections, ...collections.filter((row) => !pendingCollections.some((draft) => draft.id === row.id))];
  return listLocalPrompts();
}

export function markPromptSynced(id, updatedAt) {
  const row = prompts.find((row) => row.id === id && row.updated_at === updatedAt);
  if (row) delete row.sync_pending;
}

export function getLocalPrompt(id) {
  return prompts.find((row) => row.id === id) ?? null;
}

export function updateLocalPrompt({ id, title, content } = {}) {
  const row = prompts.find((item) => item.id === id);
  if (!row) throw new Error("提示词不存在");
  const nextTitle = String(title ?? "").trim();
  if (!nextTitle) return row;
  row.title = nextTitle;
  row.content = content ?? "";
  const stamp = Number(row.updated_at || 0);
  const previous = stamp >= 1e9 && stamp < 1e11 ? stamp * 1000 : stamp;
  row.updated_at = String(Math.max(Date.now(), previous + 1));
  return row;
}
