import {mount,flushPromises} from '@vue/test-utils';
import {it,expect,afterEach,vi} from 'vitest';
import IdentityForm from '../../../shared/IdentityForm.vue';
let w;afterEach(()=>w?.unmount());
it('blocks new verification requests when email is unavailable but permits an existing code',async()=>{
  const request=vi.fn(()=>({registration_open:true,email_enabled:false}));w=mount(IdentityForm,{props:{mode:'registration',request}});await flushPromises();expect(w.get('[data-testid=identity-submit]').attributes('disabled')).toBeDefined();
  await w.findAll('button').find(b=>b.text()==='已有验证码').trigger('click');expect(w.find('[data-testid=identity-code]').exists()).toBe(true);expect(request).toHaveBeenCalledTimes(1);
});
it('never treats mail request acceptance as account creation or login',async()=>{
  const request=vi.fn(action=>action==='options'?{registration_open:true,email_enabled:true}:{message:'若符合条件，将发送验证邮件'});w=mount(IdentityForm,{props:{mode:'registration',request}});await flushPromises();await w.get('[data-testid=identity-email]').setValue('new@example.com');await w.get('form').trigger('submit');await flushPromises();expect(w.emitted('done')).toBeUndefined();expect(w.text()).toContain('若符合条件');expect(request.mock.calls[1][1]).toEqual({kind:'registration',email:'new@example.com'});
});
it('checks password confirmation, rejects ambiguous success, and clears secrets only on verified completion',async()=>{
  const request=vi.fn(action=>action==='options'?{registration_open:true,email_enabled:true}:{});w=mount(IdentityForm,{props:{mode:'invitation',request,initialEmail:'new@example.com'}});await flushPromises();await w.get('[data-testid=identity-code]').setValue('a'.repeat(64));await w.get('[data-testid=identity-new-password]').setValue('new-password-123');await w.get('[data-testid=identity-confirm-password]').setValue('different-password');await w.get('form').trigger('submit');expect(request).toHaveBeenCalledTimes(1);expect(w.text()).toContain('两次密码不一致');
  await w.get('[data-testid=identity-confirm-password]').setValue('new-password-123');await w.get('form').trigger('submit');await flushPromises();expect(w.emitted('done')).toBeUndefined();expect(w.text()).toContain('服务端未确认');request.mockResolvedValue({status:'verified'});await w.get('form').trigger('submit');await flushPromises();expect(w.emitted('done')[0]).toEqual(['new@example.com']);expect(w.get('[data-testid=identity-code]').element.value).toBe('');expect(w.get('[data-testid=identity-new-password]').element.value).toBe('');
});
