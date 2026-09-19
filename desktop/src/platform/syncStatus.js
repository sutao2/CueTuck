import { getLocalSetting, setLocalSetting } from './library.js';

export function notifySyncStatus() {
  window.dispatchEvent(new Event('library-sync-status'));
}

export async function saveSyncResult(email, message) {
  await setLocalSetting(`sync_result:${encodeURIComponent(email)}`, JSON.stringify({ at: Date.now(), message }));
  notifySyncStatus();
}

export async function readSyncResult(email) {
  const raw = await getLocalSetting(`sync_result:${encodeURIComponent(email)}`);
  if (!raw) return null;
  try { return JSON.parse(raw); } catch { return null; }
}
