import { mount } from '@vue/test-utils';
import { afterEach, it, expect } from 'vitest';
import SquareDetail from './SquareDetailModal.vue';
let w;
afterEach(() => w?.unmount());
function render() { w = mount(SquareDetail, { props: { item: { id:'one',title:'自然光群像',content:'真实正文' } } }); }
it('shows original reference images with attribution without replacing prompt content', async () => {
  render(); await w.setProps({item:{id:'imported',title:'海报',content:'原始正文',reference:{repository:'YouMind',author:'Original author',url:'https://youmind.com/prompts?id=1',license:'CC BY 4.0',license_url:'https://creativecommons.org/licenses/by/4.0/',images:['https://cms-assets.youmind.com/media/a.jpg']}}});
  expect(w.get('.gallery-label').text()).toContain('来源参考图');
  expect(w.get('figcaption').text()).toContain('Original author');
  expect(w.get('.reference-credit').text()).toContain('CC BY 4.0');
  expect(w.find('.detail-example-button').exists()).toBe(false);
  await w.get('.gallery-stage img').trigger('error');
  expect(w.get('[data-testid="square-detail-content"]').text()).toBe('原始正文');
  expect(w.get('[data-testid="square-detail-download"]').attributes('disabled')).toBeUndefined();
});
it('does not offer unrelated example images in real content', async () => {
  render(); expect(w.find('img').exists()).toBe(false);
  expect(w.find('.detail-example-button').exists()).toBe(false);
  await w.get('[data-testid="square-detail-download"]').trigger('click');
  expect(w.emitted('download')[0]).toEqual([]);
  expect(w.get('[data-testid="square-detail-content"]').text()).toBe('真实正文');
  expect(w.find('[aria-modal]').exists()).toBe(false);
});
it('switches previews, retries failures and resets when the item changes', async () => {
  render(); await w.setProps({ item: { id: 'one', title: '有图', content: '正文', reference: { images: ['https://cms-assets.youmind.com/media/one.jpg', 'https://cms-assets.youmind.com/media/two.jpg'] } } });
  await w.get('[aria-label="预览图片 2"]').trigger('click');
  expect(w.get('.gallery-stage img').attributes('src')).toContain('two.jpg');
  await w.get('.gallery-stage img').trigger('error');
  expect(w.get('.gallery-fallback').text()).toContain('暂时无法加载');
  await w.get('.gallery-fallback button').trigger('click');
  expect(w.find('.gallery-stage img').exists()).toBe(true);
  await w.setProps({item:{id:'two',title:'另一个条目',content:'其他正文'}});
  expect(w.find('.detail-gallery').exists()).toBe(false);
});
