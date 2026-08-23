import { describe, expect, it } from "vitest";
import { saveDesktopPref, DESKTOP_PREF_KEYS } from "./desktopPrefs.js";

describe("desktopPrefs", () => {
  it("persists launch at login on macos", async () => {
    const saved = {};
    await saveDesktopPref(DESKTOP_PREF_KEYS.launchAtLogin, true, "macos", async (key, value) => {
      saved[key] = value;
    });
    expect(saved.launch_at_login).toBe("1");
  });

  it("does not claim launch at login on windows", async () => {
    const saved = {};
    await expect(
      saveDesktopPref(DESKTOP_PREF_KEYS.launchAtLogin, true, "windows", async (key, value) => {
        saved[key] = value;
      }),
    ).rejects.toThrow("尚未验证");
    expect(saved.launch_at_login).toBeUndefined();
  });
});
