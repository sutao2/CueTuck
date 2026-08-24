let prompts = [];

export function resetMemoryLibrary() {
  prompts = [];
}

export function createLocalPrompt({ title, content } = {}) {
  const row = {
    id: `mem-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    title: String(title ?? "").trim(),
    content: content ?? "",
  };
  prompts = [row, ...prompts];
  return row;
}

export function listLocalPrompts() {
  return [...prompts];
}
