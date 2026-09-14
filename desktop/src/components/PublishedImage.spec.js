import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
const { load }=vi.hoisted(()=>({load:vi.fn()}));
vi.mock('../platform/publishedImageCache.js',()=>({publishedImage:load}));
import PublishedImage from './PublishedImage.vue';
import SquareDetail from './SquareDetailModal.vue';
const file={id:'image',mime:'image/gif',name:'animation.gif',data:btoa('GIF89a')};
let wrapper;
beforeEach(()=>load.mockReset()); afterEach(()=>wrapper?.unmount());
it('shows a published animated image and opens the original in the viewer',async()=>{
 load.mockResolvedValue(file); wrapper=mount(PublishedImage,{props:{itemId:'item',file}});await flushPromises();
 expect(wrapper.get('img').attributes('src')).toBe('data:image/gif;base64,'+file.data);
 await wrapper.get('button').trigger('click');expect(wrapper.findComponent({name:'ImageViewer'}).exists()).toBe(true);
});
it('offers retry and ignores a result from a previous item',async()=>{
 let finish;load.mockImplementationOnce(()=>new Promise(resolve=>finish=resolve)).mockRejectedValueOnce(new Error('offline')).mockResolvedValue(file);
 wrapper=mount(PublishedImage,{props:{itemId:'first',file}});await flushPromises();
 await wrapper.setProps({itemId:'second'});await flushPromises();finish({...file,data:btoa('old')});await flushPromises();
 expect(wrapper.text()).toContain('点击重试');expect(wrapper.find('img').exists()).toBe(false);
 await wrapper.get('button').trigger('click');await flushPromises();expect(wrapper.get('img').attributes('src')).toContain(file.data);
});
it('renders approved image attachments in the detail gallery even without external references',async()=>{
 load.mockResolvedValue(file);wrapper=mount(SquareDetail,{props:{item:{id:'item',kind:'prompt',title:'Published',content:'body',asset_refs:[file,{id:'document',name:'notes.txt',mime:'text/plain'}]}},global:{stubs:{PromptLanguage:true,ReportPanel:true,PublishedAttachments:true}}});await flushPromises();
 expect(wrapper.get('[aria-label="发布图片"]').findAllComponents(PublishedImage)).toHaveLength(1);
});
