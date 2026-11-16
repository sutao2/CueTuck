import {mount,flushPromises} from '@vue/test-utils';
import {expect,it,vi,afterEach} from 'vitest';
import QuickClips from './QuickClips.vue';
import ClipCategorySidebar from './ClipCategorySidebar.vue';
import SearchableSelect from './SearchableSelect.vue';
import Launcher from '../LauncherApp.vue';
import * as clips from '../platform/quickClips.js';
import * as library from '../platform/library.js';
let wrapper;
afterEach(()=>{wrapper?.unmount();vi.restoreAllMocks();vi.useRealTimers();});
it('does not fetch a saved password when searching, editing its name, or cancelling deletion',async()=>{
 const row={id:'saved',title:'Work',kind:'password',content:null};
 vi.spyOn(clips,'findClips').mockResolvedValue([row]);
 const request=vi.spyOn(clips,'clipRequest').mockResolvedValue(null);
 wrapper=mount(QuickClips);await flushPromises();
 const button=t=>wrapper.findAll('button').find(b=>b.text()===t);
 await wrapper.get('[aria-expanded=false][aria-label*="更多操作"]').trigger('click');
 await button('编辑').trigger('click');
 expect(wrapper.get('input[type=password]').element.value).toBe('');
 await wrapper.get('form input[autocomplete=off]').setValue('Other');
 await wrapper.get('form').trigger('submit');await flushPromises();
 expect(request.mock.calls).toEqual([[{action:'save',id:'saved',kind:'password',title:'Other',value:null,category_id:''}]]);
 await wrapper.get('[aria-expanded=false][aria-label*="更多操作"]').trigger('click');
 await button('删除').trigger('click');await button('取消').trigger('click');expect(request).toHaveBeenCalledTimes(1);
});
it('selecting a clip in the launcher copies by ID without updating recent prompt text',async()=>{
 library.resetMemoryLibrary();vi.useFakeTimers();
 await library.setLocalSetting('close_launcher_after_use','0');await library.setLocalSetting('last_rendered_prompt','previous prompt');
 vi.spyOn(clips,'findClips').mockResolvedValue([{id:'c1',kind:'password',title:'Work',content:null}]);
 const copy=vi.spyOn(clips,'copyClip').mockResolvedValue();
 wrapper=mount(Launcher);await flushPromises();await wrapper.get('input').setValue('Work');await vi.advanceTimersByTimeAsync(300);await flushPromises();
 expect(wrapper.text()).toContain('••••••••');expect(copy).not.toHaveBeenCalled();
 await wrapper.get('input').trigger('keydown',{key:'Enter'});await flushPromises();expect(copy).toHaveBeenCalledWith('c1');expect(await library.getLocalSetting('last_rendered_prompt')).toBe('previous prompt');
});
it('copies from global clip search and clears that query before switching to an online scope',async()=>{
 const {default:GlobalSearch}=await import('./GlobalSearch.vue');
 const square=await import('../platform/square.js');
 vi.useFakeTimers();window.__TAURI_INTERNALS__={};
 try {
  const remote=vi.spyOn(square,'listSquarePage');
  vi.spyOn(clips,'findClips').mockResolvedValue([{id:'c2',title:'Account',kind:'account',content:'work-name'}]);
  const copy=vi.spyOn(clips,'copyClip').mockResolvedValue();
  wrapper=mount(GlobalSearch,{props:{initialScope:'clips'}});await wrapper.get('input').setValue('work-name');await vi.advanceTimersByTimeAsync(300);
  await wrapper.get('[role=option]').trigger('click');await flushPromises();expect(copy).toHaveBeenCalledWith('c2');
  await wrapper.get('[data-search-scope=square]').trigger('click');await vi.advanceTimersByTimeAsync(300);
  expect(wrapper.get('input').element.value).toBe('');expect(remote).not.toHaveBeenCalled();
 }finally{delete window.__TAURI_INTERNALS__;}
});
it('reveals only on request and drops both visible and pending secrets on blur or timeout',async()=>{
 vi.useFakeTimers();
 vi.spyOn(clips,'findClips').mockResolvedValue([{id:'one',title:'Work',kind:'password',content:null}]);
 const get=vi.spyOn(clips,'clipRequest').mockResolvedValue('test-secret');
 const copy=vi.spyOn(clips,'copyClip').mockResolvedValue();
 wrapper=mount(QuickClips);await flushPromises();
 const reveal=()=>wrapper.findAll('button').find(b=>b.text()==='显示密码').trigger('click');
 expect(get).not.toHaveBeenCalled();
 await reveal();await flushPromises();expect(wrapper.text()).toContain('test-secret');
 expect(copy).not.toHaveBeenCalled();await vi.advanceTimersByTimeAsync(15000);expect(wrapper.text()).not.toContain('test-secret');
 let release;get.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));
 await reveal();window.dispatchEvent(new Event('blur'));release('late-secret');await flushPromises();
 expect(wrapper.text()).not.toContain('late-secret');
 await reveal();await flushPromises();window.dispatchEvent(new Event('blur'));await flushPromises();
 expect(wrapper.text()).not.toContain('test-secret');
 await wrapper.findAll('button').find(b=>b.text()==='复制').trigger('click');await flushPromises();
 expect(copy).toHaveBeenCalledWith('one');expect(wrapper.text()).not.toContain('test-secret');
});

