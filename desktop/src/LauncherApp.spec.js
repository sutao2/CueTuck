import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it } from "vitest";
import LauncherApp from "./LauncherApp.vue";
import { createLocalPrompt, resetMemoryLibrary } from "./platform/library.js";
import { resetSquare, setSquareTransport } from "./platform/square.js";

describe("LauncherApp", () => {
  beforeEach(() => {
    resetMemoryLibrary();
    resetSquare();
  });

  it("uses mac chrome on macos", () => {
    const w = mount(LauncherApp, { props: { host: "macos" } });
    expect(w.get('[data-testid="launcher-chrome"]').classes()).toContain("host-mac");
  });

  it("hides results on empty query", async () => {
    await createLocalPrompt({ title: "官网生成器", content: "写官网" });
    const w = mount(LauncherApp);
    await flushPromises();
    expect(w.find('[role="listbox"]').exists()).toBe(false);
    expect(w.get('[data-testid="launcher-chrome"]').classes()).toContain("is-collapsed");
  });

  it("does not request admin APIs while searching locally", async () => {
    const urls = [];
    const originalFetch = globalThis.fetch;
    globalThis.fetch = async (input) => {
      urls.push(String(input));
      throw new Error("launcher must not fetch");
    };
    try {
      await createLocalPrompt({ title: "官网生成器", content: "写官网" });
      const w = mount(LauncherApp);
      await flushPromises();
      await w.get("input").setValue("官网");
      await flushPromises();
      expect(urls.some((url) => url.includes("/v1/admin"))).toBe(false);
      expect(w.get('[role="listbox"]').text()).toContain("官网生成器");
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("does not request square while searching locally", async () => {
    let called = false;
    setSquareTransport(async () => {
      called = true;
      return [];
    });
    await createLocalPrompt({ title: "官网生成器", content: "写官网" });
    const w = mount(LauncherApp);
    await flushPromises();
    await w.get("input").setValue("官网");
    await flushPromises();
    expect(called).toBe(false);
    expect(w.get('[role="listbox"]').text()).toContain("官网生成器");
  });

  it("lists a local title hit", async () => {
    await createLocalPrompt({ title: "官网生成器", content: "写官网" });
    const w = mount(LauncherApp);
    await flushPromises();
    await w.get("input").setValue("官网");
    await flushPromises();
    expect(w.get('[role="listbox"]').text()).toContain("官网生成器");
    expect(w.get(".launcher-foot").text()).toContain("选择");
    expect(w.get('[data-testid="launcher-chrome"]').classes()).not.toContain("is-collapsed");
  });

  it("opens fill step when Enter hits a variable prompt", async () => {
    await createLocalPrompt({ title: "问候", content: "你好 {{姓名}}" });
    const w = mount(LauncherApp);
    await flushPromises();
    await w.get("input").setValue("问候");
    await flushPromises();
    await w.get("input").trigger("keydown", { key: "Enter" });
    expect(w.text()).toContain("姓名");
    expect(w.get(".preview").text()).toContain("{{姓名}}");
  });
});
