import { describe, expect, it } from "vitest";
import { parseModelNames } from "./modelCatalog.js";

describe("parseModelNames", () => {
  it("splits catalog, custom, and item models without duplicates", () => {
    expect(
      parseModelNames("Flux\nGPT-5", "Flux, Midjourney", [{ model: "GPT-5" }, { model: "SDXL" }, "SDXL"]),
    ).toEqual(["Flux", "GPT-5", "Midjourney", "SDXL"]);
  });

  it("ignores blank names", () => {
    expect(parseModelNames("\n , ;", ["", "  "], [{ model: "" }])).toEqual([]);
  });
});
