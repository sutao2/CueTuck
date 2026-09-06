import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it } from "vitest";
import SettingsModal from "./SettingsModal.vue";
import { resetMemoryLibrary } from "../platform/library.js";

beforeEach(resetMemoryLibrary);

it("keeps one navigation title and marks the current settings page", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  expect(w.get('[role="dialog"]').attributes("aria-labelledby")).toBe("settings-title");
  expect(w.get(".settings-nav h2").text()).toBe("设置");
  expect(w.find(".modal-header").exists()).toBe(false);
  for (const button of w.findAll("[data-settings-page]")) {
    await button.trigger("click");
    expect(w.findAll('[aria-current="page"]')).toHaveLength(1);
    expect(button.attributes("aria-current")).toBe("page");
    expect(w.findAll(".settings-content h3")).toHaveLength(1);
    expect(w.get(".settings-content h3").text()).toBe(button.text());
  }
  await w.get('[aria-label="关闭"]').trigger("click");
  expect(w.emitted("cancel")).toHaveLength(1);
});

it("places multi-field account forms in full-width blocks", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  await w.get('[data-settings-page="account"]').trigger("click");
  const blocks = w.findAll(".setting-block");
  expect(blocks).toHaveLength(2);
  expect(blocks[0].get('[data-testid="author-display-name"]').exists()).toBe(true);
  expect(blocks[1].get('[data-testid="billing-redeem-code"]').exists()).toBe(true);
});

it("retains unsaved model fields when replacing the scrolling page", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  await w.get('[data-settings-page="models"]').trigger("click");
  await w.get('[data-testid="custom-models"]').setValue("Local model");
  await w.get('[data-settings-page="general"]').trigger("click");
  await w.get('[data-settings-page="models"]').trigger("click");
  expect(w.get('[data-testid="custom-models"]').element.value).toBe("Local model");
  const children = w.get(".settings-content section").element.children;
  expect(children[children.length - 1].querySelector('[data-testid="save-models"]')).not.toBeNull();
});
