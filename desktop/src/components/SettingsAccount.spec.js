import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import SettingsModal from './SettingsModal.vue';
import { resetMemoryLibrary } from '../platform/library.js';
import { setMeTransport } from '../platform/session.js';
import { resetBilling, setBillingTransport } from '../platform/billing.js';

let w;
beforeEach(() => {
  resetMemoryLibrary();
  setMeTransport({ get: async () => ({ display_name: '林晚', bio: '写作与设计' }) });
  setBillingTransport({ status: async () => ({ pro: false, mock: true, mock_pro: true }) });
});
afterEach(() => { w?.unmount(); setMeTransport(null); resetBilling(); });

it('keeps the saved identity on failure and updates it only after a successful retry', async () => {
  let rejectSave;
  const put = vi.fn().mockImplementationOnce(() => new Promise((_, reject) => { rejectSave = reject; }))
    .mockImplementationOnce(async body => body);
  setMeTransport({ get: async () => ({ display_name: '林晚', bio: '写作与设计' }), put });
  w = mount(SettingsModal, { props: { initialPage: 'account', session: { loggedIn: true, email: 'writer@example.com' } } });
  await flushPromises();
  await w.get('[data-settings-page="account"]').trigger('click');
  await w.get('[data-testid="author-display-name"]').setValue('新的名字');
  expect(w.get('[data-testid="author-profile-note"]').text()).toContain('未保存');
  expect(w.get('[data-testid="account-name"]').text()).toBe('林晚');
  await w.get('form.author-profile').trigger('submit');
  expect(w.get('[data-testid="save-author-profile"]').text()).toBe('正在保存…');
  await w.get('form.author-profile').trigger('submit');
  expect(put).toHaveBeenCalledTimes(1);
  rejectSave(new Error('网络不可用'));
  await flushPromises();
  expect(w.get('[data-testid="author-profile-note"]').text()).toBe('网络不可用');
  expect(w.get('[data-testid="account-name"]').text()).toBe('林晚');
  expect(w.get('[data-testid="author-display-name"]').element.value).toBe('新的名字');
  await w.get('form.author-profile').trigger('submit');
  await flushPromises();
  expect(w.get('[data-testid="account-name"]').text()).toBe('新的名字');
  expect(w.get('[data-testid="author-profile-note"]').text()).toBe('资料已保存');
  await w.get('.settings-return').trigger('click');
  expect(w.emitted('cancel')).toHaveLength(1);
});

it('keeps real and simulated entitlements distinct while test controls start collapsed', async () => {
  w = mount(SettingsModal, { props: { session: { loggedIn: true, email: 'writer@example.com' } } });
  await flushPromises();
  await w.get('[data-settings-page="account"]').trigger('click');
  expect(w.get('[data-testid="billing-pro"]').text()).toBe('未订阅');
  expect(w.get('[data-testid="billing-mock"]').text()).toContain('模拟 Pro');
  expect(w.get('[data-testid="billing-mock"]').element.closest('details')).toBeNull();
  expect(w.get('.account-billing-details').element.open).toBe(false);
  expect(w.get('[data-testid="billing-mock-success"]').element.closest('details')).not.toBeNull();
  w.unmount();
  w = mount(SettingsModal); await flushPromises();
  await w.get('[data-settings-page="account"]').trigger('click');
  expect(w.get('[data-testid="account-name"]').text()).toBe('本地访客');
  expect(w.get('[data-testid="author-display-name"]').element.disabled).toBe(true);
  await w.get('[data-testid="settings-login"]').trigger('click');
  expect(w.emitted('login')).toHaveLength(1);
});
