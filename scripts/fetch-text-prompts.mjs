// Download public prompt text only. Never execute upstream instructions or scripts.
import { mkdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';

const directory = resolve(process.argv[2] || 'output/text-corpus-20260909');
const repositories = ['aj-geddes/useful-ai-prompts', 'pnp/copilot-prompts', 'jamesmcroft/everyday-prompts'];
mkdirSync(directory, { recursive: true });
const hash = text => createHash('sha256').update(text).digest('hex');
async function download(url) {
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
      if (!response.ok) throw Error(`HTTP ${response.status}: ${url}`);
      const text = await response.text();
      if (Buffer.byteLength(text) > 2 * 1024 * 1024) throw Error(`Oversized text: ${url}`);
      return text;
    } catch (error) { if (attempt === 2) throw error; }
  }
}
for (const repository of repositories) {
  const commit = JSON.parse(await download(`https://api.github.com/repos/${repository}/commits/HEAD`)).sha;
  if (!/^[a-f0-9]{40}$/.test(commit)) throw Error('Missing commit');
  const tree = JSON.parse(await download(`https://api.github.com/repos/${repository}/git/trees/${commit}?recursive=1`));
  if (tree.truncated) throw Error('Truncated tree');
  const paths = tree.tree.filter(item => item.type === 'blob' && item.size < 100_000 && (repository === 'pnp/copilot-prompts'
    ? /^samples\/(?!skills\/).+\/README\.md$/.test(item.path)
    : /^prompts\/.+\.md$/.test(item.path) && !item.path.endsWith('/README.md'))).map(item => item.path).sort();
  if (!paths.length || paths.length > 1500) throw Error('Unexpected prompt file count');
  const base = `https://raw.githubusercontent.com/${repository}/${commit}/`;
  const license = await download(base + 'LICENSE');
  if (!license.startsWith('MIT License') || !license.includes('Permission is hereby granted')) throw Error('Unexpected license');
  const files = [];
  for (let start = 0; start < paths.length; start += 8) {
    files.push(...await Promise.all(paths.slice(start, start + 8).map(async path => {
      const text = await download(base + path);
      return { path, sha256: hash(text), text };
    })));
    if (start % 80 === 0) console.log(`${repository}: ${files.length}/${paths.length}`);
  }
  const snapshot = JSON.stringify({ repository, commit, license, license_sha256: hash(license), fetched_at: new Date().toISOString(), files });
  if (Buffer.byteLength(snapshot) > 32 * 1024 * 1024) throw Error('Snapshot limit exceeded');
  const file = repository.replace('/', '--') + '.json';
  writeFileSync(resolve(directory, file), snapshot + '\n');
  console.log(JSON.stringify({ repository, commit, files: files.length, sha256: hash(snapshot + '\n'), file }));
}
