function parseVariables(content) {
  const tokens = [];
  const pattern = /\{argument\b(?:[^{}"'\\]|\\.|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')*\}|\{\{\s*([^}]*?)\s*\}\}/g;
  const argumentsRanges = [];
  let match;
  while ((match = pattern.exec(content))) {
    if (match[0].startsWith('{argument')) {
      argumentsRanges.push([match.index, pattern.lastIndex]);
      if ((content.slice(0,match.index).match(/\\+$/)?.[0].length ?? 0) % 2) continue;
      const attributes = match[0].slice(9,-1), fields = new Map();
      const attribute = /\s+(name|default)\s*=\s*("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')/gy;
      let part, end=0, valid=true;
      while ((part=attribute.exec(attributes))) {
        if (fields.has(part[1])) {valid=false;break;}
        fields.set(part[1],part[2].slice(1,-1).replace(/\\([\\"'])/g,'$1'));end=attribute.lastIndex;
      }
      const name=fields.get('name')?.trim();
      if (valid && name && !attributes.slice(end).trim()) tokens.push({start:match.index,end:pattern.lastIndex,name,fallback:match[0],defaultValue:fields.get('default')});
      continue;
    }
    const name = match[1].trim();
    if (name) tokens.push({ start: match.index, end: pattern.lastIndex, name, fallback: `{{${name}}}` });
  }

  // Anonymous slots are prose only: skip code, quoted strings and escaped braces.
  const syntax = /```[\s\S]*?(?:```|$)|~~~[\s\S]*?(?:~~~|$)|`[^`\n]*`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\\[{}]|[{}]/g;
  const stack = [];
  while ((match = syntax.exec(content))) {
    const argument=argumentsRanges.find(([start,end])=>match.index>=start && match.index<end);
    if (argument) {syntax.lastIndex=argument[1];continue;}
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

export function variableDefaults(content) {
  const defaults = new Map();
  for (const token of parseVariables(content ?? '')) {
    if (token.defaultValue !== undefined && !defaults.has(token.name)) defaults.set(token.name,token.defaultValue);
  }
  return Object.fromEntries(defaults);
}

export function renderPrompt(content, values = {}) {
  const source = content ?? "";
  let result = "";
  let cursor = 0;
  const defaults = variableDefaults(source);
  for (const token of parseVariables(source)) {
    const value = values[token.name];
    const replacement = !Object.hasOwn(values, token.name) || value == null || value === ""
      ? (Object.hasOwn(defaults,token.name) ? defaults[token.name] : token.fallback) : String(value);
    result += source.slice(cursor, token.start) + replacement;
    cursor = token.end;
  }
  return result + source.slice(cursor);
}
