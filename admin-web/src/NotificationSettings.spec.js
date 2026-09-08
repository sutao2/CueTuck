import {mount,flushPromises} from '@vue/test-utils';
import {it,expect,afterEach,vi} from 'vitest';
import NotificationSettings from './NotificationSettings.vue';
import {setAdminApiTransport,resetAdminApi} from './adminApi.js';
let w;
afterEach(()=>{w?.unmount();resetAdminApi();vi.restoreAllMocks()});
const config=()=>({revision:1,email_enabled:false,recipient:'',webhook_enabled:true,endpoint_configured:true,webhook_host:'notify.example.com',secret_configured:true,threshold:70,daily_limit:50,retention_enabled:false,retention_days:90,active_since:null});
const list=()=>({items:[],total:0,offset:0});
function setup(handler){setAdminApiTransport(r=>handler?.(r)??(r.kind==='notificationList'?list():config()));w=mount(NotificationSettings);return flushPromises()}
it('does not allow saving defaults on a failed configuration load',async()=>{
  await setup(r=>{if(r.kind==='notificationConfig')throw Error('读取失败')});
  expect(w.find('[data-testid=notify-save]').exists()).toBe(false);expect(w.text()).toContain('读取失败');
});
it('requires password, preserves drafts on conflict and never submits secret flags',async()=>{
  let sent;await setup(r=>{if(r.kind==='notificationSave'){sent=r.config;throw Error('版本冲突')}});
  await w.get('[data-testid=notify-endpoint]').setValue('https://new.example.com/events');
  await w.get('form').trigger('submit');expect(sent).toBeUndefined();expect(w.text()).toContain('请输入当前管理员密码');
  await w.get('[data-testid=notify-password]').setValue('test-password');await w.get('form').trigger('submit');await flushPromises();
  expect(sent.endpoint_configured).toBeUndefined();expect(sent.secret).toBe('');expect(w.vm.hasUnsavedChanges).toBe(true);expect(w.text()).toContain('版本冲突');
});
it('confirms log retention and clears secrets after a successful save',async()=>{
  const save=vi.fn(r=>({...config(),...r.config,revision:2}));await setup(r=>r.kind==='notificationSave'?save(r):undefined);
  await w.get('[data-testid=notify-retention]').setValue(true);await w.get('[data-testid=notify-password]').setValue('test-password');
  vi.spyOn(window,'confirm').mockReturnValue(false);await w.get('form').trigger('submit');expect(save).not.toHaveBeenCalled();
  window.confirm.mockReturnValue(true);await w.get('[data-testid=notify-secret]').setValue('synthetic-new-secret');await w.get('form').trigger('submit');await flushPromises();
  expect(save).toHaveBeenCalledTimes(1);expect(w.get('[data-testid=notify-secret]').element.value).toBe('');expect(w.get('[data-testid=notify-password]').element.value).toBe('');expect(w.vm.hasUnsavedChanges).toBe(false);
});
it('tests only saved settings after confirmation and never labels queued as delivered',async()=>{
  const send=vi.fn(()=>({id:'test',status:'queued'}));await setup(r=>r.kind==='notificationTest'?send(r):undefined);
  const button=w.findAll('button').find(b=>b.text()==='发送测试 Webhook');
  await w.get('[data-testid=notify-endpoint]').setValue('https://new.example.com/events');expect(button.attributes('disabled')).toBeDefined();
  await w.get('[data-testid=notify-endpoint]').setValue('');vi.spyOn(window,'confirm').mockReturnValue(false);await button.trigger('click');expect(send).not.toHaveBeenCalled();
  window.confirm.mockReturnValue(true);await button.trigger('click');await flushPromises();expect(send.mock.calls[0][0].config).toEqual({revision:1,channel:'webhook'});expect(w.text()).toContain('入队不代表已接收');expect(w.text()).not.toContain('发送成功');
});
it('uses server pagination and requires explicit retry of a failed delivery',async()=>{
  const calls=[];await setup(r=>{calls.push(r);if(r.kind==='notificationList')return {items:[{id:'failed',metadata:{test:true},channel:'webhook',status:'failed',attempts:1,revision:3,created_at:new Date().toISOString(),expires_at:new Date(Date.now()+60000).toISOString()}],total:26,offset:Number(r.riskPath.split('offset=')[1]||0)};if(r.kind==='notificationRetry')return {status:'queued'}});
  vi.spyOn(window,'confirm').mockReturnValue(false);await w.findAll('button').find(b=>b.text()==='重试').trigger('click');expect(calls.some(r=>r.kind==='notificationRetry')).toBe(false);
  window.confirm.mockReturnValue(true);await w.findAll('button').find(b=>b.text()==='重试').trigger('click');await flushPromises();expect(calls.find(r=>r.kind==='notificationRetry').config).toEqual({revision:3});
  await w.findAll('button').find(b=>b.text()==='下一页').trigger('click');await flushPromises();expect(calls.some(r=>r.riskPath.endsWith('offset=25'))).toBe(true);
});
