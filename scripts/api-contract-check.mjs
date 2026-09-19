import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const methods = 'get|post|put|delete|patch|head|options|trace';
const normalize = path => path.replace(/:[A-Za-z_][\w]*/g, '{}').replace(/\{[^}]+\}/g, '{}');

// This repository registers literal paths and inline MethodRouters in lib.rs.
// Unsupported registration forms must fail instead of silently losing coverage.
export function routerOperations(source) {
  source = source.replace(/"(?:\\.|[^"\\])*"|\/\/[^\n]*|\/\*[\s\S]*?\*\//g,
    token => token.startsWith('"') ? token : ' ');
  if (/\.(?:nest|nest_service|merge|route_service)\s*\(/.test(source)) {
    throw Error('Unsupported router composition; extend the API contract checker');
  }
  const operations = [];
  for (const match of source.matchAll(/\.route\s*\(/g)) {
    const start = match.index + match[0].length;
    let depth = 1, quoted = false, end = start;
    for (; end < source.length && depth; end++) {
      const char = source[end];
      if (quoted && char === '\\') { end++; continue; }
      if (char === '"') quoted = !quoted;
      if (!quoted && char === '(') depth++;
      if (!quoted && char === ')') depth--;
    }
    if (depth) throw Error('Unclosed .route registration');
    const args = source.slice(start, end - 1);
    const path = args.match(/^\s*"([^"\\]+)"\s*,/);
    if (!path) throw Error('Route path must be a string literal');
    const handler = args.slice(path[0].length);
    if (!new RegExp(`^\\s*(?:axum::routing::)?(${methods})\\s*\\(`).test(handler)
        || /\.(?:fallback|on)\s*\(/.test(handler)) {
      throw Error(`Unsupported MethodRouter for ${path[1]}`);
    }
    for (const method of handler.matchAll(new RegExp(`\\b(${methods})\\s*\\(`, 'g'))) {
      operations.push(`${method[1].toUpperCase()} ${normalize(path[1])}`);
    }
  }
  if (!operations.length) throw Error('No backend routes found');
  return unique(operations, 'backend');
}

function unique(operations, label) {
  const seen = new Set();
  for (const operation of operations) {
    if (seen.has(operation)) throw Error(`Duplicate ${label} operation: ${operation}`);
    seen.add(operation);
  }
  return [...seen].sort();
}

export function contractOperations(documents) {
  const operations = [];
  for (const document of documents) {
    let path;
    for (const line of document.split('\n')) {
      const route = line.match(/^  (\/[^\s]+):\s*$/);
      if (route) path = route[1];
      const method = line.match(new RegExp(`^    (${methods}):\\s*$`));
      if (method && path) operations.push(`${method[1].toUpperCase()} ${normalize(path)}`);
    }
  }
  if (!operations.length) throw Error('No OpenAPI operations found');
  return unique(operations, 'OpenAPI');
}

export function checkContracts(source, documents) {
  const actual = routerOperations(source);
  const documented = contractOperations(documents);
  const missing = actual.filter(operation => !documented.includes(operation));
  const obsolete = documented.filter(operation => !actual.includes(operation));
  if (missing.length || obsolete.length) {
    throw Error([
      ...missing.map(operation => `Missing OpenAPI: ${operation}`),
      ...obsolete.map(operation => `Missing backend: ${operation}`),
    ].join('\n'));
  }
  return actual.length;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const read = path => readFileSync(resolve(root, path), 'utf8');
  try {
    const count = checkContracts(read('backend/src/lib.rs'),
      ['square', 'admin'].map(name => read(`docs/reference/openapi/${name}.yaml`)));
    console.log(`API contract check passed (${count} explicit HTTP operations).`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
