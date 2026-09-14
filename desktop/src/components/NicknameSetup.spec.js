import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import NicknameSetup from './NicknameSetup.vue';
import * as session from '../platform/session.js';

let wrapper;
beforeEach(() => session.resetMemorySession());
afterEach(() => { wrapper?.unmount(); vi.restoreAllMocks(); });
async function setup(get, put = vi.fn()) {
  session.setMeTransport({ get, put });
  wrapper = mount(NicknameSetup, { attachTo: document.body, global: { stubs: { teleport: true } } });
  await flushPromises();
  return put;
}

it('continues immediately for an existing public nickname', async () => {
  await setup(async () => ({ display_name: '阿涛' }));
  expect(wrapper.emitted('ready')).toHaveLength(1);
});

it('requires a nonblank nickname, preserves bio and prevents duplicate writes', async () => {
  let resolve;
  const put = await setup(async () => ({ email: 'private@example.test', display_name: '  ', bio: '原简介' }), vi.fn(() => new Promise(r => { resolve = r; })));
  expect(wrapper.text()).not.toContain('private@example.test');
  expect(wrapper.get('[data-testid="nickname-save"]').attributes('disabled')).toBeDefined();
  await wrapper.get('input').setValue('  阿涛  ');
  await wrapper.get('form').trigger('submit');
  await wrapper.get('form').trigger('submit');
  expect(put).toHaveBeenCalledTimes(1);
  expect(put).toHaveBeenCalledWith({ display_name: '阿涛', bio: '原简介' });
  expect(wrapper.emitted('ready')).toBeUndefined();
  resolve({ display_name: '阿涛' }); await flushPromises();
  expect(wrapper.emitted('ready')).toHaveLength(1);
});

it('retains input on save failure and retries without completing early', async () => {
  const put = vi.fn().mockRejectedValueOnce(Error('离线')).mockResolvedValue({ display_name: '阿涛' });
  await setup(async () => ({}), put);
  await wrapper.get('input').setValue('阿涛');
  await wrapper.get('form').trigger('submit'); await flushPromises();
  expect(wrapper.text()).toContain('保存失败：离线');
  expect(wrapper.get('input').element.value).toBe('阿涛');
  expect(wrapper.emitted('ready')).toBeUndefined();
  await wrapper.get('form').trigger('submit'); await flushPromises();
  expect(wrapper.emitted('ready')).toHaveLength(1);
});

it('allows retry after profile read failure and provides a local exit', async () => {
  await setup(vi.fn().mockRejectedValueOnce(Error('离线')).mockResolvedValue({}));
  expect(wrapper.text()).toContain('读取资料失败');
  await wrapper.findAll('button').find(b => b.text() === '重新读取').trigger('click'); await flushPromises();
  expect(wrapper.find('input').exists()).toBe(true);
  await wrapper.get('.nickname-exit').trigger('click');
  expect(wrapper.emitted('logout')).toHaveLength(1);
});

it('ignores a profile response after the account view is unmounted', async () => {
  let resolve;
  await setup(() => new Promise(r => { resolve = r; }));
  wrapper.unmount();
  resolve({ display_name: '旧账号' }); await flushPromises();
  expect(wrapper.emitted('ready')).toBeUndefined();
});