it('combines the active sidebar group with type and keyword and presets new clips',async()=>{
 const find=vi.spyOn(clips,'findClips').mockResolvedValue([]);
 const summary={categories:[{id:'work',name:'工作',count:1}],total:2,uncategorized:1};
 vi.spyOn(clips,'clipCategories').mockResolvedValue(summary);
 const save=vi.spyOn(clips,'clipRequest').mockResolvedValue('new');
 wrapper=mount(QuickClips,{props:{category:'work'}});await flushPromises();
 expect(wrapper.get('.content-header h1').text()).toBe('工作');
 expect(wrapper.get('.browse-summary [role=status]').text()).toBe('共 0 个结果');
 await wrapper.getComponent(SearchableSelect).vm.$emit('update:modelValue','url');
 await wrapper.get('[type=search]').setValue('alpha');await flushPromises();
 expect(find).toHaveBeenLastCalledWith('alpha',null,{kind:'url',category_id:'work'});
 expect(wrapper.text()).toContain('没有匹配的资料');
 await wrapper.findAll('button').find(b=>b.text()==='添加资料').trigger('click');
 expect(wrapper.get('input[value=url]').element.checked).toBe(true);
 await wrapper.get('input[autocomplete=off]').setValue('Site');
 await wrapper.get('textarea').setValue('https://example.test');
 await wrapper.get('form').trigger('submit');await flushPromises();
 expect(save).toHaveBeenCalledWith({action:'save',id:null,title:'Site',kind:'url',value:'https://example.test',category_id:'work'});
 expect(wrapper.emitted('categories').at(-1)[0]).toEqual(summary);
});
it('creates a sidebar group and requires confirmation before removing it',async()=>{
 const req=vi.spyOn(clips,'clipRequest').mockResolvedValue('work');
 wrapper=mount(ClipCategorySidebar,{props:{selected:'work',summary:{total:1,uncategorized:0,categories:[{id:'work',name:'工作',count:1}]}}});
 await wrapper.get('[aria-label=新建分类]').trigger('click');
 await wrapper.get('input').setValue('个人');await wrapper.get('form').trigger('submit');await flushPromises();
 expect(req).toHaveBeenCalledWith({action:'create_category',name:'个人'});
 await wrapper.get('[aria-label="管理分类 工作"]').trigger('click');
 await wrapper.findAll('button').find(b=>b.text()==='删除').trigger('click');
 expect(req).toHaveBeenCalledTimes(1);expect(wrapper.text()).toContain('移至未分类');
 await wrapper.get('form').trigger('submit');await flushPromises();
 expect(req).toHaveBeenCalledWith({action:'delete_category',id:'work'});expect(wrapper.emitted('select').at(-1)).toEqual(['']);
});

it('keeps copy accessible and dismisses secondary actions on Escape or an outside click',async()=>{
 vi.spyOn(clips,'findClips').mockResolvedValue([{id:'one',title:'Site',kind:'url',content:'https://example.test'}]);
 const request=vi.spyOn(clips,'clipRequest');
 wrapper=mount(QuickClips,{attachTo:document.body});await flushPromises();
 const more=wrapper.get('.row-more > button');
 expect(wrapper.find('.more-items').exists()).toBe(false);
 expect(wrapper.get('[title=复制]').exists()).toBe(true);
 await more.trigger('click');
 await wrapper.get('.more-items button').trigger('keydown',{key:'Escape'});
 expect(more.attributes('aria-expanded')).toBe('false');
 expect(document.activeElement).toBe(more.element);
 await more.trigger('click');
 await wrapper.get('[type=search]').trigger('pointerdown');
 expect(wrapper.find('.more-items').exists()).toBe(false);
 expect(request).not.toHaveBeenCalled();
});
