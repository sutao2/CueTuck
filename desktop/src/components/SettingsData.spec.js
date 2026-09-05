import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it } from "vitest";
import SettingsModal from "./SettingsModal.vue";
import { listLocalPrompts, resetMemoryLibrary } from "../platform/library.js";

beforeEach(resetMemoryLibrary);

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
