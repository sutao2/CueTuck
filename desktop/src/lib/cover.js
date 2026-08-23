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
