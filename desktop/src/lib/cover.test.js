import { describe, expect, it } from "vitest";
import { coverSlots, parseCoverUrls, serializeCoverUrls } from "./cover.js";

describe("cover refs", () => {
  it("keeps three grid refs and pads missing slots", () => {
    const json = serializeCoverUrls("grid", ["one.jpg", "two.jpg", "three.jpg"]);
    expect(parseCoverUrls(json)).toEqual(["one.jpg", "two.jpg", "three.jpg"]);
    const slots = coverSlots(json, 9);
    expect(slots.filter(Boolean)).toHaveLength(3);
    expect(slots).toHaveLength(9);
  });

  it("treats broken cover json as empty slots", () => {
    expect(parseCoverUrls("{not-json")).toEqual([]);
    expect(coverSlots("{not-json", 9).every((slot) => slot === "")).toBe(true);
  });
});
