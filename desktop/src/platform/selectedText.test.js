import { describe, expect, it } from "vitest";
import { supportsSelectedText } from "./selectedText.js";

describe("supportsSelectedText", () => {
  it("hides selected-text on windows", () => {
    expect(supportsSelectedText("windows")).toBe(false);
    expect(supportsSelectedText("macos")).toBe(true);
  });
});
