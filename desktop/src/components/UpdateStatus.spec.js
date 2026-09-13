import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import Status from './UpdateStatus.vue';
import { updateState, resetUpdates } from '../platform/updates.js';
const { invoke }=vi.hoisted(()=>({invoke:vi.fn(async()=>{})}));
vi.mock('../platform/tauri.js',()=>({invokeCommand:invoke}));
let w;
afterEach(()=>{w?.unmount();resetUpdates();invoke.mockClear();document.body.innerHTML='';});
it('shows byte progress and blocks install until a verified package is ready', async()=>{
 Object.assign(updateState,{phase:'downloading',downloaded:1024,total:2048,version:'9.0.0'});
 w=mount(Status);expect(w.get('progress').attributes('value')).toBe('1024');expect(w.get('progress').attributes('max')).toBe('2048');expect(w.text()).not.toContain('安装并重启');
 Object.assign(updateState,{phase:'verifying'});await flushPromises();expect(w.text()).toContain('验证更新签名');
 Object.assign(updateState,{phase:'ready'});await flushPromises();expect(w.text()).toContain('安装并重启');expect(invoke).not.toHaveBeenCalled();
});
it('requires confirmation, respects unsaved settings and allows cancellation', async()=>{
 Object.assign(updateState,{phase:'ready',version:'9.0.0'});w=mount(Status,{props:{dirty:true},attachTo:document.body});
 expect(w.get('button').attributes('disabled')).toBeDefined();await w.setProps({dirty:false});await w.get('button').trigger('click');
 expect(w.get('[role=alertdialog]').text()).toContain('请确认已保存');await w.get('[role=alertdialog]').trigger('keydown',{key:'Escape'});expect(invoke).not.toHaveBeenCalled();
 await w.get('button').trigger('click');await w.get('[role=alertdialog]').findAll('button')[1].trigger('click');await flushPromises();expect(invoke).toHaveBeenCalledWith('install_downloaded_update',{version:'9.0.0'});
});
