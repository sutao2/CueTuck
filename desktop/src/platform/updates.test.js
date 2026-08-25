import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { checkForUpdates, resetUpdates, setUpdateTransport } from "./updates.js";

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
});
