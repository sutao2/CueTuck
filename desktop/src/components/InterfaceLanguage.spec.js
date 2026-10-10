import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import Settings from './SettingsModal.vue';
import Detail from './LocalPromptDetail.vue';
import Launcher from '../LauncherApp.vue';
import { applyInterfaceLanguage, interfaceLanguage } from '../platform/interfaceLanguage.js';
import * as library from '../platform/library.js';

let wrappers = [];
const mountView = (view, options) => { const w = mount(view, options); wrappers.push(w); return w; };
beforeEach(() => { library.resetMemoryLibrary(); applyInterfaceLanguage('zh'); });
afterEach(() => { wrappers.forEach(w => w.unmount()); wrappers = []; vi.restoreAllMocks(); applyInterfaceLanguage('zh'); document.body.innerHTML = ''; });
async function chooseLanguage(w, value) {
  await w.get('[data-settings-page="appearance"]').trigger('click');
  await w.get('[data-testid=ui-language]').trigger('click');
  document.querySelector(`[role=option][id$="-${value === 'en' ? 1 : 0}"]`).click();
  await flushPromises();
}
it('switches all ten settings pages immediately and restores the saved language on remount', async () => {
  const w = mountView(Settings, { attachTo: document.body });
  await flushPromises();
  await chooseLanguage(w, 'en');
  expect(await library.getLocalSetting('ui_language')).toBe('en');
  expect(w.get('[data-testid=settings-feedback]').text()).toContain('Saved');
  for (const page of ['general','account','shortcuts','sync','models','data','network','appearance','privacy','updates']) {
    await w.get(`[data-settings-page="${page}"]`).trigger('click');
    expect(w.get('.settings-content').text(), page).not.toMatch(/[\u4e00-\u9fff]/);
  }
  w.unmount(); wrappers = [];
  const restored = mountView(Settings, { attachTo: document.body });
  await flushPromises();
  expect(restored.get('h3').text()).toBe('General');
  await chooseLanguage(restored, 'zh');
  await restored.get('[data-settings-page="general"]').trigger('click');
  expect(restored.get('h3').text()).toBe('常规');
});
it('keeps the effective language when saving fails', async () => {
  const w = mountView(Settings, { attachTo: document.body });
  await flushPromises();
  vi.spyOn(library, 'setLocalSetting').mockRejectedValue(Error('disk unavailable'));
  await chooseLanguage(w, 'en');
  expect(interfaceLanguage.value).toBe('zh');
  expect(w.text()).toContain('保存失败');
  expect(w.get('[data-testid=ui-language]').text()).toBe('中文');
});
it('updates open detail actions without translating user titles, models or content', async () => {
  const w = mountView(Detail, { props: { prompt: { id:'user-prompt',title:'设置',content:'请保留我的中文正文',model:'我的模型',author:'我的作者' } } });
  await flushPromises();
  applyInterfaceLanguage('en'); await flushPromises();
  expect(w.text()).toContain('Edit');
  expect(w.text()).toContain('Use prompt');
  expect(w.text()).toContain('设置');
  expect(w.text()).toContain('我的模型');
  expect(w.text()).toContain('请保留我的中文正文');
});
it('loads the saved language in the independent launcher', async () => {
  await library.setLocalSetting('ui_language','en');
  const w = mountView(Launcher);
  await flushPromises();
  expect(w.get('input').attributes('placeholder')).toBe('Search prompts, or enter a task…');
  await w.get('input').setValue('我的任务');
  expect(w.text()).toContain('Create prompt');
  expect(w.text()).toContain('Refine with AI');
  expect(w.text()).not.toContain('创建提示词');
});
