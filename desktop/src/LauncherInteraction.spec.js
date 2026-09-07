import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import LauncherApp from "./LauncherApp.vue";
import * as library from "./platform/library.js";
import * as windows from "./platform/launcherWindow.js";
import * as paste from "./platform/paste.js";
import { renderPrompt } from "./lib/renderPrompt.js";

enableAutoUnmount(afterEach);
let writeText;
beforeEach(async () => {
  library.resetMemoryLibrary();
  await library.setLocalSetting("close_launcher_after_use", "0");
  writeText = vi.fn().mockResolvedValue();
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
});
afterEach(() => { vi.restoreAllMocks(); delete navigator.clipboard; delete window.__TAURI_INTERNALS__; document.body.innerHTML = ""; });

async function open(content = "{{姓名}} / {{任务}} / {{姓名}}") {
  await library.createLocalPrompt({ title: "测试", content });
  const w = mount(LauncherApp, { attachTo: document.body });
  await flushPromises();
  await w.get("input").setValue("测试");
  await flushPromises();
  await w.get("input").trigger("keydown", { key: "Enter" });
  await flushPromises();
  return w;
}
const button = (w, text) => w.findAll("button").find((b) => b.text() === text);

it("focuses variables, advances with Enter and copies only on the last field", async () => {
  const w = await open();
  const fields = w.findAll("textarea");
  expect(fields).toHaveLength(2);
  expect(document.activeElement).toBe(fields[0].element);
  await fields[0].setValue("小明");
  await fields[0].trigger("keydown", { key: "Enter" });
  expect(document.activeElement).toBe(fields[1].element);
  expect(writeText).not.toHaveBeenCalled();
  await fields[1].setValue("第一行\n第二行 $& {{原样}}");
  await fields[1].trigger("keydown", { key: "Enter" });
  await flushPromises();
  expect(writeText).toHaveBeenCalledExactlyOnceWith("小明 / 第一行\n第二行 $& {{原样}} / 小明");
  expect(w.text()).toContain("已复制");
});

it("ignores composition, Shift+Enter and repeated Enter; supports modified Enter anywhere", async () => {
  const w = await open();
  const field = w.get("textarea");
  for (const modifiers of [{ isComposing: true }, { keyCode: 229 }, { shiftKey: true }, { repeat: true }]) {
    await field.trigger("keydown", { key: "Enter", ...modifiers });
  }
  await field.trigger("keydown", { key: "Escape", isComposing: true });
  expect(w.find("textarea").exists()).toBe(true);
  expect(writeText).not.toHaveBeenCalled();
  await field.trigger("keydown", { key: "Enter", metaKey: true });
  await flushPromises();
  expect(writeText).toHaveBeenCalledExactlyOnceWith("{{姓名}} / {{任务}} / {{姓名}}");
});

it("focuses plain-preview copy and restores the search query, selection and focus on return", async () => {
  const w = await open("纯文本");
  expect(w.find("form").exists()).toBe(false);
  expect(document.activeElement).toBe(button(w, "复制").element);
  await button(w, "返回").trigger("click");
  await flushPromises();
  expect(w.get("input").element.value).toBe("测试");
  expect(document.activeElement).toBe(w.get("input").element);
});

it("does not erase the active draft or duplicate writes while copying", async () => {
  let complete;
  writeText.mockImplementation(() => new Promise((resolve) => { complete = resolve; }));
  const w = await open();
  await button(w, "复制").trigger("click");
  await button(w, "返回").trigger("click");
  await w.get("main").trigger("keydown", { key: "Escape" });
  await w.get("main").trigger("keydown", { key: "Enter", ctrlKey: true });
  expect(w.find(".preview").exists()).toBe(true);
  expect(writeText).toHaveBeenCalledTimes(1);
  complete();
  await flushPromises();
  expect((await library.listLocalPrompts())[0].use_count).toBe(1);
});

it("keeps a copied draft when recording usage fails", async () => {
  const w = await open();
  vi.spyOn(library, "recordLocalPromptUse").mockRejectedValue(new Error("磁盘只读"));
  await button(w, "复制").trigger("click");
  await flushPromises();
  expect(w.text()).toContain("已复制");
  expect(w.text()).toContain("保存使用记录失败");
  expect(w.find(".preview").exists()).toBe(true);
});

