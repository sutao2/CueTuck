import { test } from 'node:test';
import assert from 'node:assert/strict';
import { validateCorpus, corpusSql } from './import-prompt-corpus.mjs';
const row = () => ({ id: 'corpus-' + 'a'.repeat(24), title: 'Lighting study', kind: 'prompt', content: "A photographer's studio", excerpt: 'Studio', category_id: 'cat-image', model: 'Stable Diffusion', reference: { repository: 'poloclub/diffusiondb', author: 'Contributors', url: 'https://huggingface.co/datasets/poloclub/diffusiondb', license: 'CC0 1.0', license_url: 'https://creativecommons.org/publicdomain/zero/1.0/', images: [] } });
test('accepts clean bodies and rejects duplicate, unsafe or unlicensed records', () => {
  assert.equal(validateCorpus([row()]), 1);
  assert.throws(() => validateCorpus([row(), row()]));
  for (const patch of [{ content: '' }, { title: '' }, { category_id: "x';drop table" }, { reference: { ...row().reference, license: 'unknown' } }, { reference: { ...row().reference, images: ['https://evil.example/x'] } }, { reference: { ...row().reference, url: 'javascript:alert(1)' } }]) assert.throws(() => validateCorpus([{ ...row(), ...patch }]));
});
test('uses an atomic append-only transaction with literal SQL data and no attribution concatenation', () => {
  const sql = corpusSql([row()]);
  assert.ok(sql.includes("photographer''s studio"));
  assert.ok(sql.startsWith('BEGIN;'));
  assert.ok(sql.endsWith('COMMIT;'));
  assert.ok(sql.includes('ON CONFLICT(id) DO NOTHING'));
  assert.ok(sql.includes('Missing enabled catalog reference'));
  assert.ok(sql.includes('Existing ID has different body'));
  assert.ok(!/DELETE FROM|TRUNCATE|UPDATE public/.test(sql));
});
