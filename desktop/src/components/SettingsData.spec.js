import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it } from "vitest";
import SettingsModal from "./SettingsModal.vue";
import { getLocalSetting, listLocalPrompts, resetMemoryLibrary } from "../platform/library.js";

beforeEach(resetMemoryLibrary);

it("refuses browser auto-backup instead of persisting an ineffective enabled flag", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  await w.get('[data-settings-page="data"]').trigger("click");
  await w.get('[data-testid="auto-backup"]').setValue(true);
  await flushPromises();
  expect(w.get('[data-testid="auto-backup"]').element.checked).toBe(false);
  expect(w.get('[data-testid="backup-error"]').text()).toContain("仅桌面窗口支持");
  expect(await getLocalSetting("auto_backup")).not.toBe("1");
});

it("invalidates changed previews and disables repeat import after success", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  await w.get('[data-settings-page="data"]').trigger("click");
  const text = w.get("textarea:not([readonly])");
  const button = (label) => w.findAll("button").find((item) => item.text() === label);
  await text.setValue('{"prompts":[{"title":"第一份","content":"正文"}]}');
  await button("预览").trigger("click");
  await flushPromises();
  expect(button("确认导入").element.disabled).toBe(false);
  await text.setValue('{"prompts":[{"title":"第二份","content":"正文"}]}');
  expect(button("确认导入").element.disabled).toBe(true);
  await button("预览").trigger("click");
  await flushPromises();
  await button("确认导入").trigger("click");
  await flushPromises();
  expect((await listLocalPrompts()).map((row) => row.title)).toEqual(["第二份"]);
  expect(button("确认导入").element.disabled).toBe(true);
  expect(w.text()).toContain("导入完成");
});

it("shows invalid JSON errors without writing any prompt", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  await w.get('[data-settings-page="data"]').trigger("click");
  await w.get("textarea:not([readonly])").setValue("broken JSON");
  await w.findAll("button").find((item) => item.text() === "预览").trigger("click");
  await flushPromises();
  expect(w.get('[data-testid="backup-error"]').text()).toContain("预览失败");
  expect(await listLocalPrompts()).toHaveLength(0);
});
