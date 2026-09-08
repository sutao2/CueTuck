import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import UserManagement from './UserManagement.vue';
import AdminApp from './AdminApp.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';
import { resetAdminSession, setAdminTransport, loginAdmin, setOAuthProviderList } from './session.js';

const target = { email: 'user@example.com', role: 'user', disabled: false, providers: ['google'], has_password: true, publication_count: 2, active_access_count: 3 };
let calls;
async function login(role = 'owner') {
  setAdminTransport(async () => ({ access_token: 'acc.owner', email: 'owner@example.com', role }));
  await loginAdmin({ email: 'owner@example.com', password: 'password' });
}
beforeEach(async () => {
  resetAdminSession(); resetAdminApi(); setOAuthProviderList([]); calls = [];
  await login();
  setAdminApiTransport(async request => {
    calls.push(request);
    if (request.kind === 'users') return { items: [target], total: 31 };
    if (request.kind === 'userDetail') return target;
    if (request.kind === 'userAction') return { updated: true };
    return { items: [] };
  });
});
it('queries and paginates on the server and shows safe account details', async () => {
  const w = mount(UserManagement); await flushPromises();
  await w.get('[data-testid="users-search"]').setValue('user');
  await w.get('[data-testid="users-role"]').setValue('user');
  await w.get('form').trigger('submit'); await flushPromises();
  expect(calls.at(-1).query).toMatchObject({ q: 'user', role: 'user', offset: 0 });
  await w.get('[data-testid="users-search"]').setValue('draft-not-submitted');
  await w.get('[data-testid="users-next"]').trigger('click'); await flushPromises();
  expect(calls.at(-1).query).toMatchObject({ q: 'user', offset: 25 });
  await w.get('[data-testid="user-detail"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="user-detail-panel"]').text()).toContain('google');
  expect(w.text()).toContain('不是设备数量');
});
it('confirms writes, blocks duplicates, retains failed form and refreshes on success', async () => {
  let reject;
  const write = vi.fn(() => new Promise((_, no) => { reject = no; }));
  setAdminApiTransport(async r => r.kind === 'users' ? { items: [target], total: 1 } : r.kind === 'userDetail' ? target : write(r));
  const w = mount(UserManagement); await flushPromises();
  await w.get('[data-testid="user-detail"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="user-toggle"]').trigger('click');
  expect(write).not.toHaveBeenCalled();
  await w.get('[data-testid="user-confirm-password"]').setValue(' current password ');
  await w.get('.user-confirm').trigger('submit');expect(write).not.toHaveBeenCalled();expect(w.text()).toContain('请填写操作原因');
  await w.get('[data-testid="user-reason"]').setValue('  违反公开内容规则  ');
  await w.get('.user-confirm').trigger('submit');
  expect(w.get('[data-testid="user-confirm"]').element.disabled).toBe(true);
  await w.get('.user-confirm').trigger('submit'); expect(write).toHaveBeenCalledTimes(1);
  expect(write.mock.calls[0][0].config.current_password).toBe(' current password ');
  expect(write.mock.calls[0][0].config.reason).toBe('违反公开内容规则');
  reject(new Error('操作失败')); await flushPromises();
  expect(w.text()).toContain('操作失败');
  expect(w.get('[data-testid="user-confirm-password"]').element.value).toBe(' current password ');
  write.mockResolvedValue({ updated: true });
  await w.get('.user-confirm').trigger('submit'); await flushPromises();
  expect(w.text()).toContain('停用账号已完成'); expect(w.find('.user-confirm').exists()).toBe(false);
  expect(w.emitted('busy-change')).toEqual([[true], [false], [true], [false]]);
});
it('never shows success without server confirmation', async () => {
  setAdminApiTransport(async r => r.kind === 'users' ? { items: [target] } : r.kind === 'userDetail' ? target : {});
  const w = mount(UserManagement); await flushPromises();
  await w.get('[data-testid="user-detail"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="user-revoke"]').trigger('click');
  await w.get('[data-testid="user-confirm-password"]').setValue('password');
  await w.get('.user-confirm').trigger('submit'); await flushPromises();
  expect(w.text()).toContain('服务端未确认'); expect(w.find('.user-confirm').exists()).toBe(true);
});
it('read failures are retryable and do not enable actions with defaults', async () => {
  let failed = true;
  setAdminApiTransport(async () => { if (failed) throw new Error('网络断开'); return { items: [target], total: 1 }; });
  const w = mount(UserManagement); await flushPromises();
  expect(w.text()).toContain('网络断开'); expect(w.find('[data-testid="user-detail"]').exists()).toBe(false);
  failed = false; await w.get('form').trigger('submit'); await flushPromises();
  expect(w.find('[data-testid="user-detail"]').exists()).toBe(true);
  failed = true; await w.get('[data-testid="user-detail"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="user-toggle"]').exists()).toBe(false);
});
it('does not allow self operations or admin operations on management staff', async () => {
  await login('admin');
  setAdminApiTransport(async r => r.kind === 'users' ? { items: [{ ...target, email: 'another@example.com', role: 'owner' }] } : { ...target, email: 'another@example.com', role: 'owner' });
  const w = mount(UserManagement); await flushPromises();
  await w.get('[data-testid="user-detail"]').trigger('click'); await flushPromises();
  expect(w.find('[data-testid="user-toggle"]').exists()).toBe(false);
  expect(w.text()).toContain('请由所有者处理');
});
it('only shows review and personal security navigation for reviewers', async () => {
  await login('reviewer');
  const w = mount(AdminApp); await flushPromises();
  expect(w.find('[data-testid="nav-review"]').exists()).toBe(true);
  expect(w.find('[data-testid="nav-security"]').exists()).toBe(true);
  for (const page of ['users', 'oauth', 'settings']) expect(w.find(`[data-testid="nav-${page}"]`).exists()).toBe(false);
  w.unmount();
});
