import {mount,flushPromises} from '@vue/test-utils';
import {expect,it,vi,afterEach} from 'vitest';
import QuickClips from './QuickClips.vue';
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
 await button('编辑').trigger('click');
 expect(wrapper.get('input[type=password]').element.value).toBe('');
 await wrapper.get('form input:not([type=password])').setValue('Other');
 await wrapper.get('form').trigger('submit');await flushPromises();
 expect(request.mock.calls).toEqual([[{action:'save',id:'saved',kind:'password',title:'Other',value:null}]]);
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
