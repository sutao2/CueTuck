import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import WindowsWindowControls from './WindowsWindowControls.vue';
const api = vi.hoisted(() => ({ minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn(), isMaximized: vi.fn(), onResized: vi.fn() }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => api }));
let wrapper, resized, stop;
beforeEach(() => {
  window.__TAURI_INTERNALS__ = {};
  vi.resetAllMocks();
  stop = vi.fn();
  api.isMaximized.mockResolvedValue(false);
  api.onResized.mockImplementation(async callback => { resized = callback; return stop; });
});
afterEach(() => { wrapper?.unmount(); delete window.__TAURI_INTERNALS__; });
async function start() { wrapper = mount(WindowsWindowControls); await flushPromises(); }
describe('Windows window controls', () => {
  it('uses native minimize and close requests without bypassing the close-to-tray policy', async () => {
    await start();
    await wrapper.get('[aria-label="最小化"]').trigger('click');
    await wrapper.get('[aria-label="关闭窗口"]').trigger('click');
    expect(api.minimize).toHaveBeenCalledOnce();
    expect(api.close).toHaveBeenCalledOnce();
    for (const button of wrapper.findAll('button')) expect(button.attributes('data-tauri-drag-region')).toBeUndefined();
  });
  it('updates maximize/restore from the native state including external resize events', async () => {
    await start();
    api.toggleMaximize.mockImplementation(async () => { api.isMaximized.mockResolvedValue(true); });
    await wrapper.get('[aria-label="最大化"]').trigger('click'); await flushPromises();
    expect(wrapper.get('[aria-label="还原窗口"]').exists()).toBe(true);
    api.isMaximized.mockResolvedValue(false); resized(); await flushPromises();
    expect(wrapper.get('[aria-label="最大化"]').exists()).toBe(true);
  });
  it('ignores stale maximize reads and removes its resize listener', async () => {
    await start();
    let older;
    api.isMaximized.mockImplementationOnce(() => new Promise(resolve => { older = resolve; })).mockResolvedValue(true);
    resized(); resized(); await flushPromises(); older(false); await flushPromises();
    expect(wrapper.get('[aria-label="还原窗口"]').exists()).toBe(true);
    wrapper.unmount(); expect(stop).toHaveBeenCalledOnce(); wrapper = null;
  });
  it('cleans up a listener that resolves after unmount', async () => {
    let ready;
    api.onResized.mockImplementation(() => new Promise(resolve => { ready = resolve; }));
    wrapper = mount(WindowsWindowControls); wrapper.unmount(); wrapper = null;
    ready(stop); await flushPromises(); expect(stop).toHaveBeenCalledOnce();
  });
  it('shows a failed operation and permits retry', async () => {
    await start(); api.minimize.mockRejectedValueOnce(new Error('denied'));
    await wrapper.get('[aria-label="最小化"]').trigger('click'); await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain('请重试');
    await wrapper.get('[aria-label="最小化"]').trigger('click'); await flushPromises();
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  });
  it('does not request native APIs in a browser preview', async () => {
    delete window.__TAURI_INTERNALS__; await start();
    await wrapper.get('[aria-label="关闭窗口"]').trigger('click');
    expect(api.onResized).not.toHaveBeenCalled(); expect(api.close).not.toHaveBeenCalled();
  });
});
