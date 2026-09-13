import { mount, flushPromises } from '@vue/test-utils';
import { h } from 'vue';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WindowedPromptGrid from './WindowedPromptGrid.vue';
let w, root, frames, resize, style, slot, columnStyle, headerTop;
beforeEach(() => {
  root = document.createElement('main'); document.body.append(root);
  frames = new Map(); let id = 0; columnStyle = '300px 300px 300px'; headerTop = 0;
  vi.stubGlobal('requestAnimationFrame', callback => { frames.set(++id, callback); return id; });
  vi.stubGlobal('cancelAnimationFrame', id => frames.delete(id));
  vi.stubGlobal('ResizeObserver', class { constructor(callback) { resize = callback; } observe() {} unobserve() {} disconnect() {} });
  vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(960);
  vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(720);
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function() { return { top: this === root ? 0 : headerTop - root.scrollTop }; });
  style = vi.fn(() => ({ gridTemplateColumns: columnStyle, rowGap: '14px' }));
  vi.stubGlobal('getComputedStyle', style);
  slot = vi.fn(({ item, cardHeight }) => h('article', { class: 'prompt-card', style: { height: `${cardHeight}px` } }, item.title));
});
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); vi.unstubAllGlobals(); document.body.innerHTML = ''; });
async function frame() { const callbacks = [...frames.values()]; frames.clear(); callbacks.forEach(callback => callback()); await flushPromises(); }
async function setup(enabled = true) {
  w = mount(WindowedPromptGrid, { attachTo: root, props: { enabled, scrollRoot: root, items: Array.from({ length: 480 }, (_, id) => ({ id, kind: 'prompt', title: `卡片 ${id}` })) }, slots: { default: slot } });
  await flushPromises(); await frame(); await frame(); slot.mockClear(); style.mockClear();
}
async function scroll(top) { root.scrollTop = top; root.dispatchEvent(new Event('scroll')); await frame(); }
it('does not render cards or reread column styles for scrolls within the same visible rows', async () => {
  await setup();
  for (let y = 1; y <= 20; y++) await scroll(y);
  expect(slot).not.toHaveBeenCalled();
  expect(style).not.toHaveBeenCalled();
  await scroll(3000);
  expect(slot).toHaveBeenCalled();
  expect(w.findAll('.prompt-card').length).toBeLessThan(30);
});
it('remeasures resized columns, updates changed data, and respects a shifted grid origin', async () => {
  await setup(); await scroll(3000);
  columnStyle = '450px 450px'; resize(); await frame();
  expect(style).toHaveBeenCalledTimes(1);
  expect(w.findAll('.prompt-card').length).toBeLessThan(20);
  await w.setProps({ list: true }); await frame();
  expect(w.findAll('.prompt-card')[0].attributes('style')).toContain('116px');
  headerTop = 1000; await scroll(3001);
  expect(w.findAll('.prompt-card')[0].text()).toBe('卡片 13');
  await w.setProps({ items: [{ id: 'new', kind: 'prompt', title: '新结果' }] }); await frame();
  expect(w.findAll('.prompt-card')).toHaveLength(1);
  expect(w.text()).toBe('新结果');
});
it('does not schedule scroll work when disabled and cancels pending work on unmount', async () => {
  await setup(false); await scroll(100); expect(frames.size).toBe(0);
  root.dispatchEvent(new Event('scroll')); expect(frames.size).toBe(0);
  await w.setProps({ enabled: true }); await frame();
  root.dispatchEvent(new Event('scroll')); expect(frames.size).toBe(1);
  w.unmount(); w = null; expect(frames.size).toBe(0);
});

it('sizes mixed local cards independently and clears packing when switching to rows', async () => {
  await setup(false);
  const cards = w.findAll('.prompt-card');
  resize([{target:cards[0].element,borderBoxSize:[{blockSize:160}]},{target:cards[1].element,borderBoxSize:[{blockSize:390}]}]);
  expect(cards[0].element.style.gridRowEnd).toBe('span 174');
  expect(cards[1].element.style.gridRowEnd).toBe('span 404');
  await w.setProps({list:true}); await flushPromises();
  expect(cards[0].element.style.gridRowEnd).toBe('');
  expect(w.classes()).not.toContain('content-grid');
});
