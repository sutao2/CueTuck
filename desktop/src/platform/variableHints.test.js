import { describe, expect, it } from "vitest";
import { hintForVariable } from "./variableHints.js";

describe("hintForVariable", () => {
  it("returns a local example for a known name and nothing for unknown", () => {
    expect(hintForVariable("城市")).toMatch(/京都/);
    expect(hintForVariable("未知变量")).toBe("");
  });
});
