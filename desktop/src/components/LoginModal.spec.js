import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import LoginModal from './LoginModal.vue';
import { listOAuthProviders, loginOAuthSession, loginSession } from '../platform/session.js';

vi.mock('../platform/session.js', () => ({
  listOAuthProviders: vi.fn(), loginOAuthSession: vi.fn(), loginSession: vi.fn(),
}));
let wrapper;
beforeEach(() => { vi.resetAllMocks(); listOAuthProviders.mockResolvedValue({ items: [] }); });
afterEach(() => { wrapper?.unmount(); document.body.innerHTML = ''; });
async function open(reason = '登录') {
  wrapper = mount(LoginModal, { props: { reason }, attachTo: document.body });
  await flushPromises();
  return wrapper;
}

it('uses a page, focuses email without trapping Tab, and returns focus on leaving', async () => {
  const trigger = document.createElement('button');
  document.body.append(trigger); trigger.focus();
  const w = await open();
  expect(w.get('h2').text()).toBe('登录提示方舟');
  expect(w.get('[data-testid="login-reason"]').text()).not.toBe('登录');
  expect(document.activeElement).toBe(w.get('[type="email"]').element);
  w.get('.login-later').element.focus();
  const tab = new KeyboardEvent('keydown', {key:'Tab',bubbles:true,cancelable:true});
  w.get('.login-later').element.dispatchEvent(tab);
  expect(tab.defaultPrevented).toBe(false);
  expect(w.find('[aria-modal]').exists()).toBe(false);
  expect(w.find('.modal-backdrop').exists()).toBe(false);
  await w.get('[role="region"]').trigger('keydown', { key: 'Escape', isComposing: true });
  expect(w.emitted('cancel')).toBeUndefined();
  await w.get('[role="region"]').trigger('keydown', { key: 'Escape' });
  expect(w.emitted('cancel')).toHaveLength(1);
  w.unmount(); wrapper = null;
  await flushPromises();
  expect(document.activeElement).toBe(trigger);
});

it('blocks duplicate email submits and closing while pending, then preserves input on failure', async () => {
  let reject;
  loginSession.mockReturnValue(new Promise((_, fail) => { reject = fail; }));
  const w = await open('发布需要登录');
  expect(w.get('[data-testid="login-reason"]').text()).toBe('发布需要登录');
  await w.get('[type="email"]').setValue('test@example.com');
  await w.get('[type="password"]').setValue('test-password');
  await w.get('form').trigger('submit');
  await w.get('form').trigger('submit');
  expect(loginSession).toHaveBeenCalledTimes(1);
  expect(w.get('[data-testid="login-submit"]').text()).toBe('正在登录…');
  expect(w.find('[data-testid="oauth-wait"]').exists()).toBe(false);
  await w.get('[role="region"]').trigger('keydown', { key: 'Escape' });
  expect(w.emitted('cancel')).toBeUndefined();
  reject(new Error('邮箱或密码不正确'));
  await flushPromises();
  expect(w.get('[role="alert"]').text()).toBe('邮箱或密码不正确');
  expect(w.get('[type="password"]').element.value).toBe('test-password');
  loginSession.mockResolvedValue({});
  await w.get('form').trigger('submit');
  await flushPromises();
  expect(w.emitted('success')).toHaveLength(1);
});

it('shows configured OAuth providers and aborts authorization on cancel', async () => {
  listOAuthProviders.mockResolvedValue({ items: ['google', 'github', 'unknown'] });
  loginOAuthSession.mockReturnValue(new Promise(() => {}));
  const w = await open();
  expect(w.findAll('.oauth-row button')).toHaveLength(2);
  await w.get('[data-testid="oauth-google"]').trigger('click');
  expect(w.get('[data-testid="oauth-wait"]').text()).toContain('浏览器授权');
  await w.get('[aria-label="返回"]').trigger('click');
  expect(loginOAuthSession.mock.calls[0][1].signal.aborted).toBe(true);
  expect(w.emitted('cancel')).toHaveLength(1);
});
