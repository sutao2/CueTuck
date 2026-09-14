import { downloadPublishedAsset } from './privateMedia.js';
const cache = new Map(), pending = new Map(), waiting = [];
let bytes = 0, active = 0;
const MAX_BYTES = 16 * 1024 * 1024;
async function slot(task) {
  if (active >= 3) await new Promise(resolve => waiting.push(resolve));
  else active++;
  try { return await task(); }
  finally { const next = waiting.shift(); if (next) next(); else active--; }
}
export function publishedImage(itemId, file, token) {
  const key = JSON.stringify([itemId, file, token || null]);
  const hit = cache.get(key);
  if (hit) { cache.delete(key); cache.set(key, hit); return Promise.resolve(hit); }
  if (pending.has(key)) return pending.get(key);
  const task = slot(() => downloadPublishedAsset(itemId, file, token)).then(asset => {
    const size = asset.data.length * 2;
    if (size <= MAX_BYTES) {
      cache.set(key, asset); bytes += size;
      while (bytes > MAX_BYTES || cache.size > 24) {
        const oldest = cache.keys().next().value;
        bytes -= cache.get(oldest).data.length * 2; cache.delete(oldest);
      }
    }
    return asset;
  }).finally(() => pending.delete(key));
  pending.set(key, task); return task;
}
