export const MAC_TRAFFIC_LIGHT_INSET_PX = 78;

const MAC_KEYS = {
  Control: "⌃",
  Ctrl: "⌃",
  Alt: "⌥",
  Option: "⌥",
  Shift: "⇧",
  Meta: "⌘",
  Command: "⌘",
  Super: "⌘",
};

export function detectHost({ platform, userAgent } = {}) {
  const plat = platform ?? (typeof navigator !== "undefined" ? navigator.platform : "");
  const ua = userAgent ?? (typeof navigator !== "undefined" ? navigator.userAgent : "");
  if (/Mac|iPhone|iPad/i.test(plat) || /Mac OS X/i.test(ua)) return "macos";
  if (/Win/i.test(plat) || /Windows/i.test(ua)) return "windows";
  return "other";
}

export function trafficLightInsetPx(host = detectHost()) {
  return host === "macos" ? MAC_TRAFFIC_LIGHT_INSET_PX : 0;
}

export function formatShortcutLabel(combo, host = detectHost()) {
  const parts = String(combo)
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);
  if (host === "macos") {
    return parts.map((part) => MAC_KEYS[part] ?? part).join("");
  }
  return parts.map((part) => (part === "Control" ? "Ctrl" : part)).join(" ");
}

export function applyHostChrome(target, host = detectHost()) {
  if (!target?.classList) return host;
  target.classList.toggle("host-mac", host === "macos");
  target.classList.toggle("host-windows", host === "windows");
  return host;
}
