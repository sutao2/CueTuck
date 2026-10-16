import { onMounted, onUnmounted, ref } from 'vue';
import { contentVersions } from './contentVersions.js';
import { getSession, subscribeSession } from './session.js';
import { getLocalSetting } from './library.js';

const interval = 5 * 60 * 1000;
const sourceKey = row => `${row.kind}:${row.remote_id}`;
export const localUpdateKey = row => `${row.kind}:${row.local_id}`;
export const skillUpdateKey = item => `${item.root_id}:${item.path}`;
export function skillUpdateRows(skills) {
  return skills.filter(item => item.community).map(item => ({
    kind: 'skill', remote_id: item.community.id, version: item.community.version,
    key: skillUpdateKey(item),
  }));
}

// Only version summaries are read here. Downloading and applying stay in the confirmation flow.
export function useContentUpdateBadges(loadRows) {
  const badges = ref({});
  const cache = new Map();
  let disposed = false, running = false, queued = false, generation = 0, timer, unsubscribe;
  async function refresh() {
    if (disposed) return;
    if (running) { queued = true; return; }
    running = true;
    const current = generation, token = getSession().accessToken;
    const valid = () => !disposed && generation === current && getSession().accessToken === token;
    try {
      if (document.visibilityState === 'hidden' || navigator.onLine === false) return;
      if (await getLocalSetting('square_access') === '0') { badges.value = {}; cache.clear(); return; }
      const rows = (await loadRows()).filter(row => row.remote_id && Number.isInteger(row.version) && row.version > 0);
      if (!valid()) return;
      const sources = [...new Map(rows.map(row => [sourceKey(row), row])).values()];
      const versions = new Map();
      let next = 0;
      await Promise.all(Array.from({ length: Math.min(3, sources.length) }, async () => {
        while (next < sources.length && valid()) {
          const row = sources[next++], key = sourceKey(row), cached = cache.get(key);
          if (cached && Date.now() - cached.time < interval) { versions.set(key, cached.version); continue; }
          try {
            const result = await contentVersions(row.kind, row.remote_id);
            if (!valid()) return;
            const version = Number.isInteger(result.current_version) ? result.current_version : 0;
            cache.set(key, { time: Date.now(), version });
            versions.set(key, version);
          } catch {
            // Failed checks are unknown, not "up to date". Retry on the next check.
            cache.delete(key);
          }
        }
      }));
      if (valid()) badges.value = Object.fromEntries(rows.filter(row => versions.get(sourceKey(row)) > row.version)
        .map(row => [row.key || localUpdateKey(row), versions.get(sourceKey(row))]));
    } catch { if (valid()) badges.value = {}; }
    finally {
      running = false;
      if (queued && !disposed) { queued = false; void refresh(); }
    }
  }
  const wake = () => { void refresh(); };
  onMounted(() => {
    unsubscribe = subscribeSession(() => { generation++; cache.clear(); badges.value = {}; wake(); });
    window.addEventListener('focus', wake);
    window.addEventListener('online', wake);
    document.addEventListener('visibilitychange', wake);
    timer = setInterval(wake, interval);
    wake();
  });
  onUnmounted(() => {
    disposed = true; generation++; clearInterval(timer); unsubscribe?.();
    window.removeEventListener('focus', wake);
    window.removeEventListener('online', wake);
    document.removeEventListener('visibilitychange', wake);
  });
  return { badges, refresh };
}
