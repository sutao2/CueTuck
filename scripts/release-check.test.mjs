import { test } from 'node:test';
import assert from 'node:assert/strict';
import { validateRelease } from './release-check.mjs';
const input = () => ({ version: '0.1.0', cargoVersion: '0.1.0', config: { version: '0.1.0', plugins: { updater: { endpoints: ['https://github.com/example/repo/releases/latest/download/latest.json'], pubkey: Buffer.from('untrusted comment: test\nRWTESTKEY\n').toString('base64') } } }, frontendUpdates: 'https://api.github.com/repos/example/repo/releases', nativeUpdates: 'https://api.github.com/repos/example/repo/releases https://github.com/example/repo/releases/download/' });
test('local preflight does not require private material', () => assert.deepEqual(validateRelease(input()), []));
test('production rejects absent origins and signing credentials', () => assert.equal(validateRelease({ ...input(), production: true }).length, 2));
test('production rejects inconsistent origins, loopback and unsafe URLs', () => {
  for (const [a, b] of [['https://localhost', 'https://localhost'], ['https://a.test', 'https://b.test'], ['http://a.test', 'http://a.test'], ['https://user:pass@a.test', 'https://a.test']]) assert.ok(validateRelease({ ...input(), production: true, env: { PROMPTARK_API_BASE: a, VITE_API_BASE: b, TAURI_SIGNING_PRIVATE_KEY: 'synthetic' } }).length);
});
test('version and updater source mismatch are rejected', () => assert.equal(validateRelease({ ...input(), cargoVersion: '0.2.0', nativeUpdates: '' }).length, 2));
