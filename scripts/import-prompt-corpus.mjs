// Explicit local-only, append-only import of the curated JSONL, never raw data.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { gunzipSync } from 'node:zlib';

const licenses = {
  'poloclub/diffusiondb': 'CC0 1.0',
  'f/prompts.chat': 'CC0 1.0',
  'YouMind-OpenLab/awesome-nano-banana-pro-prompts': 'CC BY 4.0',
};
export function validateCorpus(records) {
  if (!Array.isArray(records) || records.length < 1 || records.length > 50000) throw Error('Expected 1–50000 curated records');
  const ids = new Set(), bodies = new Set();
  for (const r of records) {
    const key = typeof r.content === 'string' ? r.content.toLowerCase().replace(/\s+/g, ' ').trim() : '';
    if (!/^corpus-[a-f0-9]{24}$/.test(r.id) || ids.has(r.id) || bodies.has(key) || !key || r.content.length > 16000
      || typeof r.title !== 'string' || !r.title.trim() || r.title.length > 160 || r.kind !== 'prompt'
      || !/^cat-[a-z]+(?:-\d+)?$/.test(r.category_id) || !(r.model === null || typeof r.model === 'string')
      || !r.reference || !licenses[r.reference.repository] || r.reference.license !== licenses[r.reference.repository]
      || !r.reference.author || !Array.isArray(r.reference.images)) throw Error(`Invalid or duplicate record: ${r.id}`);
    for (const url of [r.reference.url, r.reference.license_url, ...r.reference.images]) {
      const u = new URL(url);
      if (u.protocol !== 'https:' || u.username || u.password) throw Error('Only safe HTTPS references');
    }
    if (r.reference.images.length > 4 || r.reference.images.some(url => new URL(url).hostname !== 'cms-assets.youmind.com')) throw Error('Unreviewed image host');
    if (r.content.includes('\n来源：') && r.content.includes('\n许可：')) throw Error('Attribution must not be appended to the prompt');
    ids.add(r.id); bodies.add(key);
  }
  return records.length;
}

export function corpusSql(records) {
  validateCorpus(records);
  // stdin rather than argv avoids shell interpretation and argument size limits.
  const batches = [];
  for (let start = 0; start < records.length; start += 500) {
    const json = JSON.stringify(records.slice(start, start + 500)).replaceAll("'", "''");
    batches.push(`INSERT INTO incoming SELECT * FROM jsonb_to_recordset('${json}'::jsonb) AS x(id text,title text,kind text,content text,excerpt text,category_id text,model text,reference jsonb);`);
  }
  return `BEGIN;
SET LOCAL standard_conforming_strings=on;
SELECT pg_advisory_xact_lock(80480908);
CREATE TEMP TABLE incoming(id text PRIMARY KEY,title text,kind text,content text,excerpt text,category_id text,model text,reference jsonb) ON COMMIT DROP;
${batches.join('\n')}
DO $$ BEGIN
IF EXISTS(SELECT 1 FROM incoming i WHERE NOT EXISTS(SELECT 1 FROM public.catalog c WHERE c.kind='categories' AND c.id=i.category_id AND (c.data->>'enabled')::boolean AND NOT c.deleted)
 OR (i.model IS NOT NULL AND NOT EXISTS(SELECT 1 FROM public.catalog c WHERE c.kind='models' AND c.id=i.model AND (c.data->>'enabled')::boolean AND NOT c.deleted))) THEN RAISE EXCEPTION 'Missing enabled catalog reference'; END IF;
IF EXISTS(SELECT 1 FROM incoming i JOIN public.square_items s USING(id) WHERE s.content<>i.content) THEN RAISE EXCEPTION 'Existing ID has different body; refusing overwrite'; END IF;
END $$;
WITH added AS (INSERT INTO public.square_items(id,title,kind,content,excerpt,category_id,model,reference,sort_index)
 SELECT i.id,i.title,i.kind,i.content,i.excerpt,i.category_id,i.model,i.reference,
 CASE WHEN jsonb_array_length(i.reference->'images')>0 THEN 10 WHEN i.model IS DISTINCT FROM 'Stable Diffusion' THEN 20 ELSE 30 END
 FROM incoming i WHERE NOT EXISTS(SELECT 1 FROM public.square_items s WHERE s.content=i.content)
 ON CONFLICT(id) DO NOTHING RETURNING id) SELECT count(*) AS inserted FROM added;
COMMIT;`;
}

function main() {
  const [file, mode] = process.argv.slice(2);
  if (!file || (mode && mode !== '--apply-local')) throw Error('Usage: node scripts/import-prompt-corpus.mjs CURATED.jsonl [--apply-local]');
  const bytes = readFileSync(file);
  const input = (file.endsWith('.gz') ? gunzipSync(bytes, { maxOutputLength: 256 * 1024 * 1024 }) : bytes).toString('utf8');
  const records = input.trim().split('\n').map(line => JSON.parse(line));
  validateCorpus(records);
  const sha256 = createHash('sha256').update(input).digest('hex');
  console.log(JSON.stringify({ records: records.length, sha256, mode: mode || 'dry-run' }));
  if (!mode) return;
  const run = (args, data) => {
    const result = spawnSync('docker', args, { input: data, maxBuffer: 256 * 1024 * 1024 });
    if (result.error || result.status !== 0) throw Error(result.error?.message || result.stderr.toString());
    return result.stdout;
  };
  const context = JSON.parse(run(['context', 'inspect']))[0];
  if (!context?.Endpoints?.docker?.Host?.startsWith('unix://')) throw Error('Only a local Unix Docker context is allowed');
  const root = fileURLToPath(new URL('../', import.meta.url));
  const backup = resolve(root, `output/corpus-square-before-${Date.now()}.sql`);
  mkdirSync(resolve(root, 'output'), { recursive: true });
  writeFileSync(backup, run(['exec', 'backend-db-1', 'pg_dump', '-U', 'pl', '-d', 'promptark', '--table=public.square_items']), { mode: 0o600 });
  console.log(`Backup: ${backup}`);
  console.log(run(['exec', '-i', 'backend-db-1', 'psql', '-X', '-v', 'ON_ERROR_STOP=1', '-U', 'pl', '-d', 'promptark'], corpusSql(records)).toString());
}
if (process.argv[1] === fileURLToPath(import.meta.url)) main();
