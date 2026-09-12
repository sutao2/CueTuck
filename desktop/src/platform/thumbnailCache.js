import { firstPromptThumbnail } from './assets.js';

const cache = new Map(), pending = new Map();
const MAX_ENTRIES = 40, MAX_BYTES = 8 * 1024 * 1024;
let bytes = 0;
function remove(key) {
  const entry = cache.get(key);
  if (entry) { bytes -= entry.bytes; cache.delete(key); }
}
export function clearThumbnailCache() {
  cache.clear(); pending.clear(); bytes = 0;
}
export function cachedPromptThumbnail(promptId, revision) {
  const key = JSON.stringify([promptId, revision ?? null]);
  for (const [oldKey, entry] of cache) if (entry.promptId === promptId && oldKey !== key) remove(oldKey);
  for (const [oldKey, entry] of pending) if (entry.promptId === promptId && oldKey !== key) pending.delete(oldKey);
  const hit = cache.get(key);
  if (hit) { cache.delete(key); cache.set(key, hit); return Promise.resolve(hit.asset); }
  if (pending.has(key)) return pending.get(key).task;
  const task = firstPromptThumbnail(promptId).then(asset => {
    if (asset && pending.get(key)?.task === task) {
      const size = asset.data.length * 2;
      if (size <= MAX_BYTES) {
        cache.set(key, { promptId, asset, bytes: size }); bytes += size;
        while (cache.size > MAX_ENTRIES || bytes > MAX_BYTES) remove(cache.keys().next().value);
      }
    }
    return asset;
  }).finally(() => { if (pending.get(key)?.task === task) pending.delete(key); });
  pending.set(key, { promptId, task });
  return task;
}
