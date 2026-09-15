import { mount, flushPromises, enableAutoUnmount } from '@vue/test-utils';
import { beforeEach, afterEach, it, expect } from 'vitest';
import Workbench from './WorkbenchShell.vue';
import Detail from './SquareDetailModal.vue';
import { resetMemoryLibrary } from '../platform/library.js';
enableAutoUnmount(afterEach);
const originalWidth=window.innerWidth;
const resize=async width=>{window.innerWidth=width;window.dispatchEvent(new Event('resize'));await flushPromises();};
beforeEach(()=>{resetMemoryLibrary();window.innerWidth=1440;});
afterEach(()=>{window.innerWidth=originalWidth;});
it('temporarily collapses a narrow sidebar without overwriting wide preferences',async()=>{
 const w=mount(Workbench,{attachTo:document.body});await flushPromises();expect(w.get('[data-testid="toggle-sidebar"]').attributes('aria-expanded') === 'true').toBe(true);
 await resize(820);expect(w.get('[data-testid="toggle-sidebar"]').attributes('aria-expanded') === 'true').toBe(false);
 await w.get('[data-testid="toggle-sidebar"]').trigger('click');expect(w.get('[data-testid="toggle-sidebar"]').attributes('aria-expanded') === 'true').toBe(true);
 await w.get('[data-testid="toggle-sidebar"]').trigger('click');await resize(1440);
 expect(w.get('[data-testid="toggle-sidebar"]').attributes('aria-expanded') === 'true').toBe(true);
 await w.get('[data-testid="toggle-sidebar"]').trigger('click');await flushPromises();
 await resize(820);await w.get('[data-testid="toggle-sidebar"]').trigger('click');await resize(1440);
 expect(w.get('[data-testid="toggle-sidebar"]').attributes('aria-expanded') === 'true').toBe(false);
});
it('renders body before media and retains the single use action and attribution',()=>{
 const item={id:'p',kind:'prompt',title:'示例',content:'正文',reference:{images:['https://cms-assets.youmind.com/example.png'],author:'来源作者'},publisher:{display_name:'发布昵称'}};
 const w=mount(Detail,{props:{item},global:{stubs:{PromptLanguage:true,ReportPanel:true,PublishedAttachments:true}}});
 const text=w.get('[data-testid="square-detail-content"]').element,media=w.get('.detail-media').element;
 expect(text.compareDocumentPosition(media)&Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
 expect(w.findAll('[data-testid="square-detail-use"]')).toHaveLength(1);expect(w.text()).toContain('发布昵称');expect(w.text()).toContain('来源作者');
});
it('does not reserve an empty media column for a text-only prompt',()=>{
 const w=mount(Detail,{props:{item:{id:'p',kind:'prompt',title:'纯文字',content:'正文'}},global:{stubs:{PromptLanguage:true,ReportPanel:true,PublishedAttachments:true}}});
 expect(w.find('.detail-media').exists()).toBe(false);expect(w.get('.detail-reading-grid').classes()).not.toContain('has-media');
});
