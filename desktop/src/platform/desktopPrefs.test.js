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

  it("persists launch at login and tray on windows", async () => {
    const saved = {};
    await saveDesktopPref(DESKTOP_PREF_KEYS.launchAtLogin, true, "windows", async (key, value) => {
      saved[key] = value;
    });
    await saveDesktopPref(DESKTOP_PREF_KEYS.minimizeToTray, true, "windows", async (key, value) => {
      saved[key] = value;
    });
    expect(saved.launch_at_login).toBe("1");
    expect(saved.minimize_to_tray).toBe("1");
    expect(JSON.stringify(saved)).not.toMatch(/NSIS 已验证/);
  });

  it("does not claim launch at login on linux until that slice lands", async () => {
    const saved = {};
    await expect(
      saveDesktopPref(DESKTOP_PREF_KEYS.launchAtLogin, true, "linux", async (key, value) => {
        saved[key] = value;
      }),
    ).rejects.toThrow("尚未验证");
    expect(saved.launch_at_login).toBeUndefined();
  });
});
