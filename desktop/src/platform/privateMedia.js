import { invokeCommand } from './tauri.js';
import { validateAssets, assetSize } from './assets.js';

const native = () => Boolean(window.__TAURI_INTERNALS__);
const base = 'http://127.0.0.1:8787';
const bytesOf = asset => Uint8Array.from(atob(asset.data), c => c.charCodeAt(0));
export async function assetHash(asset) {
  if (native()) return invokeCommand('hash_private_asset', { asset });
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytesOf(asset)))].map(n => n.toString(16).padStart(2, '0')).join('');
}
export function validateReferences(refs) {
  if (!Array.isArray(refs) || refs.length > 12) throw new Error('附件引用数量无效');
  const ids = new Set(); let total = 0;
  for (const ref of refs) {
    if (!ref || !/^[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$/i.test(ref.id) || ids.has(ref.id)
      || !/^media\.[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$/i.test(ref.media_id)
      || !Number.isSafeInteger(ref.size) || ref.size < 0 || ref.size > 5 * 1024 * 1024
      || !/^[a-f0-9]{64}$/.test(ref.sha256) || typeof ref.name !== 'string' || typeof ref.mime !== 'string') throw new Error('附件引用无效');
    ids.add(ref.id); total += ref.size;
  }
  if (total > 20 * 1024 * 1024) throw new Error('合并后附件超过 20 MiB，请先整理附件');
  return refs;
}
export async function verifyAsset(asset, ref) {
  validateReferences([ref]); validateAssets([asset]);
  if (asset.id !== ref.id || asset.name !== ref.name || asset.mime !== ref.mime || assetSize(asset) !== ref.size || await assetHash(asset) !== ref.sha256) throw new Error('附件内容校验失败');
  return asset;
}
function check(response) {
  if (response.ok) return;
  if (response.status === 401) throw new Error('登录已失效，请重新登录');
  if (response.status === 409) throw new Error('附件正在上传或账号存储配额已满，请稍后重试');
  throw new Error(`附件传输失败（${response.status}），请重试`);
}
export async function uploadPrivateAsset(asset, token) {
  validateAssets([asset]);
  if (native()) return invokeCommand('upload_private_asset', { access_token: token, asset });
  const form = new FormData(); form.append('file', new Blob([bytesOf(asset)], { type: asset.mime }), asset.name);
  const response = await fetch(`${base}/v1/media/upload`, { method: 'POST', headers: { Authorization: `Bearer ${token}` }, body: form, redirect: 'error', signal: AbortSignal.timeout(45000) });
  check(response);
  const value = await response.json();
  const ref = { id: asset.id, media_id: value.id, name: value.name, mime: value.mime, size: value.size, sha256: value.sha256 };
  await verifyAsset(asset, ref);
  return ref;
}
export async function downloadPrivateAsset(reference, token) {
  validateReferences([reference]);
  if (native()) return invokeCommand('download_private_asset', { access_token: token, reference });
  const response = await fetch(`${base}/v1/media/${reference.media_id}/content`, { headers: { Authorization: `Bearer ${token}` }, redirect: 'error', signal: AbortSignal.timeout(45000) });
  return readAsset(response, reference);
}
export async function downloadPublishedAsset(itemId, reference, token) {
  validateReferences([reference]);
  if (native()) return invokeCommand('download_published_asset', { item_id: itemId, access_token: token || null, reference });
  const response = await fetch(`${base}/v1/square/items/${encodeURIComponent(itemId)}/assets/${reference.id}`, { headers: token ? { Authorization: `Bearer ${token}` } : {}, redirect: 'error', signal: AbortSignal.timeout(45000) });
  return readAsset(response, reference);
}
async function readAsset(response, reference) {
  check(response);
  const reader = response.body.getReader(); const bytes = new Uint8Array(reference.size); let offset = 0;
  try {
    while (true) {
      const { value, done } = await reader.read(); if (done) break;
      if (offset + value.length > bytes.length) throw new Error('附件长度校验失败');
      bytes.set(value, offset); offset += value.length;
    }
  } finally { await reader.cancel(); }
  if (offset !== bytes.length) throw new Error('附件长度校验失败');
  let raw = ''; for (let i = 0; i < bytes.length; i += 8192) raw += String.fromCharCode(...bytes.subarray(i, i + 8192));
  return verifyAsset({ id: reference.id, name: reference.name, mime: reference.mime, data: btoa(raw) }, reference);
}
