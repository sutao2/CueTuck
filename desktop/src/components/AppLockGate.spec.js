import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { defineComponent, inject } from 'vue';
import Gate from './AppLockGate.vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}));
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn()}));
let state, changed, wrapper, deviceAvailable, deviceVerified;
const deviceRequests = () => invoke.mock.calls.filter(([,args])=>args?.request?.action==='unlock_device').length;
const workspace=defineComponent({
  setup:()=>({lock:inject('appLock',null)}),
  template:'<section data-private><button v-if="lock" @click="lock.openSettings">settings</button><button v-if="lock" @click="lock.lock(true)">dirty</button><button v-if="lock" @click="lock.lock()">lock</button></section>',
});
it.each(['subscription','status'])('recovers from a temporary %s error while keeping the workspace hidden',async failure=>{
  vi.useFakeTimers();
  try {
    (failure==='subscription' ? listen : invoke).mockRejectedValueOnce(new Error('not ready'));
    state.device=false;
    wrapper=mount(Gate,{slots:{default:workspace}});await flushPromises();
    expect(wrapper.get('[role=alert]').text()).toContain('无法读取应用锁状态');
    expect(wrapper.find('[data-private]').exists()).toBe(false);
    await vi.advanceTimersByTimeAsync(1000);await flushPromises();
    expect(wrapper.get('input[type=password]').exists()).toBe(true);
    expect(wrapper.find('[data-private]').exists()).toBe(false);
  } finally { vi.useRealTimers(); }
});
it('keeps manual retry available when the lock status stays unreadable',async()=>{
  vi.useFakeTimers();
  try {
    listen.mockRejectedValue(new Error('unavailable'));
    wrapper=mount(Gate,{slots:{default:workspace}});await flushPromises();
    await vi.advanceTimersByTimeAsync(2000);await flushPromises();
    expect(listen).toHaveBeenCalledTimes(2);
    expect(wrapper.find('[data-private]').exists()).toBe(false);
    await wrapper.get('section button').trigger('click');await flushPromises();
    expect(listen).toHaveBeenCalledTimes(3);
  } finally { vi.useRealTimers(); }
});
beforeEach(()=>{
  HTMLDialogElement.prototype.showModal ??= function(){ this.open=true; };
  HTMLDialogElement.prototype.close ??= function(){ this.open=false; };
  window.__TAURI_INTERNALS__={};
  state={enabled:true,locked:true,device:true}; deviceAvailable=true; deviceVerified=false;
  vi.spyOn(document,'hasFocus').mockReturnValue(true);
  listen.mockImplementation(async(_,callback)=>{changed=callback;return ()=>{};});
  invoke.mockImplementation(async(cmd,{request}={})=>{
    if(cmd==='app_lock_status')return {...state};
    if(cmd==='app_lock_device_available')return deviceAvailable;
    if(cmd==='app_lock_action'){
      if(request.action==='unlock' && request.password!=='correct-password')throw new Error('解锁口令不正确');
      if(request.action==='unlock_device' && !deviceVerified)throw new Error('设备验证未完成，可使用解锁口令');
      if(request.action==='unlock' || request.action==='unlock_device')state.locked=false;
      if(request.action==='lock')state.locked=true;
      if(request.action==='enable')state.enabled=true;
      await changed();
    }
  });
});
afterEach(()=>{wrapper?.unmount();delete window.__TAURI_INTERNALS__;vi.restoreAllMocks();vi.resetAllMocks();});
it('only mounts private content after a successful unlock and removes it on manual lock',async()=>{
  wrapper=mount(Gate,{slots:{default:workspace}});
  expect(wrapper.find('[data-private]').exists()).toBe(false);
  await flushPromises();
  expect(deviceRequests()).toBe(1);
  await changed();window.dispatchEvent(new Event('focus'));await flushPromises();
  expect(deviceRequests()).toBe(1);
  expect(wrapper.get('[role=alert]').text()).toContain('设备验证未完成');
  await wrapper.get('input').setValue('wrong');await wrapper.get('form').trigger('submit');await flushPromises();
  expect(wrapper.find('[data-private]').exists()).toBe(false);
  expect(wrapper.get('input').element.value).toBe('');
  await wrapper.get('input').setValue('correct-password');await wrapper.get('form').trigger('submit');await flushPromises();
  expect(wrapper.get('[data-private]').exists()).toBe(true);
  await wrapper.get('[data-private] button:last-child').trigger('click');await flushPromises();
  expect(wrapper.find('[data-private]').exists()).toBe(false);
  expect(deviceRequests()).toBe(2);
});
it('defers the first device request until foreground and unlocks only on confirmed verification',async()=>{
  document.hasFocus.mockReturnValue(false);
  wrapper=mount(Gate,{slots:{default:workspace}});await flushPromises();
  expect(deviceRequests()).toBe(0);
  expect(wrapper.find('[data-private]').exists()).toBe(false);
  expect(wrapper.find('input[type=password]').exists()).toBe(false);
  expect(wrapper.get('.unlock-alternative').text()).toBe('使用解锁口令');
  deviceVerified=true;document.hasFocus.mockReturnValue(true);
  window.dispatchEvent(new Event('focus'));await flushPromises();
  expect(deviceRequests()).toBe(1);
  expect(wrapper.get('[data-private]').exists()).toBe(true);
});
it('switches to the passphrase field on request and keeps that choice on refocus',async()=>{
  document.hasFocus.mockReturnValue(false);
  wrapper=mount(Gate,{attachTo:document.body,slots:{default:workspace}});await flushPromises();
  await wrapper.get('.unlock-alternative').trigger('click');await flushPromises();
  const input=wrapper.get('input[type=password]');
  expect(document.activeElement).toBe(input.element);
  document.hasFocus.mockReturnValue(true);
  window.dispatchEvent(new Event('focus'));await flushPromises();
  expect(deviceRequests()).toBe(0);
  await input.setValue('correct-password');await wrapper.get('form').trigger('submit');await flushPromises();
  expect(wrapper.get('[data-private]').exists()).toBe(true);
});
it('allows an explicit retry after cancellation',async()=>{
  wrapper=mount(Gate,{slots:{default:workspace}});await flushPromises();
  expect(wrapper.get('[role=alert]').text()).toContain('设备验证未完成');
  deviceVerified=true;await wrapper.get('section > button').trigger('click');await flushPromises();
  expect(deviceRequests()).toBe(2);
  expect(wrapper.get('[data-private]').exists()).toBe(true);
});
it.each([[false,true,false],[true,false,false],[true,true,true]])('avoids device prompts when not enabled, unavailable, or in the launcher (%s,%s,%s)',async(enabled,available,launcher)=>{
  state.device=enabled;deviceAvailable=available;
  wrapper=mount(Gate,{props:{launcher},slots:{default:workspace}});await flushPromises();
  expect(deviceRequests()).toBe(0);
  if(!launcher) expect(wrapper.get('input[type=password]').exists()).toBe(true);
});
it('keeps both failed reads and a stale unlock response out of the launcher',async()=>{
  invoke.mockRejectedValueOnce(new Error('offline'));
  wrapper=mount(Gate,{props:{launcher:true},slots:{default:workspace}});await flushPromises();
  expect(wrapper.find('[data-private]').exists()).toBe(false);
  let release;
  invoke.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));
  const pending=changed();await changed();release({locked:false});await pending;
  expect(wrapper.find('[data-private]').exists()).toBe(false);
  expect(wrapper.find('form').exists()).toBe(false);
  state.locked=false;await changed();await flushPromises();expect(wrapper.get('[data-private]').exists()).toBe(true);
});
it('checks confirmation before enabling and clears the password fields after saving',async()=>{
  state={enabled:false,locked:false,device:false};
  wrapper=mount(Gate,{slots:{default:workspace}});
  await flushPromises();await wrapper.get('[data-private] button').trigger('click');await flushPromises();
  const fields=wrapper.findAll('dialog input');
  await fields[0].setValue('12345678');await fields[1].setValue('different');await wrapper.get('dialog form').trigger('submit');
  expect(wrapper.get('[role=alert]').text()).toContain('不一致');
  expect(invoke.mock.calls.some(([cmd])=>cmd==='app_lock_action')).toBe(false);
  await wrapper.get('input[type=checkbox]').setValue(true);
  await fields[1].setValue('12345678');await wrapper.get('dialog form').trigger('submit');await flushPromises();
  expect(invoke).toHaveBeenCalledWith('app_lock_action',{request:{action:'enable',password:'12345678',device:true}});
  expect(wrapper.find('dialog').exists()).toBe(false);
});

it('lets the user keep editing an unsaved clip before locking',async()=>{
  state.locked=false;
  wrapper=mount(Gate,{slots:{default:workspace}});await flushPromises();
  await wrapper.get('[data-private] button:nth-child(2)').trigger('click');await flushPromises();
  expect(wrapper.get('dialog').text()).toContain('资料尚未保存');
  const button=text=>wrapper.findAll('button').find(b=>b.text()===text);
  await button('继续编辑').trigger('click');
  expect(wrapper.get('[data-private]').exists()).toBe(true);
  await wrapper.get('[data-private] button:nth-child(2)').trigger('click');await flushPromises();
  await button('丢弃并锁定').trigger('click');await flushPromises();
  expect(wrapper.find('[data-private]').exists()).toBe(false);
});
