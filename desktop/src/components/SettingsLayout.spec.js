import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it } from "vitest";
import SettingsModal from "./SettingsModal.vue";
import { resetMemoryLibrary, getLocalSetting } from "../platform/library.js";
import WorkbenchShell from './WorkbenchShell.vue';

beforeEach(resetMemoryLibrary);

it("keeps one navigation title and marks the current settings page", async () => {
  const w = mount(SettingsModal);
  await flushPromises();
  expect(w.find('[role="dialog"]').exists()).toBe(false);
  expect(w.find('.modal-backdrop').exists()).toBe(false);
  expect(w.get('[data-testid="settings-page"]').attributes('aria-label')).toBe('设置');
  expect(w.get(".settings-nav h2").text()).toBe("设置");
  expect(w.find(".modal-header").exists()).toBe(false);
  for (const button of w.findAll("[data-settings-page]")) {
    await button.trigger("click");
    expect(w.findAll('[aria-current="page"]')).toHaveLength(1);
    expect(button.attributes("aria-current")).toBe("page");
    expect(w.findAll(".settings-content h3")).toHaveLength(1);
    expect(w.get(".settings-content h3").text()).toBe(button.text());
  }
  await w.get('[aria-label="返回应用"]').trigger("click");
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
  for (const selector of ['author-display-name', 'author-bio', 'billing-redeem-code']) {
    expect(w.get(`[data-testid="${selector}"]`).element.closest('label').textContent.trim()).not.toBe('');
  }
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

it('searches settings without losing drafts and confirms returning to the app', async () => {
  const w = mount(SettingsModal, { attachTo: document.body }); await flushPromises();
  expect(w.get('.settings-body').attributes('inert')).toBeUndefined();
  await w.get('[data-settings-page="models"]').trigger('click');
  await w.get('[data-testid="custom-models"]').setValue('Unsaved model');
  const search = w.get('[aria-label="搜索设置"]');
  await search.setValue('备份');
  expect(w.findAll('[data-settings-page]')).toHaveLength(1);
  expect(w.get('[data-settings-page]').attributes('data-settings-page')).toBe('data');
  expect(w.get('[data-testid="custom-models"]').element.value).toBe('Unsaved model');
  await search.setValue('no-such-setting'); expect(w.text()).toContain('没有匹配的设置');
  await search.trigger('keydown', { key: 'Escape', isComposing:true });
  expect(search.element.value).toBe('no-such-setting');
  await search.trigger('keydown', { key: 'Escape' });
  expect(w.findAll('[data-settings-page]')).toHaveLength(10);
  await w.get('.settings-return').trigger('click');
  expect(w.get('[role="alertdialog"]').exists()).toBe(true);
  expect(w.get('.settings-body').attributes('inert')).toBe('');
  await w.get('[data-testid="cancel-settings-action"]').trigger('click');
  expect(w.get('.settings-body').attributes('inert')).toBeUndefined();
  expect(w.get('[data-testid="custom-models"]').element.value).toBe('Unsaved model');
  w.unmount();
});

it('theme previews save real theme preferences', async () => {
  const w = mount(SettingsModal); await flushPromises();
  await w.get('[data-settings-page="appearance"]').trigger('click');
  await w.get('[data-theme-choice="dark"]').trigger('click'); await flushPromises();
  expect(await getLocalSetting('theme')).toBe('dark');
  expect(w.get('[data-theme-choice="dark"]').attributes('aria-pressed')).toBe('true');
  expect(w.emitted('theme').at(-1)).toEqual(['dark']); w.unmount();
});

it('replaces the workbench visually and restores its search and entry focus on return', async () => {
  const w = mount(WorkbenchShell, {attachTo:document.body}); await flushPromises();
  await w.get('.inline-search input').setValue('keep query');
  const opener = w.get('[data-testid="open-settings"]'); opener.element.focus(); await opener.trigger('click'); await flushPromises();
  expect(w.get('.workspace').isVisible()).toBe(false);
  expect(w.get('.titlebar').isVisible()).toBe(false);
  expect(w.get('.statusbar').isVisible()).toBe(false);
  expect(w.get('[data-testid="settings-page"]').isVisible()).toBe(true);
  expect(w.find('[role="dialog"]').exists()).toBe(false);
  await w.get('.settings-return').trigger('click'); await flushPromises();
  expect(w.get('.workspace').isVisible()).toBe(true);
  expect(w.get('.inline-search input').element.value).toBe('keep query');
  expect(document.activeElement).toBe(opener.element); w.unmount();
});
