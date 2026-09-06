import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, afterEach, expect, it, vi } from "vitest";
import SettingsModal from "./SettingsModal.vue";
import WorkbenchShell from "./WorkbenchShell.vue";
import * as library from "../platform/library.js";

let w;
beforeEach(() => { library.resetMemoryLibrary(); });
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); delete document.body.dataset.density; });
async function settings(page) {
  w = mount(SettingsModal);
  await flushPromises();
  await w.get(`[data-settings-page="${page}"]`).trigger("click");
}

it("rolls back an immediate switch when persistence fails", async () => {
  await library.setLocalSetting("square_access", "1");
  await settings("network");
  vi.spyOn(library, "setLocalSetting").mockRejectedValueOnce(new Error("disk full"));
  await w.get('[data-testid="square-access"]').setValue(false);
  await flushPromises();
  expect(w.get('[data-testid="square-access"]').element.checked).toBe(true);
  expect(await library.getLocalSetting("square_access")).toBe("1");
  expect(w.get('[data-testid="settings-feedback"]').text()).toContain("保存失败");
});

it("keeps model drafts when cancelling exit and closes after a successful save", async () => {
  await settings("models");
  await w.get('[data-testid="custom-models"]').setValue("Local model");
  await w.get('[aria-label="关闭"]').trigger("click");
  expect(w.get('[role="alertdialog"]').text()).toContain("未保存");
  await w.get('[data-testid="cancel-settings-action"]').trigger("click");
  expect(w.emitted("cancel")).toBeUndefined();
  expect(w.get('[data-testid="custom-models"]').element.value).toBe("Local model");
  await w.get('[data-testid="save-models"]').trigger("click");
  await flushPromises();
  expect(w.get('[data-testid="settings-feedback"]').text()).toContain("模型偏好已保存");
  await w.get('[aria-label="关闭"]').trigger("click");
  expect(w.emitted("cancel")).toHaveLength(1);
});

it("restores failed appearance choices without applying them", async () => {
  await settings("appearance");
  vi.spyOn(library, "setLocalSetting").mockRejectedValue(new Error("locked"));
  await w.get('[data-testid="density"]').setValue("compact");
  await flushPromises();
  expect(w.get('[data-testid="density"]').element.value).toBe("comfortable");
  expect(document.body.dataset.density).not.toBe("compact");
  await w.get('[data-testid="theme-select"]').setValue("dark");
  await flushPromises();
  expect(w.get('[data-testid="theme-select"]').element.value).toBe("light");
  expect(w.emitted("theme")).toBeUndefined();
});

it("blocks repeat saves and close while persistence is pending", async () => {
  await settings("models");
  let resolve;
  const pending = new Promise(done => { resolve = done; });
  const save = vi.spyOn(library, "setLocalSetting").mockImplementationOnce(() => pending);
  await w.get('[data-testid="save-models"]').trigger("click");
  await w.get('[aria-label="关闭"]').trigger("click");
  expect(w.emitted("cancel")).toBeUndefined();
  expect(w.get("fieldset").element.disabled).toBe(true);
  expect(save).toHaveBeenCalledTimes(1);
  resolve();
  await flushPromises();
  expect(w.get("fieldset").element.disabled).toBe(false);
});

it("reports partial model saves as failures and allows retry", async () => {
  await settings("models");
  await w.get('[data-testid="custom-models"]').setValue("Draft");
  vi.spyOn(library, "setLocalSetting").mockRejectedValueOnce(new Error("locked"));
  await w.get('[data-testid="save-models"]').trigger("click");
  await flushPromises();
  expect(w.get('[data-testid="settings-feedback"]').text()).toContain("保存失败");
  expect(w.get('[data-testid="custom-models"]').element.value).toBe("Draft");
  await w.get('[data-testid="save-models"]').trigger("click");
  await flushPromises();
  expect(await library.getLocalSetting("custom_models")).toBe("Draft");
});

it("requires confirmation before clearing history and never deletes prompts", async () => {
  const row = await library.createLocalPrompt({ title: "Keep", content: "body" });
  await library.recordLocalPromptUse(row.id);
  await settings("privacy");
  await w.get('[data-testid="clear-use-history"]').trigger("click");
  await w.get('[data-testid="cancel-settings-action"]').trigger("click");
  expect((await library.listLocalPrompts())[0].use_count).toBe(1);
  await w.get('[data-testid="clear-use-history"]').trigger("click");
  await w.get('[data-testid="confirm-settings-action"]').trigger("click");
  await flushPromises();
  expect((await library.listLocalPrompts())[0]).toMatchObject({ content: "body", use_count: 0 });
});

it("does not restore a database until the exact operation is confirmed", async () => {
  await settings("data");
  const restore = vi.spyOn(library, "restoreLocalLibrary").mockResolvedValue();
  await w.get('input[placeholder="/path/to/promptark.sqlite"]').setValue("/tmp/example.sqlite");
  const action = () => w.findAll('button').find(button => button.text() === '恢复库文件');
  await action().trigger("click");
  expect(w.get('[role="alertdialog"]').text()).toContain("/tmp/example.sqlite");
  await w.get('[data-testid="cancel-settings-action"]').trigger("click");
  expect(restore).not.toHaveBeenCalled();
  await action().trigger("click");
  await w.get('[data-testid="confirm-settings-action"]').trigger("click");
  await flushPromises();
  expect(restore).toHaveBeenCalledExactlyOnceWith("/tmp/example.sqlite");
});

it("loads persisted density and supports sidebar and settings shortcuts", async () => {
  await library.setLocalSetting("density", "compact");
  w = mount(WorkbenchShell, { props: { host: "macos" } });
  await flushPromises();
  expect(document.body.dataset.density).toBe("compact");
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'b', metaKey: true }));
  await flushPromises();
  expect(w.get('.app-shell').classes()).toContain('sidebar-collapsed');
  await w.get('[data-testid="toggle-sidebar"]').trigger('click');
  expect(w.get('.app-shell').classes()).not.toContain('sidebar-collapsed');
  window.dispatchEvent(new KeyboardEvent('keydown', { key: ',', metaKey: true }));
  await flushPromises();
  expect(w.get('[data-testid="settings-modal"]').exists()).toBe(true);
});
