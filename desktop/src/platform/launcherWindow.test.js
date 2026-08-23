import { describe, it, expect } from "vitest";
import { LAUNCHER_LABEL, launcherHeightFor } from "./launcherWindow.js";

describe("launcher window", () => {
  it("uses the independent window label", () => {
    expect(LAUNCHER_LABEL).toBe("launcher");
  });

  it("sizes the palette like the old independent window", () => {
    expect(launcherHeightFor("collapsed")).toBe(80);
    expect(launcherHeightFor("expanded")).toBe(500);
    expect(launcherHeightFor("fill")).toBe(520);
  });
});
