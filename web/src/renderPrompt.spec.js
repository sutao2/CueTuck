import { describe, expect, it } from "vitest";
import { extractVariables, renderPrompt } from "./renderPrompt.js";

describe("renderPrompt", () => {
  it("dedupes repeated variables", () => {
    expect(extractVariables("为 {{产品}} 写介绍，再次强调 {{产品}}")).toEqual(["产品"]);
  });

  it("keeps unfilled placeholders", () => {
    expect(renderPrompt("给 {{受众}} 看", {})).toBe("给 {{受众}} 看");
  });
});
