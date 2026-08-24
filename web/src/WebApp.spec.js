import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it } from "vitest";
import { resetMemoryLibrary } from "./memoryLibrary.js";
import WebApp from "./WebApp.vue";

describe("WebApp", () => {
  beforeEach(() => {
    resetMemoryLibrary();
  });

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

  it("creates a memory prompt and lists it without claiming desktop sync", async () => {
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("测试");
    await w.get('[data-testid="prompt-content"]').setValue("正文");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    expect(w.get('[data-testid="prompt-list"]').text()).toContain("测试");
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
    expect(w.text()).not.toContain("已与桌面库同步");
  });

  it("opens a memory prompt and shows its body", async () => {
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("测试");
    await w.get('[data-testid="prompt-content"]').setValue("你好");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-row"]').trigger("click");
    expect(w.get('[data-testid="prompt-body"]').text()).toContain("你好");
  });
});
