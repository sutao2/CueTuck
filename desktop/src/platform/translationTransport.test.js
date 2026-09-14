import { afterEach, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { squareVersions } from './translation.js';
import { loginSession, resetMemorySession, setSessionTransport } from './session.js';
const invoke = vi.hoisted(() => vi.fn(async () => ({ en: { status: 'queued' } })));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
afterEach(() => { delete window.__TAURI_INTERNALS__; resetMemorySession(); invoke.mockClear(); vi.unstubAllGlobals(); });
async function login() {
  setSessionTransport(async () => ({ email: 'translation@example.test', access_token: 'translation-test-token' }));
  await loginSession({ email: 'translation@example.test', password: 'fixture' });
}
it('keeps the translation native command compatible with the shared camelCase IPC transport', async () => {
  window.__TAURI_INTERNALS__ = {};
  await login();
  await squareVersions('prompt-1', 'en');
  expect(invoke).toHaveBeenCalledExactlyOnceWith('square_translations', { id: 'prompt-1', target: 'en', accessToken: 'translation-test-token' });
  // Check the receiving Rust contract too: mocking invoke alone cannot detect a
  // rename_all mismatch that silently drops an optional access token.
  const source = readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), '../../src-tauri/src/commands/square.rs'), 'utf8');
  const attribute = source.match(/(#\[tauri::command[^\n]*\])\s*pub async fn square_translations\b/)[1];
  expect(attribute).not.toContain('snake_case');
});
it('sends the current credential when polling native translations and none after logout', async () => {
  window.__TAURI_INTERNALS__ = {};
  await login();
  await squareVersions('prompt-1');
  expect(invoke).toHaveBeenLastCalledWith('square_translations', { id: 'prompt-1', target: null, accessToken: 'translation-test-token' });
  resetMemorySession();
  await squareVersions('prompt-1', 'en');
  expect(invoke).toHaveBeenLastCalledWith('square_translations', { id: 'prompt-1', target: 'en', accessToken: null });
});
it('carries the same login credential on browser translation requests', async () => {
  await login();
  const fetch = vi.fn(async () => ({ ok: true, json: async () => ({ en: { status: 'queued' } }) }));
  vi.stubGlobal('fetch', fetch);
  await squareVersions('prompt-1', 'en');
  expect(fetch).toHaveBeenCalledWith(expect.stringContaining('/prompt-1/translations/en'), { method: 'POST', headers: { Authorization: 'Bearer translation-test-token' } });
});
it('continues to report rejected authentication without retrying anonymously', async () => {
  await login();
  const fetch = vi.fn(async () => ({ ok: false, status: 401 }));
  vi.stubGlobal('fetch', fetch);
  await expect(squareVersions('prompt-1', 'en')).rejects.toThrow('请先登录');
  expect(fetch).toHaveBeenCalledTimes(1);
});
