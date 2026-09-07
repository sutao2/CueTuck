import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { LAUNCHER_LABEL, launcherHeightFor } from "./launcherWindow.js";

describe("launcher window", () => {
  it("uses the independent window label", () => {
    expect(LAUNCHER_LABEL).toBe("launcher");
  });

  it("sizes the palette like the old independent window", () => {
    expect(launcherHeightFor("collapsed")).toBe(80);
    expect(launcherHeightFor("expanded")).toBe(500);
    expect(launcherHeightFor("fill")).toBe(420);
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
    expect(show).toContain(".set_position(launcher_show_position(monitor.work_area(), monitor.scale_factor()))");
    expect(show.indexOf("resize_launcher_window(app, \"collapsed\")")).toBeLessThan(show.indexOf(".set_position("));
    expect(show.indexOf(".set_position(")).toBeLessThan(show.indexOf(".show()"));
  });
});
