import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import SettingsModal from './SettingsModal.vue';
import * as library from '../platform/library.js';
import { DEFAULT_LAUNCHER_PREFERENCES, LAUNCHER_PREFERENCES_KEY, readLauncherPreferences } from '../platform/launcherPreferences.js';

enableAutoUnmount(afterEach);
beforeEach(library.resetMemoryLibrary);
afterEach(() => vi.restoreAllMocks());

it('saves each launcher option, reloads it and resets only launcher preferences', async () => {
  await library.setLocalSetting('launcher_shortcut', 'Control+Shift+P');
  await library.setLocalSetting('close_launcher_after_use', '0');
  const w = mount(SettingsModal); await flushPromises();
  for (const [key, value] of Object.entries({ size: 'large', position: 'center', fontSize: '16', resultLimit: '50' })) {
    await w.get(`[data-testid="launcher-${key}"]`).setValue(value); await flushPromises();
  }
  expect(await readLauncherPreferences()).toEqual({ size: 'large', position: 'center', fontSize: 16, resultLimit: 50 });
  expect(w.get('[data-testid="settings-feedback"]').text()).toContain('下次唤起');
  w.unmount();
  const reopened = mount(SettingsModal); await flushPromises();
  expect(reopened.get('[data-testid="launcher-size"]').element.value).toBe('large');
  expect(reopened.get('[data-testid="launcher-fontSize"]').element.value).toBe('16');
  await reopened.get('[data-testid="reset-launcher-preferences"]').trigger('click'); await flushPromises();
  expect(await readLauncherPreferences()).toEqual(DEFAULT_LAUNCHER_PREFERENCES);
  expect(await library.getLocalSetting('launcher_shortcut')).toBe('Control+Shift+P');
  expect(await library.getLocalSetting('close_launcher_after_use')).toBe('0');
});

it('retains saved preferences on write failure and blocks repeat writes while pending', async () => {
  const w = mount(SettingsModal); await flushPromises();
  let reject;
  const write = vi.spyOn(library, 'setLocalSetting').mockImplementation(() => new Promise((_, fail) => { reject = fail; }));
  await w.get('[data-testid="launcher-size"]').setValue('large');
  expect(w.get('fieldset').element.disabled).toBe(true);
  expect(w.get('.settings-return').element.disabled).toBe(true);
  expect(write).toHaveBeenCalledTimes(1);
  reject(new Error('磁盘只读')); await flushPromises();
  expect(w.get('[data-testid="launcher-size"]').element.value).toBe('compact');
  expect(w.get('[data-testid="settings-feedback"]').text()).toContain('保存失败：磁盘只读');
  expect(await library.getLocalSetting(LAUNCHER_PREFERENCES_KEY)).toBe('');
});

it('finds launcher controls through settings search without adding duplicate pages', async () => {
  const w = mount(SettingsModal); await flushPromises();
  await w.get('[aria-label="搜索设置"]').setValue('字号');
  expect(w.findAll('[data-settings-page]')).toHaveLength(1);
  expect(w.get('[data-settings-page]').attributes('data-settings-page')).toBe('general');
});
