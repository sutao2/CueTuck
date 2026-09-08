import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, it, expect, vi } from 'vitest';
import AccountSecurity from './AccountSecurity.vue';
import AdminApp from './AdminApp.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';
import { resetAdminSession, setAdminTransport, getAdminSession } from './session.js';

beforeEach(() => { resetAdminApi(); resetAdminSession(); });
const info = { has_password: true, active_access_count: 2 };
async function fill(w, current = 'current-password', next = 'changed-password', confirm = next) {
  await w.get('[data-testid="security-current-password"]').setValue(current);
  await w.get('[data-testid="security-new-password"]').setValue(next);
  await w.get('[data-testid="security-confirm-password"]').setValue(confirm);
}
async function submit(w) { await w.get('form').trigger('submit'); await flushPromises(); }

it('requires loaded state and retries instead of showing a default editable form', async () => {
  const api = vi.fn().mockRejectedValueOnce(new Error('读取失败')).mockResolvedValue(info);
  setAdminApiTransport(api);
  const w = mount(AccountSecurity); await flushPromises();
  expect(w.text()).toContain('读取失败');
  expect(w.find('form').exists()).toBe(false);
  await w.get('button').trigger('click'); await flushPromises();
  expect(w.find('form').exists()).toBe(true);
  expect(w.text()).toContain('2 个有效访问令牌');
});

it('validates passwords before sending and does not trim them', async () => {
  const api = vi.fn(async () => info); setAdminApiTransport(api);
  const w = mount(AccountSecurity); await flushPromises();
  for (const values of [['current-password', 'short'], ['current-password', 'current-password'], ['current-password', 'changed-password', 'different-password']]) {
    await fill(w, ...values); await submit(w);
    expect(w.find('[role="alert"]').exists()).toBe(true);
  }
  expect(api).toHaveBeenCalledTimes(1);
  await fill(w, ' current-password ', ' changed-password '); await submit(w);
  expect(api).toHaveBeenLastCalledWith({ kind: 'password', config: { current_password: ' current-password ', new_password: ' changed-password ' } });
});

it('blocks duplicate submissions, keeps failed input and clears secrets after success', async () => {
  let fail;
  const save = vi.fn(() => new Promise((_, reject) => { fail = reject; }));
  setAdminApiTransport(req => req.kind === 'security' ? info : save(req));
  const w = mount(AccountSecurity); await flushPromises(); await fill(w);
  await submit(w); await submit(w);
  expect(save).toHaveBeenCalledTimes(1);
  expect(w.get('fieldset').element.disabled).toBe(true);
  expect(w.emitted('busy-change')[0]).toEqual([true]);
  fail(new Error('当前密码不正确')); await flushPromises();
  expect(w.get('[data-testid="security-new-password"]').element.value).toBe('changed-password');
  expect(w.emitted('signed-out')).toBeUndefined();
  save.mockResolvedValue({ signed_out: true }); await submit(w);
  expect(w.emitted('signed-out')).toHaveLength(1);
  expect(w.get('[data-testid="security-current-password"]').element.value).toBe('');
  expect(w.get('[data-testid="security-new-password"]').element.value).toBe('');
});

it('requires explicit confirmation to revoke sessions and supports retry', async () => {
  const revoke = vi.fn().mockRejectedValueOnce(new Error('退出失败')).mockResolvedValue({ signed_out: true });
  setAdminApiTransport(req => req.kind === 'security' ? info : revoke(req));
  const w = mount(AccountSecurity); await flushPromises();
  await w.get('[data-testid="security-revoke"]').trigger('click');
  expect(revoke).not.toHaveBeenCalled();
  await w.get('[data-testid="security-revoke-confirm"]').trigger('click'); await flushPromises();
  expect(w.text()).toContain('退出失败');
  expect(w.emitted('signed-out')).toBeUndefined();
  await w.get('[data-testid="security-revoke-confirm"]').trigger('click'); await flushPromises();
  expect(w.emitted('signed-out')).toHaveLength(1);
});

it('does not offer password creation to OAuth-only accounts', async () => {
  setAdminApiTransport(async () => ({ ...info, has_password: false }));
  const w = mount(AccountSecurity); await flushPromises();
  expect(w.find('form').exists()).toBe(false);
  expect(w.text()).toContain('尚未设置本地密码');
  expect(w.find('[data-testid="security-revoke"]').exists()).toBe(true);
});

it('returns the management app to login and clears its token after successful change', async () => {
  setAdminTransport(async () => ({ email: 'admin@example.com', access_token: 'acc.admin' }));
  setAdminApiTransport(async req => req.kind === 'security' ? info : req.kind === 'password' ? { signed_out: true } : { items: [] });
  const w = mount(AdminApp);
  await w.get('[data-testid="admin-email"]').setValue('admin@example.com');
  await w.get('[data-testid="admin-password"]').setValue('current-password');
  await w.get('[data-testid="admin-login"]').trigger('click'); await flushPromises();
  await w.get('[data-testid="nav-security"]').trigger('click'); await flushPromises();
  await fill(w); await submit(w);
  expect(getAdminSession().loggedIn).toBe(false);
  expect(w.text()).toContain('密码已修改，所有设备已退出');
  expect(w.find('[data-testid="nav-security"]').exists()).toBe(false);
  expect(w.get('[data-testid="admin-password"]').element.value).toBe('');
});
