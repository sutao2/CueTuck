import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import CollectionCover from './CollectionCover.vue';
import ImageViewer from './ImageViewer.vue';

it.each(['single', 'grid'])('opens the selected local %s cover and clears stale previews', async type => {
  const urls = ['data:image/png;base64,YQ==', 'data:image/png;base64,Yg=='];
  const w = mount(CollectionCover, { props: { type, json: JSON.stringify(urls) }, global: { stubs: { ImageViewer: true } } });
  const buttons = w.findAll('.cover-preview');
  expect(buttons).toHaveLength(type === 'single' ? 1 : 2);
  await buttons.at(-1).trigger('click');
  expect(w.getComponent(ImageViewer).props('src')).toBe(urls[type === 'single' ? 0 : 1]);
  w.getComponent(ImageViewer).vm.$emit('close');
  await w.vm.$nextTick();
  expect(w.findComponent(ImageViewer).exists()).toBe(false);
  await buttons[0].trigger('click');
  await w.setProps({ json: '[]' });
  expect(w.findComponent(ImageViewer).exists()).toBe(false);
  expect(w.findAll('.cover-preview')).toHaveLength(0);
  w.unmount();
});
it('keeps local card covers as navigation thumbnails without preview buttons', () => {
  const w = mount(CollectionCover, { props: { type: 'grid', json: '["image.png"]', variant: 'card' } });
  expect(w.findAll('img')).toHaveLength(1);
  expect(w.findAll('.cover-preview')).toHaveLength(0);
  expect(w.findAll('i')).toHaveLength(9);
  w.unmount();
});
