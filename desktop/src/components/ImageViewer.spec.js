import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, expect, it } from 'vitest';
import { nextTick } from 'vue';
import ImageViewer from './ImageViewer.vue';
import SquareDetail from './SquareDetailModal.vue';
import AttachmentPanel from './AttachmentPanel.vue';
let w;
afterEach(()=>{w?.unmount();document.body.innerHTML='';});
it('opens a local image, zooms it, closes only the viewer and restores keyboard focus',async()=>{
  w=mount(AttachmentPanel,{attachTo:document.body,props:{modelValue:[{id:crypto.randomUUID(),name:'picture.png',mime:'image/png',data:'iVBORw0KGgo='}]}});
  const button=w.get('.attachment-open');button.element.focus();await button.trigger('click');await flushPromises();
  const viewer=w.getComponent(ImageViewer), img=document.querySelector('.image-viewer img');
  Object.defineProperty(img,'naturalWidth',{value:1000});img.dispatchEvent(new Event('load'));await nextTick();
  document.querySelector('[aria-label="放大图片"]').click();await nextTick();expect(img.style.width).toBe('1250px');
  document.querySelector('.image-viewer').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));await flushPromises();
  expect(viewer.emitted('close')).toHaveLength(1);expect(document.querySelector('.image-viewer')).toBeNull();expect(document.activeElement).toBe(button.element);
});
it('opens square reference images without dismissing the detail page and offers explicit repair',async()=>{
  w=mount(SquareDetail,{attachTo:document.body,props:{downloaded:true,item:{id:'test',title:'Example',content:'body',reference:{images:['https://cms-assets.youmind.com/a.png']}}}});
  await w.get('[data-testid="complete-square-images"]').trigger('click');expect(w.emitted('complete-images')).toHaveLength(1);
  await w.get('.gallery-open').trigger('click');await flushPromises();expect(document.querySelector('[aria-modal="true"]')).not.toBeNull();
  document.querySelector('.image-viewer').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));await flushPromises();expect(w.emitted('cancel')).toBeUndefined();expect(w.find('.gallery-open').exists()).toBe(true);
});
it('reports load failure and disables zoom without loading documents',async()=>{
  w=mount(ImageViewer,{props:{src:'https://cms-assets.youmind.com/missing.png'}});document.querySelector('.image-viewer img').dispatchEvent(new Event('error'));await nextTick();
  expect(document.querySelector('[role="alert"]').textContent).toContain('无法加载');expect(document.querySelector('[aria-label="放大图片"]').disabled).toBe(true);
});
