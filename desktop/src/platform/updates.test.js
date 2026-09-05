import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  checkForUpdates,
  queueUpdateInstall,
  resetUpdates,
  setInstallTransport,
  setUpdateTransport,
} from "./updates.js";

const tauriVersion = JSON.parse(
  readFileSync(
    resolve(dirname(fileURLToPath(import.meta.url)), "../../src-tauri/tauri.conf.json"),
    "utf8",
  ),
).version;

describe("updates", () => {
  beforeEach(() => {
    resetUpdates();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("asks GitHub Releases and reports none when the list is empty", async () => {
    const fetchSpy = vi.fn(async () => ({
      ok: true,
      json: async () => [],
    }));
    vi.stubGlobal("fetch", fetchSpy);
    const result = await checkForUpdates();
    expect(fetchSpy.mock.calls[0][0]).toMatch(/api\.github\.com\/repos\/.+\/releases/);
    expect(result.available).toBe(false);
  });

  it("does not treat a failed GitHub read as no updates", async () => {
    const fetchSpy = vi.fn(async () => ({
      ok: false,
      status: 404,
      json: async () => ({}),
    }));
    vi.stubGlobal("fetch", fetchSpy);
    await expect(checkForUpdates()).rejects.toThrow(/检查失败/);
    expect(fetchSpy.mock.calls[0][0]).toMatch(/api\.github\.com\/repos\/.+\/releases/);
  });

  it("does not claim a store listing when a transport reports no release", async () => {
    setUpdateTransport(async () => ({ available: false, notes: "" }));
    const result = await checkForUpdates();
    expect(result.available).toBe(false);
    expect(JSON.stringify(result)).not.toMatch(/商店/);
  });

  it("picks a prerelease only on the preview channel", async () => {
    vi.stubGlobal("fetch", async () => ({
      ok: true,
      json: async () => [
        { tag_name: "v0.2.0-beta", prerelease: true, body: "beta notes" },
        { tag_name: "v0.1.0", prerelease: false, body: "stable notes" },
      ],
    }));
    const preview = await checkForUpdates({ channel: "preview" });
    expect(preview.version).toBe("0.2.0-beta");
    expect(preview.notes).toBe("beta notes");
    const stable = await checkForUpdates({ channel: "stable" });
    expect(stable.version).toBe("0.1.0");
    expect(stable.notes).toBe("stable notes");
  });

  it("reports no update when the latest stable tag matches the tauri build", async () => {
    vi.stubGlobal("fetch", async () => ({
      ok: true,
      json: async () => [
        { tag_name: `v${tauriVersion}`, prerelease: false, body: "same build" },
      ],
    }));
    const result = await checkForUpdates({ channel: "stable" });
    expect(result.version).toBe(tauriVersion);
    expect(result.available).toBe(false);
  });

  it("does not queue an install when auto-download is off", async () => {
    const install = vi.fn(async () => ({ queued: true, via: "updater" }));
    setInstallTransport(install);
    const result = await queueUpdateInstall({
      autoDownload: false,
      channel: "stable",
    });
    expect(result.queued).toBe(false);
    expect(install).not.toHaveBeenCalled();
    expect(JSON.stringify(result)).not.toMatch(/商店/);
  });

  it.each(["v0.0.9", `v${tauriVersion}+rebuild`])("does not offer older or equivalent version %s", async (tag_name) => {
    vi.stubGlobal("fetch", async () => ({ ok: true, json: async () => [{ tag_name }] }));
    expect((await checkForUpdates()).available).toBe(false);
  });

  it("chooses the highest valid non-draft version rather than the first release", async () => {
    vi.stubGlobal("fetch", async () => ({ ok: true, json: async () => [
      { tag_name: "v99.0.0", draft: true },
      { tag_name: "nightly" },
      { tag_name: "v0.2.0" },
      { tag_name: "V0.10.0", body: "newest" },
      { tag_name: "v1.0.0-beta.1", prerelease: true },
    ] }));
    expect(await checkForUpdates()).toMatchObject({ available: true, version: "0.10.0", notes: "newest" });
  });

  it("orders prerelease numbers semantically and rejects invalid or mislabeled releases", async () => {
    vi.stubGlobal("fetch", async () => ({ ok: true, json: async () => [
      { tag_name: "v1.0.0-beta.2", prerelease: true },
      { tag_name: "v1.0.0-beta.10", prerelease: true },
      { tag_name: "v2.0.0-beta.1", prerelease: false },
      { tag_name: "bad", prerelease: false },
    ] }));
    expect((await checkForUpdates({ channel: "preview" })).version).toBe("1.0.0-beta.10");
    expect((await checkForUpdates()).available).toBe(false);
  });

  it("queues an updater install when auto-download is on and the channel has a package", async () => {
    const install = vi.fn(async ({ channel }) => ({
      queued: true,
      via: "updater",
      channel,
    }));
    setInstallTransport(install);
    const result = await queueUpdateInstall({
      autoDownload: true,
      channel: "preview",
    });
    expect(result.queued).toBe(true);
    expect(result.via).toBe("updater");
    expect(install).toHaveBeenCalledWith({ channel: "preview" });
    expect(JSON.stringify(result)).not.toMatch(/商店/);
  });
});
