import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import WebApp from "./WebApp.vue";

describe("WebApp", () => {
  it("keeps local space when the sidebar is collapsed", async () => {
    const w = mount(WebApp);
    expect(w.get('[data-space="local"]').exists()).toBe(true);
    expect(w.get('[data-testid="sidebar"]').classes()).not.toContain("is-collapsed");
    await w.get('[data-testid="toggle-sidebar"]').trigger("click");
    expect(w.get('[data-testid="sidebar"]').classes()).toContain("is-collapsed");
    expect(w.get('[data-space="local"]').exists()).toBe(true);
    expect(w.get('[data-region="content"]').exists()).toBe(true);
  });

  it("does not claim the browser library is synced to desktop sqlite", () => {
    const w = mount(WebApp);
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
    expect(w.text()).not.toContain("已与桌面库同步");
  });
});
