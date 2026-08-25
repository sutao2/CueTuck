let prompts = [];

export function resetMemoryLibrary() {
  prompts = [];
}

export function createLocalPrompt({ title, content, source = "local" } = {}) {
  const row = {
    id: `mem-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    title: String(title ?? "").trim(),
    content: content ?? "",
    source,
    updated_at: String(Date.now()),
  };
  prompts = [row, ...prompts];
  return row;
}

export function importDownloadedPrompt({ title, content, remoteId = null } = {}) {
  const row = createLocalPrompt({
    title,
    content,
    source: "downloaded",
  });
  row.remote_id = remoteId;
  return row;
}

export function listLocalPrompts() {
  return [...prompts];
}

export function replacePromptsFromAccount(items) {
  prompts = (Array.isArray(items) ? items : [])
    .filter((item) => item?.kind === "prompt" && !item.deleted_at)
    .map((item) => ({
      id: item.id,
      title: String(item.payload?.title ?? "").trim(),
      content: item.payload?.content ?? "",
      source: "account",
      updated_at: String(item.updated_at ?? "0"),
    }));
  return listLocalPrompts();
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
  row.updated_at = String(Date.now());
  return row;
}
