import { timestampMillis } from '../platform/library.js';

const normalize = value => String(value ?? '').trim().toLowerCase();
const compare = (a, b) => a < b ? -1 : a > b ? 1 : 0;

// Relevance always wins; usage only orders equally relevant local results.
export function rankLauncherResults(rows, query, favoriteIds = [], now = Date.now()) {
  const needle = normalize(query), favorites = new Set(favoriteIds);
  if (!needle) return [];
  return rows.map(row => {
    const title = normalize(row.title);
    const tier = title === needle ? 0 : title.startsWith(needle) ? 1 : title.includes(needle) ? 2 : 3;
    const used = timestampMillis(row.last_used_at);
    const recency = used > 0 && Number.isFinite(used) ? Math.max(0, 1 - Math.max(0, now - used) / (30 * 86400000)) * 30 : 0;
    const count = Math.max(0, Number(row.use_count) || 0);
    const score = recency + Math.min(20, Math.log2(count + 1) * 4) + (favorites.has(row.id) ? 15 : 0);
    return { row, title, tier, score };
  }).sort((a, b) => a.tier - b.tier || b.score - a.score || compare(a.title, b.title) || compare(a.row.id, b.row.id))
    .map(({ row }) => row);
}
