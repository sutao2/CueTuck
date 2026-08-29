import { getLocalSetting, setLocalSetting } from "./library.js";

export function parseFavoriteIds(raw) {
  try {
    const parsed = JSON.parse(raw || "[]");
    return Array.isArray(parsed) ? parsed.map(String).filter(Boolean) : [];
  } catch {
    return [];
  }
}

export async function listLocalFavoriteIds() {
  return parseFavoriteIds(await getLocalSetting("local_favorite_ids"));
}

export async function toggleLocalFavorite(id) {
  const ids = await listLocalFavoriteIds();
  const next = ids.includes(id) ? ids.filter((item) => item !== id) : [...ids, id];
  await setLocalSetting("local_favorite_ids", JSON.stringify(next));
  return next;
}

export function filterLocalItems(items, { tab, favoriteIds } = {}) {
  if (tab === "最近") {
    return items
      .filter((item) => item.kind === "prompt" && item.last_used_at)
      .slice()
      .sort((left, right) => String(right.last_used_at).localeCompare(String(left.last_used_at)));
  }
  if (tab === "收藏") {
    return items.filter((item) => (favoriteIds ?? []).includes(item.id));
  }
  return items;
}
