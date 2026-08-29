import { describe, expect, it } from "vitest";
import { uiText } from "./uiStrings.js";

describe("uiText", () => {
  it("switches chrome labels between Chinese and English", () => {
    expect(uiText("zh", "square")).toBe("提示词广场");
    expect(uiText("en", "square")).toBe("Prompt Square");
    expect(uiText("en", "settingsGeneral")).toBe("General");
  });
});
