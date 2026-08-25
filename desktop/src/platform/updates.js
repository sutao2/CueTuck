import pkg from "../../package.json";

const RELEASES_URL = "https://api.github.com/repos/sutao2/PromptArk/releases";

let testTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function tauriInvoke(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}

export function resetUpdates() {
  testTransport = null;
}

export function setUpdateTransport(transport) {
  testTransport = transport;
}

function fromReleases(releases) {
  if (!Array.isArray(releases) || releases.length === 0) {
    return { available: false, notes: "" };
  }
  const latest = releases[0] ?? {};
  const remote = String(latest.tag_name ?? "").replace(/^v/i, "");
  const current = String(pkg.version ?? "").replace(/^v/i, "");
  const available = Boolean(remote) && remote !== current;
  return {
    available,
    notes: String(latest.body ?? ""),
    version: remote,
  };
}

export async function checkForUpdates() {
  if (testTransport) return testTransport();
  if (isTauri()) {
    return tauriInvoke("check_for_updates");
  }
  const response = await fetch(`${RELEASES_URL}?per_page=5`, {
    headers: { Accept: "application/vnd.github+json" },
  });
  if (!response.ok) {
    throw new Error("检查失败");
  }
  return fromReleases(await response.json());
}
