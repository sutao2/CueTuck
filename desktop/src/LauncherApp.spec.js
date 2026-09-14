import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import * as library from "./platform/library.js";
import * as launcherWindow from "./platform/launcherWindow.js";
import LauncherApp from "./LauncherApp.vue";
import { createLocalPrompt, resetMemoryLibrary } from "./platform/library.js";
import { resetSquare, setSquareTransport } from "./platform/square.js";

enableAutoUnmount(afterEach);

describe("LauncherApp", () => {
  afterEach(() => { vi.restoreAllMocks(); delete navigator.clipboard; });
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

  it("only resizes when repeatedly typing, clearing and entering or leaving fill", async () => {
    const resize = vi.spyOn(launcherWindow, "resizeLauncherWindow").mockResolvedValue();
    const show = vi.spyOn(launcherWindow, "openLauncherWindow");
    await createLocalPrompt({ title: "问候", content: "你好 {{姓名}}" });
    const w = mount(LauncherApp);
    await flushPromises();
    for (let cycle = 0; cycle < 3; cycle += 1) {
      await w.get("input").setValue("问候");
      await flushPromises();
      await w.get("input").setValue("");
      await flushPromises();
    }
    await w.get("input").setValue("问候");
    await flushPromises();
    await w.get("input").trigger("keydown", { key: "Enter" });
    await flushPromises();
    await w.findAll("button").find((button) => button.text() === "返回").trigger("click");
    await flushPromises();
    expect(resize.mock.calls.map(([layout]) => layout)).toEqual([
      "collapsed", "expanded", "collapsed", "expanded", "collapsed", "expanded", "collapsed", "expanded", "fill", "expanded",
    ]);
    expect(show).not.toHaveBeenCalled();
    w.unmount();
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

  it("keeps keyboard selection when layout puts an action under a stationary pointer", async () => {
    await createLocalPrompt({ title: "官网生成器", content: "写官网" });
    const w = mount(LauncherApp);
    await flushPromises();
    await w.get("input").setValue("官网");
    await flushPromises();
    const rows = w.findAll('[role="option"]');
    await rows[2].trigger('mouseenter');
    await rows[2].trigger('mousemove', { movementX: 0, movementY: 0 });
    expect(rows[0].attributes('aria-selected')).toBe('true');
    await rows[2].trigger('mousemove', { movementX: 1, movementY: 0 });
    expect(rows[2].attributes('aria-selected')).toBe('true');
    await w.get('input').trigger('keydown', { key: 'ArrowUp' });
    await rows[2].trigger('mouseenter');
    expect(rows[1].attributes('aria-selected')).toBe('true');
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

  it("previews plain prompts and keeps clipboard failures out of usage history", async () => {
    await createLocalPrompt({ title: "普通正文", content: "未复制的文本" });
    const writeText = vi.fn().mockRejectedValueOnce(new Error("denied")).mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
    const w = mount(LauncherApp);
    await flushPromises();
    await w.get("input").setValue("普通");
    await flushPromises();
    await w.get("input").trigger("keydown", { key: "Enter" });
    expect(w.get(".preview").text()).toBe("未复制的文本");
    expect(writeText).not.toHaveBeenCalled();
    await w.findAll("button").find((button) => button.text() === "复制").trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="launcher-feedback"]').text()).toContain("复制失败");
    expect((await library.listLocalPrompts())[0].use_count).toBe(0);
    expect(await library.getLocalSetting("last_rendered_prompt")).toBeFalsy();
    await w.findAll("button").find((button) => button.text() === "复制").trigger("click");
    await flushPromises();
    expect((await library.listLocalPrompts())[0].use_count).toBe(1);
    expect(await library.getLocalSetting("last_rendered_prompt")).toBe("未复制的文本");
  });

  it("does not let old searches overwrite new results or reopen a cleared query", async () => {
    const pending = {};
    vi.spyOn(library, "listLocalPrompts").mockImplementation(({ query }) => new Promise((resolve) => { pending[query] = resolve; }));
    const w = mount(LauncherApp);
    await flushPromises();
    await w.get("input").setValue("old");
    await w.get("input").setValue("new");
    pending.new([{ id: "new", title: "新结果", content: "正文" }]);
    await flushPromises();
    pending.old([{ id: "old", title: "旧结果", content: "正文" }]);
    await flushPromises();
    expect(w.text()).toContain("新结果");
    expect(w.text()).not.toContain("旧结果");
    await w.get("input").setValue("late");
    await w.get("input").setValue("");
    pending.late([{ id: "late", title: "迟到结果", content: "正文" }]);
    await flushPromises();
    expect(w.find('[role="listbox"]').exists()).toBe(false);
  });
});
