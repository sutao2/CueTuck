import pkg from "../../package.json";

const RELEASES_URL = "https://api.github.com/repos/sutao2/PromptArk/releases";

let testTransport = null;
let installTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function tauriInvoke(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}

export function normalizeUpdateChannel(channel) {
  return channel === "preview" ? "preview" : "stable";
}

export function resetUpdates() {
  testTransport = null;
  installTransport = null;
}

export function setUpdateTransport(transport) {
  testTransport = transport;
}

export function setInstallTransport(transport) {
  installTransport = transport;
}

function fromReleases(releases, channel = "stable") {
  if (!Array.isArray(releases) || releases.length === 0) {
    return { available: false, notes: "" };
  }
  const wantPreview = channel === "preview";
  const latest = releases.find((row) => Boolean(row?.prerelease) === wantPreview);
  if (!latest) {
    return { available: false, notes: "" };
  }
  const remote = String(latest.tag_name ?? "").replace(/^v/i, "");
  const current = String(pkg.version ?? "").replace(/^v/i, "");
  const available = Boolean(remote) && remote !== current;
  return {
    available,
    notes: String(latest.body ?? ""),
    version: remote,
  };
}

export async function checkForUpdates({ channel } = {}) {
  const selected = normalizeUpdateChannel(channel);
  if (testTransport) return testTransport({ channel: selected });
  if (isTauri()) {
    return tauriInvoke("check_for_updates", { channel: selected });
  }
  const response = await fetch(`${RELEASES_URL}?per_page=5`, {
    headers: { Accept: "application/vnd.github+json" },
  });
  if (!response.ok) {
    throw new Error("检查失败");
  }
  return fromReleases(await response.json(), selected);
}

export async function queueUpdateInstall({ autoDownload, channel } = {}) {
  const selected = normalizeUpdateChannel(channel);
  if (!autoDownload) {
    return { queued: false, via: "updater" };
  }
  if (installTransport) {
    return installTransport({ channel: selected });
  }
  if (isTauri()) {
    return tauriInvoke("queue_update_install", { channel: selected });
  }
  return { queued: false, via: "updater" };
}
