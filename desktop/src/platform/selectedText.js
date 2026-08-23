export function detectPlatform(userAgent = globalThis.navigator?.userAgent ?? "") {
  if (/Mac/i.test(userAgent)) return "macos";
  if (/Win/i.test(userAgent)) return "windows";
  if (/Linux/i.test(userAgent)) return "linux";
  return "unknown";
}

export function supportsSelectedText(platform = detectPlatform()) {
  return platform === "macos";
}
