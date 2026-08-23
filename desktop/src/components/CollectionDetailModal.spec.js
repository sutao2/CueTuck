import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import CollectionDetailModal from "./CollectionDetailModal.vue";

describe("CollectionDetailModal", () => {
  it("opens a sparse grid with real images and placeholders", () => {
    const w = mount(CollectionDetailModal, {
      props: {
        collection: {
          id: "col-1",
          title: "人像灵感",
          cover_type: "grid",
          cover_json: JSON.stringify(["one.jpg", "two.jpg", "three.jpg"]),
        },
        members: [],
        prompts: [],
      },
    });
    const cells = w.findAll('[data-testid="cover-grid"] i');
    expect(cells).toHaveLength(9);
    expect(w.findAll('[data-testid="cover-grid"] img')).toHaveLength(3);
    expect(cells.filter((cell) => cell.classes().includes("filled"))).toHaveLength(3);
    expect(w.get('[data-testid="collection-detail"]').exists()).toBe(true);
  });
});
