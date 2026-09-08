import {mount,flushPromises} from '@vue/test-utils';
import {it,expect,afterEach} from 'vitest';
import AiJobs from './AiJobs.vue';
import {setAdminApiTransport,resetAdminApi} from './adminApi.js';
let w;afterEach(()=>{w?.unmount();resetAdminApi()});
it('shows persisted status, retries failures and does not claim approval',async()=>{
  let retried=false;setAdminApiTransport(r=>{if(r.method==='POST'){retried=true;return {queued:true}}return {items:[{id:'p',status:retried?'queued':'failed',attempts:1}],total:1}});
  w=mount(AiJobs);await flushPromises();await w.findAll('button').find(b=>b.text()==='立即重试').trigger('click');await flushPromises();
  expect(retried).toBe(true);expect(w.text()).toContain('不代表审核通过');expect(w.text()).toContain('等待执行');
});
it('does not offer retry for exhausted jobs and reports load errors',async()=>{
  setAdminApiTransport(()=>({items:[{id:'p',status:'failed',attempts:3}],total:1}));w=mount(AiJobs);await flushPromises();expect(w.text()).not.toContain('立即重试');
  setAdminApiTransport(()=>{throw Error('服务不可用')});await w.get('button').trigger('click');await flushPromises();expect(w.get('[role=alert]').text()).toBe('服务不可用');
});
it('preserves a retry failure message and does not show queued success',async()=>{
  setAdminApiTransport(r=>{if(r.method==='POST')throw Error('任务已变化');return {items:[{id:'p',status:'failed',attempts:1}],total:1}});
  w=mount(AiJobs);await flushPromises();await w.findAll('button').find(b=>b.text()==='立即重试').trigger('click');await flushPromises();
  expect(w.get('[role=alert]').text()).toBe('任务已变化');expect(w.text()).not.toContain('已重新排队');
});
