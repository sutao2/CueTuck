import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import LocalPromptCover from './LocalPromptCover.vue';
import { firstPromptImage } from '../platform/assets.js';
vi.mock('../platform/assets.js',()=>({firstPromptImage:vi.fn(),assetUrl:asset=>`data:${asset.mime};base64,${asset.data}`}));
let w,intersect,disconnect;
const picture={mime:'image/png',data:'iVBORw0KGgo='};
beforeEach(()=>{
  firstPromptImage.mockReset();disconnect=vi.fn();
  vi.stubGlobal('IntersectionObserver',class {constructor(callback){intersect=callback;}observe(){}disconnect(){disconnect();}});
});
afterEach(()=>{w?.unmount();vi.unstubAllGlobals();document.body.innerHTML='';});
it('only reads visible covers, releases offscreen bytes and reloads after editing',async()=>{
  firstPromptImage.mockResolvedValue(picture);
  w=mount(LocalPromptCover,{props:{promptId:'one',title:'示例',revision:'1'}});
  expect(firstPromptImage).not.toHaveBeenCalled();
  intersect([{isIntersecting:true}]);await flushPromises();expect(firstPromptImage).toHaveBeenCalledWith('one');expect(w.find('img').exists()).toBe(true);
  intersect([{isIntersecting:false}]);await flushPromises();expect(w.find('img').exists()).toBe(false);
  intersect([{isIntersecting:true}]);await flushPromises();expect(firstPromptImage).toHaveBeenCalledTimes(2);
  await w.setProps({revision:'2'});await flushPromises();expect(firstPromptImage).toHaveBeenCalledTimes(3);
  w.unmount();w=null;expect(disconnect).toHaveBeenCalled();
});
it('ignores stale loads when a card changes, and contains missing or failed images',async()=>{
  let finish;firstPromptImage.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;})).mockResolvedValueOnce(null);
  w=mount(LocalPromptCover,{props:{promptId:'one'}});intersect([{isIntersecting:true}]);await flushPromises();
  await w.setProps({promptId:'two'});await flushPromises();finish(picture);await flushPromises();
  expect(w.find('img').exists()).toBe(false);expect(w.text()).toContain('图片暂不可用');
  firstPromptImage.mockRejectedValueOnce(Error('read failed'));await w.setProps({revision:'3'});await flushPromises();expect(w.get('button').attributes('disabled')).toBeDefined();
  firstPromptImage.mockResolvedValueOnce(picture);await w.setProps({revision:'4'});await flushPromises();await w.get('img').trigger('error');expect(w.find('img').exists()).toBe(false);
});
it('opens full preview without opening the editor and Escape returns focus to the cover',async()=>{
  firstPromptImage.mockResolvedValue(picture);const parentClick=vi.fn();
  const container=document.createElement('article');document.body.append(container);container.addEventListener('click',parentClick);
  w=mount(LocalPromptCover,{attachTo:container,props:{promptId:'one',title:'示例'}});
  intersect([{isIntersecting:true}]);await flushPromises();const button=w.get('button');button.element.focus();await button.trigger('click');await flushPromises();
  expect(parentClick).not.toHaveBeenCalled();expect(document.querySelector('.image-viewer')).not.toBeNull();
  document.querySelector('.image-viewer').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));await flushPromises();
  expect(document.querySelector('.image-viewer')).toBeNull();expect(document.activeElement).toBe(button.element);
});
