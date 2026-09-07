import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const read = (path) => readFileSync(fileURLToPath(new URL(path, import.meta.url)));

describe("application icon assets", () => {
  it.each([["32x32.png", 32], ["128x128.png", 128], ["icon.png", 512]])("ships a transparent RGBA %s", (name, size) => {
    const png = read(`../../src-tauri/icons/${name}`);
    expect(png.subarray(0, 8).toString("hex")).toBe("89504e470d0a1a0a");
    expect(png.readUInt32BE(16)).toBe(size);
    expect(png.readUInt32BE(20)).toBe(size);
    expect(png[25]).toBe(6);
  });

  it("uses the same generated icon for launcher and desktop", () => {
    expect(read("../assets/app-icon.png")).toEqual(read("../../src-tauri/icons/128x128.png"));
    const launcher = read("../LauncherApp.vue").toString();
    expect(launcher.match(/class="brand-mark" :src="appIcon"/g)).toHaveLength(2);
  });

  it("includes macOS ICNS and multi-resolution Windows ICO in the bundle", () => {
    const config = JSON.parse(read("../../src-tauri/tauri.conf.json"));
    expect(config.bundle.icon).toContain("icons/icon.icns");
    expect(config.bundle.icon).toContain("icons/icon.ico");
    const icns = read("../../src-tauri/icons/icon.icns");
    expect(icns.subarray(0, 4).toString()).toBe("icns");
    expect(icns.readUInt32BE(4)).toBe(icns.length);
    const ico = read("../../src-tauri/icons/icon.ico");
    expect(ico.readUInt16LE(2)).toBe(1);
    expect(ico.readUInt16LE(4)).toBeGreaterThan(1);
  });
});
