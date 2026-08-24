export function extractVariables(content) {
  const names = [];
  const pattern = /\{\{\s*([^}]*?)\s*\}\}/g;
  let match;
  while ((match = pattern.exec(content ?? ""))) {
    const name = match[1].trim();
    if (!name || names.includes(name)) continue;
    names.push(name);
  }
  return names;
}

export function renderPrompt(content, values = {}) {
  return (content ?? "").replace(/\{\{\s*([^}]*?)\s*\}\}/g, (original, raw) => {
    const name = raw.trim();
    if (!name) return original;
    const value = values[name];
    if (value == null || value === "") return `{{${name}}}`;
    return String(value);
  });
}
