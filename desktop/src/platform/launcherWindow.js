export const LAUNCHER_LABEL = "launcher";
export const LAUNCHER_WIDTH = 680;
export const LAUNCHER_HEIGHTS = {
  collapsed: 80,
  expanded: 500,
  fill: 520,
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

export function openLauncherWindow() {
  if (window.__TAURI_INTERNALS__) {
    return import("@tauri-apps/api/core").then(({ invoke }) => invoke("show_launcher"));
  }
  const popup = window.open("/launcher.html", LAUNCHER_LABEL, "width=680,height=420");
  if (!popup) {
    window.location.assign("/launcher.html");
  }
}
