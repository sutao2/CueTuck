import { afterEach, beforeEach, expect, it, vi } from 'vitest';
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
import { getSession, loginSession, logoutSession, refreshSession, resetMemorySession, restoreSession } from './session.js';
beforeEach(() => { resetMemorySession(); invoke.mockReset(); window.__TAURI_INTERNALS__ = { invoke }; localStorage.clear(); sessionStorage.clear(); });
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

it('clears in-memory login only when refresh explicitly rejects saved credentials', async () => {
  invoke.mockResolvedValueOnce(pair); await restoreSession();
  invoke.mockRejectedValueOnce('SESSION_EXPIRED');
  await expect(refreshSession()).rejects.toThrow('登录已失效');
  expect(getSession().loggedIn).toBe(false);
});

it('merges concurrent unauthorized requests and retries each once with the new token', async () => {
  const { invokeCommand } = await import('./tauri.js');
  invoke.mockResolvedValueOnce(pair); await restoreSession(); invoke.mockClear();
  invoke.mockImplementation(async (command, args) => {
    if (command === 'refresh_local_session') return {...pair, access_token:'acc.new'};
    if (args.accessToken === pair.access_token) throw 'SESSION_EXPIRED';
    return 'ok';
  });
  expect(await Promise.all([invokeCommand('get_me',{access_token:pair.access_token}),invokeCommand('get_me',{access_token:pair.access_token})])).toEqual(['ok','ok']);
  expect(invoke.mock.calls.filter(([c])=>c==='refresh_local_session')).toHaveLength(1);
  expect(invoke.mock.calls.filter(([c])=>c==='get_me')).toHaveLength(4);
});

it('does not replay writes for network failures or repeated unauthorized responses', async () => {
  const { invokeCommand } = await import('./tauri.js');
  invoke.mockResolvedValueOnce(pair); await restoreSession(); invoke.mockClear();
  invoke.mockRejectedValueOnce('network timeout');
  await expect(invokeCommand('submit_publication',{accessToken:pair.access_token})).rejects.toBe('network timeout');
  expect(invoke).toHaveBeenCalledTimes(1);
  invoke.mockClear();
  invoke.mockImplementation(async c=>{if(c==='refresh_local_session')return {...pair,access_token:'acc.new'};throw 'SESSION_EXPIRED';});
  await expect(invokeCommand('submit_publication',{accessToken:pair.access_token})).rejects.toThrow('登录已失效');
  expect(invoke).toHaveBeenCalledTimes(3);
});

it('does not replay an old request after switching accounts', async () => {
  const { invokeCommand } = await import('./tauri.js');
  invoke.mockResolvedValueOnce(pair); await restoreSession();
  let rejectOld; invoke.mockImplementationOnce(()=>new Promise((_,reject)=>{rejectOld=reject;}));
  const request=invokeCommand('get_me',{accessToken:pair.access_token});
  const failure=expect(request).rejects.toThrow('登录已变更');
  await vi.waitFor(()=>expect(rejectOld).toBeTypeOf('function'));
  invoke.mockResolvedValueOnce({email:'other@test',access_token:'acc.other'});
  await loginSession({email:'other@test',password:'test'});
  rejectOld('SESSION_EXPIRED'); await failure;
  expect(getSession().email).toBe('other@test');
  expect(invoke.mock.calls.filter(([c])=>c==='get_me')).toHaveLength(1);
});

it('reports a failed local logout cleanup instead of claiming credentials were deleted',async()=>{
 invoke.mockResolvedValueOnce(pair);await restoreSession();
 invoke.mockRejectedValueOnce('无法删除登录配置文件');
 await expect(logoutSession()).rejects.toBe('无法删除登录配置文件');
 expect(getSession().loggedIn).toBe(false);
});
