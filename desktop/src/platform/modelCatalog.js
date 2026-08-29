export function parseModelNames(...sources) {
  const names = [];
  const seen = new Set();

  function add(name) {
    const trimmed = String(name ?? "").trim();
    if (!trimmed || seen.has(trimmed)) return;
    seen.add(trimmed);
    names.push(trimmed);
  }

  for (const source of sources) {
    if (source == null) continue;
    if (Array.isArray(source)) {
      for (const part of source) {
        if (part && typeof part === "object") add(part.model);
        else add(part);
      }
      continue;
    }
    String(source)
      .split(/[\n,;]+/)
      .forEach(add);
  }
  return names;
}
