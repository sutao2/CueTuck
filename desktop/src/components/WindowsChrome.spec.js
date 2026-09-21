import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import WorkbenchShell from './WorkbenchShell.vue';
import { resetMemoryLibrary } from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setCatalogTransport, setSquarePageTransport } from '../platform/square.js';
let wrapper;
beforeEach(() => {
  resetMemoryLibrary(); resetMemorySession(); resetSquare();
  setCatalogTransport(async () => ({ categories: [], models: [] }));
  setSquarePageTransport(async () => ({ items: [], total: 0, next_offset: null }));
});
afterEach(() => { wrapper?.unmount(); document.body.innerHTML = ''; });
it('keeps Windows controls outside inert content and visible in search and settings', async () => {
  wrapper = mount(WorkbenchShell, { props: { host: 'windows' }, attachTo: document.body }); await flushPromises();
  const controls = document.querySelector('.window-controls');
  expect(controls).not.toBeNull(); expect(controls.closest('.app-shell')).toBeNull();
  await wrapper.get('[data-testid="titlebar-search"]').trigger('click');
  expect(wrapper.get('[data-region="titlebar"]').attributes('inert')).toBeDefined();
  expect(controls.closest('[inert]')).toBeNull();
  await wrapper.get('[data-testid="global-search-input"]').trigger('keydown', {key: 'Escape'}); await flushPromises();
  await wrapper.get('[data-testid="open-settings"]').trigger('click'); await flushPromises();
  expect(wrapper.get('[data-testid="settings-page"]').exists()).toBe(true);
  expect(controls.isConnected).toBe(true); expect(controls.closest('[inert]')).toBeNull();
});
it.each(['macos', 'linux'])('retains native window controls on %s', async host => {
  wrapper = mount(WorkbenchShell, { props: { host }, attachTo: document.body }); await flushPromises();
  expect(document.querySelector('.window-controls')).toBeNull();
});
