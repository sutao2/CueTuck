import { describe, expect, it } from "vitest";
import {
  detectHost,
  formatShortcutLabel,
  trafficLightInsetPx,
} from "./windowChrome.js";

describe("window chrome", () => {
  it("treats MacIntel as macos", () => {
    expect(detectHost({ platform: "MacIntel", userAgent: "Mozilla/5.0" })).toBe("macos");
  });

  it("gives traffic-light inset and glyph shortcut on macos", () => {
    expect(trafficLightInsetPx("macos")).toBe(78);
    expect(formatShortcutLabel("Control+Space", "macos")).toBe("⌃Space");
  });

  it("keeps Windows inset and Ctrl Space label", () => {
    expect(trafficLightInsetPx("windows")).toBe(0);
    expect(formatShortcutLabel("Control+Space", "windows")).toBe("Ctrl Space");
  });
});
