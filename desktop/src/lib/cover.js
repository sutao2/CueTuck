export function parseCoverUrls(coverJson) {
  try {
    const parsed = JSON.parse(coverJson || "[]");
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((item) => typeof item === "string" && item.trim());
  } catch {
    return [];
  }
}

export function coverSlots(coverJson, size = 9) {
  const urls = parseCoverUrls(coverJson);
  return Array.from({ length: size }, (_, index) => urls[index] || "");
}

export function serializeCoverUrls(coverType, coverUrls = []) {
  if (coverType === "none") return "[]";
  const urls = (coverUrls || []).filter((item) => typeof item === "string" && item.trim());
  const limit = coverType === "single" ? 1 : 9;
  return JSON.stringify(urls.slice(0, limit));
}

// Covers are stored locally as data URLs. Publication sends verified media references instead.
export function collectionCoverAssets(collection) {
  if (!collection || collection.cover_type === 'none') return [];
  return parseCoverUrls(collection.cover_json).slice(0, collection.cover_type === 'single' ? 1 : 9).map((url, index) => {
    const match = /^data:(image\/(?:png|jpeg|gif|webp));base64,([A-Za-z0-9+/=]+)$/.exec(url);
    if (!match) throw new Error('合集封面不是可上传的本地图片，请在编辑合集中重新选择图片');
    const ext = match[1] === 'image/jpeg' ? 'jpg' : match[1].slice(6);
    return { id: crypto.randomUUID(), name: `cover-${index + 1}.${ext}`, mime: match[1], data: match[2] };
  });
}