it("retains paste failure details and the completed draft", async () => {
  vi.spyOn(paste, "copyThenPaste").mockResolvedValue({ ok: false, message: "已复制，未能粘贴：缺少辅助功能权限" });
  const w = await open();
  await button(w, "粘贴到原窗口").trigger("click");
  await flushPromises();
  expect(w.text()).toContain("缺少辅助功能权限");
  expect(w.find(".preview").exists()).toBe(true);
});

it("distinguishes loading and search errors from no matches", async () => {
  let reject;
  vi.spyOn(library, "listLocalPrompts").mockImplementation(() => new Promise((_, fail) => { reject = fail; }));
  const w = mount(LauncherApp);
  await w.get("input").setValue("测试");
  expect(w.text()).toContain("正在搜索");
  expect(w.text()).not.toContain("没有找到");
  reject(new Error("数据库不可用"));
  await flushPromises();
  expect(w.text()).toContain("搜索失败");
  expect(w.text()).not.toContain("没有找到");
});

it("does not fill prototype properties, and treats values as literal text", () => {
  expect(renderPrompt("{{constructor}} {{toString}} {{__proto__}}", {})).toBe("{{constructor}} {{toString}} {{__proto__}}");
  expect(renderPrompt("{{x}}", { x: "$& {{x}}\n" })).toBe("$& {{x}}\n");
});

it("updates preview for special variable names without Vue or object prototype collisions", async () => {
  const w = await open("{{__proto__}} {{constructor}} {{toString}} {{__v_isReactive}}");
  for (const field of w.findAll("textarea")) await field.setValue("literal");
  expect(w.get(".preview").text()).toBe("literal literal literal literal");
});

it("handles PageUp on a short list without a negative selection and mouse click opens the same preview", async () => {
  await library.createLocalPrompt({ title: "测试 A", content: "A" });
  await library.createLocalPrompt({ title: "测试 B", content: "B" });
  const w = mount(LauncherApp);
  await w.get("input").setValue("测试");
  await flushPromises();
  await w.get("input").trigger("keydown", { key: "PageUp" });
  expect(w.get("input").attributes("aria-activedescendant")).toBe("launcher-result-1");
  const row = w.get('[role="option"][aria-selected="true"]');
  const content = row.text().includes("测试 A") ? "A" : "B";
  await row.trigger("click");
  expect(w.get(".preview").text()).toBe(content);
});

it("copies directly, respects auto-close, and rejects empty content", async () => {
  const w = await open("纯文本");
  await button(w, "返回").trigger("click");
  await flushPromises();
  await w.get("input").trigger("keydown", { key: "Enter", ctrlKey: true });
  await flushPromises();
  expect(writeText).toHaveBeenCalledExactlyOnceWith("纯文本");
  expect(w.find(".preview").exists()).toBe(false);
  await library.setLocalSetting("close_launcher_after_use", "1");
  await w.get("input").trigger("keydown", { key: "Enter", metaKey: true });
  await flushPromises();
  expect(w.get("input").element.value).toBe("");
  vi.spyOn(library, "listLocalPrompts").mockResolvedValue([{ id: "empty", title: "空文本", content: " " }]);
  await w.get("input").setValue("空");
  await flushPromises();
  await w.get("input").trigger("keydown", { key: "Enter", ctrlKey: true });
  await flushPromises();
  expect(w.text()).toContain("内容为空");
  expect(writeText).toHaveBeenCalledTimes(2);
});

it("copies a single variable with Enter and preserves its draft if hiding fails", async () => {
  const w = await open("{{单项}}");
  await library.setLocalSetting("close_launcher_after_use", "1");
  vi.spyOn(windows, "launcherCommand").mockRejectedValue(new Error("窗口不可用"));
  await w.get("textarea").setValue("单项文本");
  await w.get("textarea").trigger("keydown", { key: "Enter" });
  await flushPromises();
  expect(writeText).toHaveBeenCalledExactlyOnceWith("单项文本");
  expect(w.get("textarea").element.value).toBe("单项文本");
  expect(w.text()).toContain("已复制；窗口恢复失败");
});

