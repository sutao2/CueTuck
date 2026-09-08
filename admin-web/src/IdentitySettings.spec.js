import {mount,flushPromises} from '@vue/test-utils';
import {it,expect,afterEach,vi} from 'vitest';
import IdentitySettings from './IdentitySettings.vue';
import {setAdminApiTransport,resetAdminApi} from './adminApi.js';
let w;
afterEach(()=>{w?.unmount();resetAdminApi();vi.restoreAllMocks();});
const defaults=r=>r.kind==='invitations'?{items:[],total:0}:{revision:0,registration_open:true};
it('does not offer a default registration write after load failure',async()=>{
  setAdminApiTransport(r=>{if(r.kind==='invitations')return defaults(r);throw Error('加载失败');});
  w=mount(IdentitySettings);await flushPromises();expect(w.find('[data-testid=identity-save]').exists()).toBe(false);expect(w.text()).toContain('加载失败');
});
it('keeps registration draft on conflict and only accepts an incremented revision',async()=>{
  setAdminApiTransport(r=>r.kind==='identitySave'?Promise.reject(Error('版本冲突')):defaults(r));
  w=mount(IdentitySettings);await flushPromises();await w.get('[data-testid=registration-open]').setValue(false);await w.get('form').trigger('submit');await flushPromises();expect(w.vm.hasUnsavedChanges).toBe(true);expect(w.text()).toContain('版本冲突');
});
it('confirms invitation recipients, clears passwords, and distinguishes queued invites from created accounts',async()=>{
  const send=vi.fn(()=>({status:'pending',id:'invite'}));setAdminApiTransport(r=>r.kind==='invite'?send(r):defaults(r));
  w=mount(IdentitySettings);await flushPromises();await w.get('[data-testid=invite-email]').setValue('new@example.com');await w.get('[data-testid=invite-password]').setValue('password');
  vi.spyOn(window,'confirm').mockReturnValue(false);await w.get('.invite-form').trigger('submit');expect(send).not.toHaveBeenCalled();
  window.confirm.mockReturnValue(true);await w.get('.invite-form').trigger('submit');await flushPromises();expect(send.mock.calls[0][0].config.role).toBe('reviewer');expect(w.get('[data-testid=invite-password]').element.value).toBe('');expect(w.text()).toContain('接受前不会创建管理员账号');
});
