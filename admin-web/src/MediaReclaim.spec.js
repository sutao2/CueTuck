import {mount,flushPromises} from '@vue/test-utils';
import {it,expect,afterEach,vi} from 'vitest';
import MediaReclaim from './MediaReclaim.vue';
import {setAdminApiTransport,resetAdminApi,listOrphanMedia,purgeOrphanMedia} from './adminApi.js';
import {loginAdmin,resetAdminSession,setAdminTransport} from './session.js';
let wrapper;
const file={id:'media.test',name:'example.txt',size:1024,deleting:false};
afterEach(()=>{wrapper?.unmount();resetAdminApi();resetAdminSession();vi.unstubAllGlobals()});
async function inspect(){await wrapper.get('[data-testid=media-inspect]').trigger('click');await flushPromises()}
async function select(){await wrapper.get('.reclaim-row button').trigger('click')}
it('requires explicit inspection and confirmation; cancelling never deletes',async()=>{
  const calls=[];setAdminApiTransport(r=>{calls.push(r);return {items:[file],more:true}});
  wrapper=mount(MediaReclaim);await flushPromises();expect(calls).toHaveLength(0);
  await inspect();expect(calls[0].riskPath).toBe('media/orphans');expect(wrapper.text()).toContain('25');
  await select();expect(wrapper.text()).toContain('无法撤销');expect(wrapper.text()).toContain('离线');
  await wrapper.get('[data-testid=media-cancel]').trigger('click');expect(calls).toHaveLength(1);expect(wrapper.find('[aria-label=确认回收]').exists()).toBe(false);
});
it('blocks duplicate requests while busy and only removes a confirmed successful candidate',async()=>{
  let resolve;const calls=[];setAdminApiTransport(r=>{calls.push(r);return r.method==='POST'?new Promise(done=>{resolve=done}):{items:[file]}});
  wrapper=mount(MediaReclaim);await inspect();await select();
  const confirm=wrapper.get('[data-testid=media-confirm]');await confirm.trigger('click');await confirm.trigger('click');
  expect(calls).toHaveLength(2);expect(calls[1]).toMatchObject({method:'POST',riskPath:'media/orphans/media.test/purge',config:{confirm:true}});
  expect(wrapper.get('[data-testid=media-inspect]').element.disabled).toBe(true);expect(wrapper.emitted('busy-change').at(-1)).toEqual([true]);
  resolve({removed:true});await flushPromises();expect(wrapper.find('.reclaim-row').exists()).toBe(false);expect(wrapper.text()).toContain('本地副本未删除');expect(wrapper.emitted('busy-change').at(-1)).toEqual([false]);
});
it('retains failed and unconfirmed candidates for explicit retry',async()=>{
  let attempt=0;setAdminApiTransport(r=>{if(r.method!=='POST')return {items:[{...file,deleting:true}]};attempt++;if(attempt===1)throw Error('存储不可用');return attempt===2?{}:{removed:true}});
  wrapper=mount(MediaReclaim);await inspect();expect(wrapper.text()).toContain('重试清理');await select();
  await wrapper.get('[data-testid=media-confirm]').trigger('click');await flushPromises();expect(wrapper.text()).toContain('存储不可用');expect(wrapper.find('.reclaim-row').exists()).toBe(true);
  await wrapper.get('[data-testid=media-confirm]').trigger('click');await flushPromises();expect(wrapper.text()).toContain('未确认回收完成');expect(wrapper.find('.reclaim-row').exists()).toBe(true);
  await wrapper.get('[data-testid=media-confirm]').trigger('click');await flushPromises();expect(wrapper.find('.reclaim-row').exists()).toBe(false);
});
it('clears stale candidates on failed or malformed reinspection',async()=>{
  let attempt=0;setAdminApiTransport(()=>{attempt++;if(attempt===1)return {items:[file]};if(attempt===2)throw Error('权限已撤销');return {}});
  wrapper=mount(MediaReclaim);await inspect();await select();await inspect();expect(wrapper.text()).toContain('权限已撤销');expect(wrapper.find('.reclaim-row').exists()).toBe(false);expect(wrapper.find('[data-testid=media-confirm]').exists()).toBe(false);
  await inspect();expect(wrapper.text()).toContain('候选响应无效');expect(wrapper.text()).not.toContain('当前没有可回收文件');
});
it('sends authenticated explicit confirmations and distinguishes read failure from uncertain deletion',async()=>{
  setAdminTransport(()=>({access_token:'test-only',email:'owner@example.com'}));await loginAdmin({email:'owner@example.com',password:'test-only'});
  const fetch=vi.fn().mockResolvedValue({ok:false,status:503});vi.stubGlobal('fetch',fetch);
  await expect(listOrphanMedia()).rejects.toThrow('本次没有执行清理');
  await expect(purgeOrphanMedia('media.test')).rejects.toThrow('不能假定文件仍可读取');
  expect(fetch.mock.calls[1][0]).toContain('/v1/admin/media/orphans/media.test/purge');expect(fetch.mock.calls[1][1]).toMatchObject({method:'POST',headers:{authorization:'Bearer test-only'},body:'{"confirm":true}'});
  fetch.mockResolvedValue({ok:false,status:409});await expect(purgeOrphanMedia('media.test')).rejects.toThrow('候选已变化');
});