it("resets on native hide/show, restores focus/theme, and releases event listeners", async () => {
  let shown, hidden, notice;
  const cleanup = vi.fn();
  vi.spyOn(windows, "listenLauncherLifecycle").mockImplementation(async (...handlers) => {
    [shown, hidden, notice] = handlers;
    return cleanup;
  });
  const w = await open();
  await w.get("textarea").setValue("旧草稿");
  hidden();
  await library.setLocalSetting("theme", "dark");
  await shown();
  await flushPromises();
  expect(w.get("input").element.value).toBe("");
  expect(document.activeElement).toBe(w.get("input").element);
  expect(document.body.classList.contains("theme-dark")).toBe(true);
  notice({ payload: "没有最近使用的提示词" });
  await flushPromises();
  expect(w.text()).toContain("没有最近使用的提示词");
  expect(w.get(".launcher-stage").classes()).not.toContain("is-collapsed");
  w.unmount();
  expect(cleanup).toHaveBeenCalledTimes(1);
});

it("restores the current field when native focus arrives after the shown event", async () => {
  const w = await open();
  const field = w.findAll("textarea")[1];
  field.element.focus();
  field.element.blur();
  window.dispatchEvent(new Event("focus"));
  await flushPromises();
  expect(document.activeElement).toBe(field.element);
  await button(w, "返回").trigger("click");
  await flushPromises();
  w.get("input").element.blur();
  window.dispatchEvent(new Event("focus"));
  await flushPromises();
  expect(document.activeElement).toBe(w.get("input").element);
});

it("ignores native hide/blur while copying, then returns after paste when keep-open is enabled", async () => {
  let hidden, complete;
  vi.spyOn(windows, "listenLauncherLifecycle").mockImplementation(async (_, handler) => { hidden = handler; return () => {}; });
  const command = vi.spyOn(windows, "launcherCommand").mockResolvedValue(true);
  vi.spyOn(paste, "copyThenPaste").mockImplementation(() => new Promise((resolve) => { complete = resolve; }));
  const w = await open();
  await button(w, "粘贴到原窗口").trigger("click");
  window.__TAURI_INTERNALS__ = {};
  window.dispatchEvent(new Event("blur"));
  hidden();
  delete window.__TAURI_INTERNALS__;
  expect(w.find(".preview").exists()).toBe(true);
  expect(command).not.toHaveBeenCalled();
  complete({ ok: true });
  await flushPromises();
  expect(command).toHaveBeenCalledWith("resume_launcher");
  expect(w.text()).toContain("已复制并发送粘贴指令");
});

it("fills the focused variable from the original selection and preserves values on empty/error", async () => {
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue("Macintosh");
  vi.spyOn(library, "getLocalSetting").mockResolvedValue("0");
  vi.spyOn(library, "listLocalPrompts").mockResolvedValue([{ id: "x", title: "测试", content: "{{甲}} {{乙}}" }]);
  vi.spyOn(windows, "listenLauncherLifecycle").mockResolvedValue(() => {});
  vi.spyOn(windows, "resizeLauncherWindow").mockResolvedValue();
  const command = vi.spyOn(windows, "launcherCommand").mockResolvedValue("原选区");
  window.__TAURI_INTERNALS__ = {};
  const w = mount(LauncherApp, { attachTo: document.body });
  await w.get("input").setValue("测试");
  await flushPromises();
  await w.get("input").trigger("keydown", { key: "Enter" });
  await flushPromises();
  const fields = w.findAll("textarea");
  await fields[0].setValue("甲内容");
  fields[1].element.focus();
  await w.get('[data-testid="read-selected"]').trigger("click");
  await flushPromises();
  expect(w.get(".preview").text()).toBe("甲内容 原选区");
  command.mockResolvedValue("");
  await w.get('[data-testid="read-selected"]').trigger("click");
  await flushPromises();
  expect(fields[1].element.value).toBe("原选区");
  expect(w.text()).toContain("没有可读取的选区");
  command.mockRejectedValue(new Error("没有权限"));
  await w.get('[data-testid="read-selected"]').trigger("click");
  await flushPromises();
  expect(w.text()).toContain("没有权限");
  expect(fields[1].element.value).toBe("原选区");
  expect(document.activeElement).toBe(fields[1].element);
});
