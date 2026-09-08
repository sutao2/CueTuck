import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { csv, validate, importSql } from './import-community.mjs';
const manifest = JSON.parse(readFileSync(new URL('../samples/community/prompts.json',import.meta.url)));
test('CSV handles commas, escaped quotes and multiline prompts', () => {
  assert.deepEqual(csv('act,prompt\r\nA,"a, b\n""quoted"""\r\n'), [{act:'A',prompt:'a, b\n"quoted"'}]);
  assert.throws(() => csv('act,prompt\nA,"broken'));
});
test('reviewed samples contain provenance and valid categories', () => {
  validate(manifest);
  assert.equal(manifest.records.filter(r=>r.reference.images.length).length,12);
  assert.equal(new Set(manifest.records.map(r=>r.reference.repository)).size,3);
  assert.ok(new Set(manifest.records.map(r=>r.category_id.split('-')[1])).size >= 9);
  for (const r of manifest.records.filter(r=>r.reference.license==='MIT')) assert.match(r.content,/Copyright \(c\) 2025 plex/);
});
test('import is transactional, append-only and duplicate IDs are rejected', () => {
  const sql = importSql(manifest);
  assert.ok(sql.startsWith('BEGIN;')); assert.ok(sql.endsWith('COMMIT;'));
  assert.match(sql,/ON CONFLICT\(id\) DO NOTHING/);
  assert.doesNotMatch(sql,/\b(DELETE FROM|TRUNCATE|DO UPDATE|DROP TABLE)\b/);
  const duplicate = structuredClone(manifest); duplicate.records[1] = duplicate.records[0];
  assert.throws(()=>validate(duplicate));
});
