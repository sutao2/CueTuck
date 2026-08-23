import { setLocalSetting } from "./library.js";

export const DEFAULT_LAUNCHER_SHORTCUT = "Control+Space";

export async function registerLauncherShortcut(
  combo = DEFAULT_LAUNCHER_SHORTCUT,
  {
    register,
    unregisterAll,
    persist = (value) => setLocalSetting("launcher_shortcut", value),
  } = {},
) {
  const plugin = register && unregisterAll
    ? { register, unregisterAll }
    : await loadShortcutPlugin();
  await plugin.unregisterAll();
  try {
    await plugin.register(combo, async (event) => {
      if (event?.state && event.state !== "Pressed") return;
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("toggle_launcher");
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(message.includes("冲突") ? message : `快捷键冲突：${message}`);
  }
  await persist(combo);
  return combo;
}

async function loadShortcutPlugin() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    throw new Error("当前环境不能注册全局快捷键");
  }
  const gs = await import("@tauri-apps/plugin-global-shortcut");
  return {
    register: gs.register,
    unregisterAll: gs.unregisterAll,
  };
}
