// Read-only export from the documented local database; importing is an explicit separate step.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

export const tables = ['catalog', 'catalog_redirects', 'square_items'];
const key = table => table === 'catalog' ? ['kind', 'id'] : table === 'catalog_redirects' ? ['kind', 'source'] : ['id'];
const literal = value => `'${JSON.stringify(value).replaceAll("'", "''")}'::jsonb`;
export function syncSql(data) {
  if (tables.some(t => !Array.isArray(data[t]))) throw Error('Missing allowlisted table');
  let sql = "BEGIN;\nSET LOCAL standard_conforming_strings=on;\nSET LOCAL lock_timeout='10s';\nSET LOCAL statement_timeout='60s';\nSELECT pg_advisory_xact_lock(20260913);\n";
  for (const table of tables) {
    const rows = data[table];
    if (rows.length > 50000) throw Error('Too many rows');
    const keys = key(table);
    if (rows.some(r => keys.some(k => typeof r[k] !== 'string' || !r[k]))) throw Error(`Invalid ${table} identity`);
    const ids = rows.map(r => JSON.stringify(keys.map(k => r[k])));
    if (new Set(ids).size !== ids.length) throw Error('Duplicate identities');
    sql += `CREATE TEMP TABLE incoming_${table} (LIKE public.${table}) ON COMMIT DROP;\n`;
    for (let start = 0; start < rows.length; start += 200) {
      sql += `INSERT INTO incoming_${table} SELECT * FROM jsonb_populate_recordset(NULL::public.${table}, ${literal(rows.slice(start, start + 200))});\n`;
    }
    sql += `LOCK TABLE public.${table} IN SHARE ROW EXCLUSIVE MODE;\n`;
    if (table === 'catalog') sql += "UPDATE incoming_catalog i SET data=jsonb_set(i.data,'{region}',t.data->'region') FROM public.catalog t WHERE i.kind=t.kind AND i.id=t.id AND NOT (i.data ? 'region') AND t.data ? 'region';\n";
    sql += `DO $$ BEGIN IF EXISTS(SELECT 1 FROM incoming_${table} i JOIN public.${table} t USING (${keys.join(',')}) WHERE to_jsonb(i) IS DISTINCT FROM to_jsonb(t)) THEN RAISE EXCEPTION 'Conflicting existing ${table} rows; import refused'; END IF; END $$;\n`;
    sql += `INSERT INTO public.${table} SELECT * FROM incoming_${table} ON CONFLICT (${keys.join(',')}) DO NOTHING;\n`;
  }
  sql += "DO $$ BEGIN IF EXISTS(SELECT 1 FROM incoming_square_items i WHERE i.category_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM public.catalog c WHERE c.kind='categories' AND c.id=i.category_id AND NOT c.deleted)) THEN RAISE EXCEPTION 'Missing category'; END IF; END $$;\nCOMMIT;\n";
  return sql;
}
function main() {
  const directory = process.argv[2];
  if (!directory) throw Error('Usage: node scripts/export-square-sync.mjs PRIVATE_OUTPUT_DIRECTORY');
  const query = "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;\n" + tables.map(table => `SELECT json_build_object('table','${table}','rows',COALESCE(jsonb_agg(to_jsonb(t)),'[]'::jsonb)) FROM public.${table} t;`).join('\n') + '\nCOMMIT;';
  const result = spawnSync('docker', ['exec', '-i', 'backend-db-1', 'psql', '-X', '-qAt', '-v', 'ON_ERROR_STOP=1', '-U', 'pl', '-d', 'promptark'], { input: query, maxBuffer: 256 * 1024 * 1024 });
  if (result.status !== 0) throw Error('Local export failed; inspect database connectivity');
  const data = Object.fromEntries(result.stdout.toString().trim().split('\n').map(line => { const row = JSON.parse(line); return [row.table, row.rows]; }));
  const sql = syncSql(data);
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const file = resolve(directory, 'square-sync.sql');
  writeFileSync(file, sql, { mode: 0o600 });
  const manifest = { counts: Object.fromEntries(tables.map(t => [t, data[t].length])), sha256: createHash('sha256').update(sql).digest('hex') };
  writeFileSync(resolve(directory, 'manifest.json'), JSON.stringify(manifest, null, 2), { mode: 0o600 });
  console.log(JSON.stringify(manifest));
}
if (process.argv[1] === fileURLToPath(import.meta.url)) main();
