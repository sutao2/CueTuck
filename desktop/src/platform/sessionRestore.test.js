import { afterEach, beforeEach, expect, it, vi } from 'vitest';
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
import { getSession, loginSession, logoutSession, refreshSession, resetMemorySession, restoreSession } from './session.js';
beforeEach(() => { resetMemorySession(); invoke.mockReset(); window.__TAURI_INTERNALS__ = {}; localStorage.clear(); sessionStorage.clear(); });
afterEach(() => { delete window.__TAURI_INTERNALS__; resetMemorySession(); });
const pair = { email: 'restore@example.test', access_token: 'acc.restored' };
it('restores a saved native session once without persisting tokens in web storage', async () => {
  invoke.mockResolvedValue(pair);
  await Promise.all([restoreSession(), restoreSession(), refreshSession()]);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(invoke).toHaveBeenCalledWith('refresh_local_session', undefined);
  expect(getSession()).toEqual({email: pair.email, accessToken: pair.access_token, loggedIn:true});
  await restoreSession(); expect(invoke).toHaveBeenCalledTimes(1);
  expect(localStorage.length).toBe(0); expect(sessionStorage.length).toBe(0);
});
it('keeps first install as guest and skips native credentials in browser preview', async () => {
  invoke.mockResolvedValue(null); await restoreSession(); expect(getSession().loggedIn).toBe(false);
  invoke.mockClear(); delete window.__TAURI_INTERNALS__; await restoreSession(); expect(invoke).not.toHaveBeenCalled();
});
it('allows retry after a temporary restoration failure', async () => {
  invoke.mockRejectedValueOnce(new Error('offline')).mockResolvedValueOnce(pair);
  await expect(restoreSession()).rejects.toThrow('offline'); expect(getSession().loggedIn).toBe(false);
  await restoreSession(); expect(getSession().loggedIn).toBe(true);
});
it('waits for restoration before logout so a late result cannot log the user back in', async () => {
  let complete; invoke.mockImplementationOnce(() => new Promise(resolve => { complete=resolve; })).mockResolvedValue(null);
  const restore = restoreSession(); await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
  const logout = logoutSession(); complete(pair); await Promise.all([restore,logout]);
  expect(invoke).toHaveBeenLastCalledWith('logout_local_session', {accessToken:'acc.restored'});
  expect(getSession().loggedIn).toBe(false);
});
it('waits for restoration before switching accounts', async () => {
  let complete; invoke.mockImplementationOnce(() => new Promise(resolve => { complete=resolve; })).mockResolvedValue({email:'new@example.test',access_token:'acc.new'});
  const restore = restoreSession(); await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
  const login = loginSession({email:'new@example.test',password:'test'}); complete(pair); await Promise.all([restore,login]);
  expect(getSession().email).toBe('new@example.test'); expect(getSession().accessToken).toBe('acc.new');
});
it('clears saved credentials on explicit logout even without an in-memory access token', async () => {
  invoke.mockResolvedValue(null); await logoutSession();
  expect(invoke).toHaveBeenCalledWith('logout_local_session', {accessToken:null});
});
