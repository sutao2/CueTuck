import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import NicknameSetup from './NicknameSetup.vue';
import * as session from '../platform/session.js';
import { resetMemoryLibrary } from '../platform/library.js';
import { resetSquare, setSquareTransport, setCatalogTransport } from '../platform/square.js';

let wrapper;
beforeEach(() => {
  resetMemoryLibrary(); resetSquare(); session.resetMemorySession();
  session.setSessionTransport(async () => ({ email: 'test@example.test', access_token: 'test-token' }));
  setSquareTransport(async () => []);
  setCatalogTransport(async () => ({ categories: [], models: [] }));
});
afterEach(() => { wrapper?.unmount(); vi.restoreAllMocks(); });
async function shell() {
  wrapper = mount(WorkbenchShell, { attachTo: document.body, global: { stubs: { teleport: true } } });
  await flushPromises();
}

it.each(['email', 'google', 'github'])('requires a nickname before resuming publication after %s login', async provider => {
  session.setOAuthProviderList(['google', 'github']);
  session.setMeTransport({ get: async () => ({}), put: async value => value });
  await shell();
  await wrapper.get('[data-space="square"]').trigger('click');
  await wrapper.get('[data-testid="publish-prompt"]').trigger('click'); await flushPromises();
  if (provider === 'email') {
    await wrapper.get('[data-testid="login-email"]').setValue('test@example.test');
    await wrapper.get('[data-testid="login-password"]').setValue('test-password');
    await wrapper.get('[data-testid="login-modal"] form').trigger('submit');
  } else await wrapper.get(`[data-testid="oauth-${provider}"]`).trigger('click');
  await flushPromises();
  expect(wrapper.attributes('inert')).toBe('');
  expect(wrapper.find('[data-testid="publish-resume"]').exists()).toBe(false);
  const setup = wrapper.getComponent(NicknameSetup);
  await setup.get('input').setValue('测试作者');
  await setup.get('form').trigger('submit'); await flushPromises();
  expect(wrapper.attributes('inert')).toBeUndefined();
  expect(wrapper.find('[data-testid="publish-resume"]').exists()).toBe(true);
});

it('checks a restored session and allows logout without losing local access', async () => {
  session.setMeTransport({ get: async () => ({}) });
  vi.spyOn(session, 'restoreSession').mockImplementation(async () => {
    await session.loginSession({ email: 'test@example.test', password: 'test-password' });
    return session.getSession();
  });
  await shell();
  expect(wrapper.findComponent(NicknameSetup).exists()).toBe(true);
  await wrapper.getComponent(NicknameSetup).get('.nickname-exit').trigger('click'); await flushPromises();
  expect(session.getSession().loggedIn).toBe(false);
  expect(wrapper.findComponent(NicknameSetup).exists()).toBe(false);
  expect(wrapper.attributes('inert')).toBeUndefined();
});

it('restores an existing nickname on repeated launches without displaying onboarding', async () => {
  const put = vi.fn();
  let resolve;
  session.setMeTransport({ get: () => new Promise(r => { resolve = r; }), put });
  vi.spyOn(session, 'restoreSession').mockImplementation(async () => {
    await session.loginSession({ email: 'test@example.test', password: 'test-password' });
    return session.getSession();
  });
  for (let launch = 0; launch < 2; launch++) {
    await shell();
    expect(wrapper.find('.nickname-backdrop').exists()).toBe(false);
    resolve({ display_name: '已有昵称', bio: '已有简介' }); await flushPromises();
    expect(wrapper.findComponent(NicknameSetup).exists()).toBe(false);
    expect(wrapper.attributes('inert')).toBeUndefined();
    wrapper.unmount(); wrapper = null;
  }
  expect(put).not.toHaveBeenCalled();
});
