export const DESKTOP_PREF_KEYS = {
  launchAtLogin: "launch_at_login",
  minimizeToTray: "minimize_to_tray",
  closeLauncherAfterUse: "close_launcher_after_use",
};

export function canApplyPref(key, host) {
  if (key === DESKTOP_PREF_KEYS.launchAtLogin || key === DESKTOP_PREF_KEYS.minimizeToTray) {
    return host === "macos" || host === "windows";
  }
  return true;
}

export async function saveDesktopPref(key, enabled, host, persist) {
  if (!canApplyPref(key, host)) {
    throw new Error("当前系统尚未验证该选项，不会声称已生效");
  }
  if (typeof window !== "undefined" && window.__TAURI_INTERNALS__) {
    const { invoke } = await import("@tauri-apps/api/core");
    if (key === DESKTOP_PREF_KEYS.launchAtLogin) {
      await invoke("apply_launch_at_login", { enabled });
    }
    if (key === DESKTOP_PREF_KEYS.minimizeToTray) {
      await invoke("apply_minimize_to_tray", { enabled });
    }
  }
  await persist(key, enabled ? "1" : "0");
  return enabled;
}

export function isPrefOn(value, defaultOn = false) {
  if (value === "1") return true;
  if (value === "0") return false;
  return defaultOn;
}
