// Resumable public DiffusionDB metadata download. Fixed source and SHA-256;
// never imports the dataset's Python loader or downloads the full image corpus.
import { mkdirSync, existsSync, statSync, writeFileSync, openSync, readSync, closeSync, renameSync, createReadStream, createWriteStream } from 'node:fs';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { once } from 'node:events';

const directory = resolve(process.argv[2] || 'output/prompt-corpus-20260908');
const size = 194548652, chunkSize = 4 * 1024 * 1024;
const expected = 'eecd341187bc91c07f5994ad0660d40228ea025616fd57a509bef8323677c68f';
const target = resolve(directory, 'diffusiondb.parquet');
const cache = resolve(directory, 'metadata-parts');
mkdirSync(cache, { recursive: true });
// Reuse a stopped curl prefix. The assembled result must still match the hash.
const prefix = existsSync(target) ? openSync(target, 'r') : null;
const prefixLength = prefix === null ? 0 : statSync(target).size;
const jobs = [];
for (let start = 0; start < size; start += chunkSize) {
  const end = Math.min(size, start + chunkSize) - 1;
  const path = resolve(cache, `${start}.part`), length = end - start + 1;
  if (!existsSync(path) && prefix !== null && start + length <= prefixLength) {
    const bytes = Buffer.alloc(length);
    if (readSync(prefix, bytes, 0, length, start) === length) writeFileSync(path, bytes);
  }
  jobs.push({ start, end, path, length });
}
if (prefix !== null) closeSync(prefix);
let cursor = 0;
async function worker() {
  while (cursor < jobs.length) {
    const job = jobs[cursor++];
    if (existsSync(job.path) && statSync(job.path).size === job.length) continue;
    const url = `https://huggingface.co/datasets/poloclub/diffusiondb/resolve/main/metadata.parquet?download=true&range=${job.start}`;
    const child = spawn('curl', ['-fsSL', '--retry', '2', '--max-time', '120', '-r', `${job.start}-${job.end}`, url, '-o', job.path], { stdio: ['ignore', 'ignore', 'inherit'] });
    const [code] = await once(child, 'close');
    if (code || !existsSync(job.path) || statSync(job.path).size !== job.length) throw Error(`Incomplete range ${job.start}; rerun to resume`);
    console.log(`Downloaded range ${job.start}-${job.end}`);
  }
}
const workers = await Promise.allSettled([worker(), worker(), worker()]);
if (workers.some(result => result.status === 'rejected')) throw Error('Some ranges failed; cached complete ranges retained for retry');
const hash = createHash('sha256'), output = createWriteStream(`${target}.verified-part`);
for (const job of jobs) {
  for await (const bytes of createReadStream(job.path)) {
    hash.update(bytes);
    if (!output.write(bytes)) await once(output, 'drain');
  }
}
output.end(); await once(output, 'finish');
if (hash.digest('hex') !== expected) throw Error('Metadata SHA-256 mismatch; original target unchanged');
renameSync(`${target}.verified-part`, target);
console.log(`Verified ${size} bytes, SHA-256 ${expected}`);
