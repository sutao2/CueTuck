import { test } from 'node:test';
import assert from 'node:assert/strict';
import { previewEnvironment } from './build-preview.mjs';

test('Windows and explicit identities do not read local signing secrets', () => {
  const read = () => { throw new Error('must not read'); };
  assert.deepEqual(previewEnvironment({X:'1'}, 'win32', read), {X:'1'});
  assert.deepEqual(previewEnvironment({APPLE_SIGNING_IDENTITY:'Developer ID test'}, 'darwin', read), {APPLE_SIGNING_IDENTITY:'Developer ID test'});
});
test('Mac preview reuses existing identity and certificate without mutating caller environment', () => {
  const read = path => path.endsWith('identity.json') ? JSON.stringify({identity:'A'.repeat(40),keychain:'/test/build.keychain-db'}) : path.endsWith('signing.p12') ? Buffer.from('synthetic-p12') : 'synthetic-password\n';
  const original = {TAURI_SIGNING_PRIVATE_KEY:'separate-update-key'};
  const result = previewEnvironment(original, 'darwin', read);
  assert.equal(result.APPLE_SIGNING_IDENTITY, 'A'.repeat(40));
  assert.equal(result.CUETUCK_SIGNING_KEYCHAIN, '/test/build.keychain-db');
  assert.equal(result.APPLE_CERTIFICATE_PASSWORD, undefined);
  assert.equal(original.APPLE_CERTIFICATE, undefined);
  assert.equal(result.TAURI_SIGNING_PRIVATE_KEY, original.TAURI_SIGNING_PRIVATE_KEY);
});
test('Missing or damaged local material fails without regenerating identity or leaking errors', () => {
  for (const read of [() => { throw new Error('private detail'); }, () => '{"identity":"-"}']) {
    assert.throws(() => previewEnvironment({}, 'darwin', read), e => e.message.includes('固定签名身份') && !e.message.includes('private detail'));
  }
});
