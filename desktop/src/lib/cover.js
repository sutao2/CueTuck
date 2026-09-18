import { assetUrl, validateAssets } from '../platform/assets.js';

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
  return validateAssets(parseCoverUrls(collection.cover_json).slice(0, collection.cover_type === 'single' ? 1 : 9).map(coverAsset));
}

function coverAsset(url, index) {
  const match = /^data:[^;,]*;base64,([A-Za-z0-9+/]*={0,2})$/i.exec(url);
  if (!match || match[1].length % 4) throw new Error('合集封面不是可上传的本地图片，请在编辑合集中重新选择图片');
  // FileReader's MIME can come from a misleading extension. Keep the bytes and
  // derive both upload metadata fields from the same signatures as asset validation.
  const data = match[1], header = atob(data.slice(0, 24));
  const ext = header.startsWith('\x89PNG\r\n\x1a\n') ? 'png'
    : header.startsWith('\xff\xd8\xff') ? 'jpg'
    : /^GIF8[79]a/.test(header) ? 'gif'
    : header.startsWith('RIFF') && header.slice(8, 12) === 'WEBP' ? 'webp' : '';
  if (!ext) throw new Error(`第 ${index + 1} 张封面无法识别，请选择 PNG、JPEG、GIF 或 WebP 图片`);
  return { id: crypto.randomUUID(), name: `cover-${index + 1}.${ext}`, mime: ext === 'jpg' ? 'image/jpeg' : `image/${ext}`, data };
}

export function normalizeCoverUrls(urls) {
  return validateAssets(urls.map(coverAsset)).map(assetUrl);
}
