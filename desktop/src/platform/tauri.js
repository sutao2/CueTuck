import { getSession, getSessionEpoch, refreshSession } from './session.js';

// Tauri's command macro accepts camelCase argument names by default.
// Only command arguments are converted; nested API/SQLite records keep their schema.
export async function invokeCommand(command, args) {
  const epoch = getSessionEpoch();
  const { invoke } = await import("@tauri-apps/api/core");
  const payload = args && Object.fromEntries(Object.entries(args).map(([key, value]) => [
    key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase()), value,
  ]));
  if (!payload?.accessToken || command === 'logout_local_session') {
    try { return await invoke(command, payload); }
    catch (error) {
      if (command !== 'refresh_local_session' && String(error?.message || error) === 'SESSION_EXPIRED') throw new Error('登录已失效，请重新登录');
      throw error;
    }
  }
  const sentToken = payload.accessToken;
  try { return await invoke(command, payload); }
  catch (error) {
    if (String(error?.message || error) !== 'SESSION_EXPIRED') throw error;
    if (getSessionEpoch() !== epoch || !getSession().loggedIn) throw new Error('登录已变更，请重新操作');
    if (getSession().accessToken === sentToken) await refreshSession();
    if (getSessionEpoch() !== epoch || !getSession().loggedIn) throw new Error('登录已失效，请重新登录');
    try { return await invoke(command, { ...payload, accessToken: getSession().accessToken }); }
    catch (retryError) {
      if (String(retryError?.message || retryError) === 'SESSION_EXPIRED') throw new Error('登录已失效，请重新登录');
      throw retryError;
    }
  }
}
