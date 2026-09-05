import pkg from "../../package.json";
import parseVersion from "semver/functions/parse.js";

const RELEASES_URL = "https://api.github.com/repos/sutao2/PromptArk/releases";

let testTransport = null;
let installTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

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
  const candidates = releases.flatMap((row) => {
    const version = parseVersion(String(row?.tag_name ?? "").replace(/^v/i, ""));
    if (row?.draft || !version || Boolean(row?.prerelease) !== wantPreview || Boolean(version.prerelease.length) !== wantPreview) return [];
    return [{ row, version }];
  }).sort((a, b) => b.version.compare(a.version));
  const latest = candidates[0];
  if (!latest) {
    return { available: false, notes: "" };
  }
  const remote = String(latest.row.tag_name).replace(/^v/i, "");
  const current = parseVersion(String(pkg.version ?? ""));
  const available = Boolean(current) && latest.version.compare(current) > 0;
  return {
    available,
    notes: String(latest.row.body ?? ""),
    version: remote,
  };
}

export async function checkForUpdates({ channel } = {}) {
  const selected = normalizeUpdateChannel(channel);
  if (testTransport) return testTransport({ channel: selected });
  if (isTauri()) {
    return tauriInvoke("check_for_updates", { channel: selected });
  }
  const response = await fetch(`${RELEASES_URL}?per_page=100`, {
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
