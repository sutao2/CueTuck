export const LAUNCHER_LABEL = "launcher";
export const LAUNCHER_WIDTH = 680;
export const LAUNCHER_HEIGHTS = {
  collapsed: 80,
  expanded: 500,
  fill: 420,
};

export function launcherHeightFor(layout) {
  return LAUNCHER_HEIGHTS[layout] ?? LAUNCHER_HEIGHTS.expanded;
}

export async function resizeLauncherWindow(layout) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("resize_launcher", { layout });
}

export async function startDraggingLauncher() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().startDragging();
}

export async function launcherCommand(command) {
  if (!window.__TAURI_INTERNALS__) return;
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command);
}

export async function listenLauncherLifecycle(onShown, onHidden, onFeedback) {
  if (!window.__TAURI_INTERNALS__) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  const shown = await listen("launcher-shown", onShown);
  let hidden = () => {};
  try {
    hidden = await listen("launcher-hidden", onHidden);
    const feedback = await listen("launcher-feedback", onFeedback);
    return () => { shown(); hidden(); feedback(); };
  } catch (error) { shown(); hidden(); throw error; }
}

export function openLauncherWindow() {
  if (window.__TAURI_INTERNALS__) {
    return import("@tauri-apps/api/core").then(({ invoke }) => invoke("show_launcher"));
  }
  const popup = window.open("/launcher.html", LAUNCHER_LABEL, "width=680,height=420");
  if (!popup) {
    window.location.assign("/launcher.html");
  }
}
