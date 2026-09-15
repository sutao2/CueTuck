import { invokeCommand } from './tauri.js';
import { validateAssets } from './assets.js';
import { referenceImages } from '../lib/squareReference.js';

// Public reference bytes only: short-lived memory cache, never account attachments.
const imageCache = new Map();
const CACHE_LIMIT = 32 * 1024 * 1024, CACHE_TTL = 5 * 60_000;
export function clearReferenceImageCache() { imageCache.clear(); }
function cachedImage(url) {
  for (const [key, value] of imageCache) if (Date.now() - value.saved > CACHE_TTL) imageCache.delete(key);
  return imageCache.get(url)?.asset;
}
function rememberImage(url, asset) {
  imageCache.delete(url);
  let size = asset.data.length * 2;
  for (const value of imageCache.values()) size += value.asset.data.length * 2;
  for (const [key, value] of imageCache) { if (size <= CACHE_LIMIT) break; size -= value.asset.data.length * 2; imageCache.delete(key); }
  if (asset.data.length * 2 <= CACHE_LIMIT) imageCache.set(url, { asset, saved: Date.now() });
}
async function downloadBrowserImage(url, index, signal) {
  const cached = cachedImage(url);
  if (cached) return { ...cached, id: crypto.randomUUID(), name: `参考图-${index+1}.${cached.name.split('.').at(-1)}` };
  const response = await fetch(url, { credentials:'omit', referrerPolicy:'no-referrer', redirect:'error', signal });
  if (!response.ok) throw Error('图片连接失败');
  const reader = response.body.getReader(), chunks = []; let size = 0;
  try {
    while (true) {
      const {value,done} = await reader.read(); if (done) break;
      size += value.length; if (size > 5 * 1024 * 1024) throw Error('图片超过 5 MiB');
      chunks.push(value);
    }
  } finally { await reader.cancel(); }
  let raw = ''; for (const bytes of chunks) for (let i=0;i<bytes.length;i+=8192) raw += String.fromCharCode(...bytes.subarray(i,i+8192));
  const type = raw.startsWith('\x89PNG\r\n\x1a\n') ? ['png','image/png'] : raw.startsWith('\xff\xd8\xff') ? ['jpg','image/jpeg'] : /^GIF8[79]a/.test(raw) ? ['gif','image/gif'] : raw.startsWith('RIFF') && raw.slice(8,12)==='WEBP' ? ['webp','image/webp'] : null;
  if (!type) throw Error('不支持的图片内容');
  const asset = {id:crypto.randomUUID(),name:`参考图-${index+1}.${type[0]}`,mime:type[1],data:btoa(raw)};
  validateAssets([asset]);
  rememberImage(url, asset);
  return asset;
}

export async function downloadReferenceImages(item, onProgress) {
  const urls = referenceImages(item);
  if (!urls.length) return [];
  onProgress?.({ completed: 0, total: urls.length });
  if (window.__TAURI_INTERNALS__) {
    const { Channel } = await import('@tauri-apps/api/core');
    let settled = false;
    const on_progress = new Channel(progress => { if (!settled) onProgress?.(progress); });
    try {
      const assets = await invokeCommand('download_reference_images', { urls, on_progress });
      validateAssets(assets);
      return assets;
    } finally { settled = true; }
  }
  const assets = new Array(urls.length), controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 45_000);
  let next = 0, completed = 0, failure;
  async function worker() {
    while (!failure && next < urls.length) {
      const index = next++;
      try {
        assets[index] = await downloadBrowserImage(urls[index], index, controller.signal);
        if (failure) return;
        validateAssets(assets.filter(Boolean));
        onProgress?.({ completed: ++completed, total: urls.length });
      } catch (error) {
        if (!failure) failure = Error(`参考图 ${index+1} 下载失败：${error.message || error}，请重试`);
        controller.abort();
      }
    }
  }
  try {
    await Promise.all(Array.from({ length: Math.min(3, urls.length) }, worker));
    if (failure) throw failure;
    return assets;
  } finally { clearTimeout(timer); }
}
