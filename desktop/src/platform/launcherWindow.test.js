import { describe, it, expect, vi } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { LAUNCHER_LABEL, LAUNCHER_WIDTH, launcherHeightFor, openLauncherWindow } from "./launcherWindow.js";

describe("launcher window", () => {
  it("uses the independent window label", () => {
    expect(LAUNCHER_LABEL).toBe("launcher");
  });

  it("keeps search and fill equally compact", () => {
    expect(LAUNCHER_WIDTH).toBe(620);
    expect(launcherHeightFor("collapsed")).toBe(64);
    expect(launcherHeightFor("expanded")).toBe(420);
    expect(launcherHeightFor("fill")).toBe(420);
    expect(launcherHeightFor("fill")).toBe(launcherHeightFor("expanded"));
  });

  it("keeps native config and browser popup sizes aligned", async () => {
    const config = JSON.parse(readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), "../../src-tauri/tauri.conf.json"), "utf8"));
    const launcher = config.app.windows.find((window) => window.label === LAUNCHER_LABEL);
    expect(launcher.width).toBe(LAUNCHER_WIDTH);
    expect(launcher.height).toBe(launcherHeightFor("collapsed"));
    const popup = vi.spyOn(window, "open").mockReturnValue({});
    try {
      await openLauncherWindow();
      expect(popup).toHaveBeenCalledWith("/launcher.html", LAUNCHER_LABEL, `width=${LAUNCHER_WIDTH},height=${launcherHeightFor("expanded")}`);
    } finally { popup.mockRestore(); }
  });

  it("preserves the native top-left during resize and positions only on show", () => {
    const source = readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), "../../src-tauri/src/commands/launcher.rs"), "utf8");
    const resize = source.split("fn resize_launcher_window(")[1].split("fn show_launcher_window(")[0];
    const show = source.split("fn show_launcher_window(")[1].split("#[tauri::command]")[0];
    expect(resize).not.toContain(".center()");
    expect(resize).not.toContain(".set_position(launcher_show_position(");
    expect(resize).toMatch(/let position = window\.outer_position\(\)/);
    expect(resize.indexOf(".outer_position()")).toBeLessThan(resize.indexOf(".set_size("));
    expect(resize.indexOf(".set_size(")).toBeLessThan(resize.indexOf(".set_position(position)"));
    expect(resize).toContain(".set_position(position)");
    expect(show).toContain(".set_position(launcher_show_position(monitor.work_area(), monitor.scale_factor(), &preferences))");
    expect(show).toContain('guard.current_layout()');
    expect(show.indexOf("resize_launcher_window(app, layout)")).toBeLessThan(show.indexOf(".set_position("));
    expect(show.indexOf(".set_position(")).toBeLessThan(show.indexOf(".show()"));
  });
});
