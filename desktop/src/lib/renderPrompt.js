function parseVariables(content) {
  const tokens = [];
  const pattern = /\{\{\s*([^}]*?)\s*\}\}/g;
  let match;
  while ((match = pattern.exec(content))) {
    const name = match[1].trim();
    if (name) tokens.push({ start: match.index, end: pattern.lastIndex, name, fallback: `{{${name}}}` });
  }

  // Anonymous slots are prose only: skip code, quoted strings and escaped braces.
  const syntax = /```[\s\S]*?(?:```|$)|~~~[\s\S]*?(?:~~~|$)|`[^`\n]*`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\\[{}]|[{}]/g;
  const stack = [];
  while ((match = syntax.exec(content))) {
    if (match[0] === "{") stack.push(match.index);
    else if (match[0] === "}") {
      const start = stack.pop();
      if (start == null || stack.length) continue;
      const original = content.slice(start, syntax.lastIndex);
      if (/^\{[ \t]*\}$/.test(original)) tokens.push({ start, end: syntax.lastIndex, fallback: original });
    }
  }

  tokens.sort((a, b) => a.start - b.start);
  const reserved = new Set(tokens.map((token) => token.name).filter(Boolean));
  let number = 1;
  for (const token of tokens) {
    if (token.name) continue;
    while (reserved.has(`占位符 ${number}`)) number++;
    token.name = `占位符 ${number++}`;
  }
  return tokens;
}

export function extractVariables(content) {
  return [...new Set(parseVariables(content ?? "").map((token) => token.name))];
}

export function renderPrompt(content, values = {}) {
  const source = content ?? "";
  let result = "";
  let cursor = 0;
  for (const token of parseVariables(source)) {
    const value = values[token.name];
    const replacement = !Object.hasOwn(values, token.name) || value == null || value === ""
      ? token.fallback : String(value);
    result += source.slice(cursor, token.start) + replacement;
    cursor = token.end;
  }
  return result + source.slice(cursor);
}
