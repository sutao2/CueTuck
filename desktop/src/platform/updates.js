import pkg from "../../package.json";
import parseVersion from "semver/functions/parse.js";

const RELEASES_URL = "https://api.github.com/repos/sutao2/CueTuck/releases";

let testTransport = null;
let installTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

export function normalizeUpdateChannel(channel) {
  return channel === "preview" ? "preview" : "stable";
}

export function resetUpdates() {
  testTransport = null;
  installTransport = null;
  Object.assign(updateState, { phase:'idle', version:'', notes:'', downloaded:0, total:null, error:'', checkedAt:'' });
}

export function setUpdateTransport(transport) {
  testTransport = transport;
}

export function setInstallTransport(transport) {
  installTransport = transport;
}

function fromReleases(releases, channel = "stable") {
  if (!Array.isArray(releases) || releases.length === 0) {
    return { available: false, notes: "" };
  }
  const wantPreview = channel === "preview";
  const candidates = releases.flatMap((row) => {
    const version = parseVersion(String(row?.tag_name ?? "").replace(/^v/i, ""));
    if (row?.draft || !version || Boolean(row?.prerelease) !== wantPreview || Boolean(version.prerelease.length) !== wantPreview) return [];
    return [{ row, version }];
  }).sort((a, b) => b.version.compare(a.version));
  const latest = candidates[0];
  if (!latest) {
    return { available: false, notes: "" };
  }
  const remote = String(latest.row.tag_name).replace(/^v/i, "");
  const current = parseVersion(String(pkg.version ?? ""));
  const available = Boolean(current) && latest.version.compare(current) > 0;
  return {
    available,
    notes: String(latest.row.body ?? ""),
    version: remote,
  };
}

export async function checkForUpdates({ channel } = {}) {
  const selected = normalizeUpdateChannel(channel);
  if (testTransport) return testTransport({ channel: selected });
  if (isTauri()) {
    return tauriInvoke("check_for_updates", { channel: selected });
  }
  const response = await fetch(`${RELEASES_URL}?per_page=100`, {
    headers: { Accept: "application/vnd.github+json" },
  });
  if (!response.ok) {
    throw new Error("检查失败");
  }
  return fromReleases(await response.json(), selected);
}

export async function queueUpdateInstall({ autoDownload, channel, version } = {}) {
  const selected = normalizeUpdateChannel(channel);
  if (!autoDownload) {
    return { queued: false, via: "updater" };
  }
  if (installTransport) {
    return installTransport({ channel: selected });
  }
  if (isTauri()) {
    return tauriInvoke("queue_update_install", { channel: selected, version });
  }
  return { queued: false, via: "updater" };
}

import { reactive } from 'vue';
import { getLocalSetting } from './library.js';
export const updateState = reactive({ phase: 'idle', version: '', notes: '', downloaded: 0, total: null, error: '', checkedAt: '' });
export const defaultUpdateChannel = pkg.version.includes('-') ? 'preview' : 'stable';
const busy = () => ['checking', 'downloading', 'verifying', 'installing'].includes(updateState.phase);
export async function selectedUpdateChannel() { return (await getLocalSetting('update_channel')) || defaultUpdateChannel; }
export async function refreshUpdate({ channel, autoDownload = false } = {}) {
  if (busy() || updateState.phase === 'ready') return;
  Object.assign(updateState, { phase: 'checking', error: '' });
  try {
    const result = await checkForUpdates({ channel: channel || await selectedUpdateChannel() });
    Object.assign(updateState, { phase: result.available ? 'available' : 'current', version: result.version || '', notes: result.notes || '', checkedAt: new Date().toLocaleString() });
    if (result.available && autoDownload) await downloadUpdate(channel);
  } catch (error) { Object.assign(updateState, { phase: 'error', error: error.message || '检查失败' }); }
}
export async function downloadUpdate(channel) {
  if (busy() || !updateState.version) return;
  if (!isTauri() && !installTransport) { updateState.error = '请在桌面客户端下载安装更新'; return; }
  Object.assign(updateState, { phase: 'downloading', downloaded: 0, total: null, error: '' });
  let unlisten;
  try {
    if (isTauri()) {
      const { listen } = await import('@tauri-apps/api/event');
      unlisten = await listen('update-progress', ({ payload }) => {
        if (['downloading','verifying'].includes(payload?.phase)) Object.assign(updateState, payload);
      });
    }
    const result = await queueUpdateInstall({ autoDownload: true, channel: channel || await selectedUpdateChannel(), version: updateState.version });
    if (!result?.ready) throw new Error('未能下载更新包，请重试');
    Object.assign(updateState, { phase: 'ready', downloaded: result.size, total: result.size });
  } catch (error) { Object.assign(updateState, { phase: 'available', error: error.message || '下载失败，请重试' }); }
  finally { unlisten?.(); }
}
export async function installUpdate() {
  if (updateState.phase !== 'ready') return;
  updateState.phase = 'installing'; updateState.error = '';
  try { await tauriInvoke('install_downloaded_update', { version: updateState.version }); }
  catch (error) { updateState.phase = 'ready'; updateState.error = String(error?.message || error); }
}
export function startUpdateChecks() {
  if (!isTauri()) return () => {};
  const check = async () => { try { if (!busy() && !['available','ready'].includes(updateState.phase)) await refreshUpdate({ autoDownload: (await getLocalSetting('auto_download')) === '1' }); } catch { /* A later check retries unavailable local preferences. */ } };
  const first = setTimeout(check, 10000);
  const timer = setInterval(check, 4 * 60 * 60 * 1000);
  return () => { clearTimeout(first); clearInterval(timer); };
}
