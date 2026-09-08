import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, it, expect, vi } from 'vitest';
import OAuthSettings from './OAuthSettings.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';

const provider = (name = 'google') => ({ provider: name, enabled: false, client_id: '', redirect_uri: 'http://localhost:8787/v1/session/oauth/callback', secret_configured: false, source: 'none' });
let wrapper;
beforeEach(resetAdminApi);
afterEach(() => { wrapper?.unmount(); vi.restoreAllMocks(); });

it('loads both providers without a secret value and validates before enabling', async () => {
  const transport = vi.fn(async () => ({ items: [provider(), provider('github')] }));
  setAdminApiTransport(transport);
  wrapper = mount(OAuthSettings); await flushPromises();
  expect(wrapper.findAll('.provider-card')).toHaveLength(2);
  expect(wrapper.get('[data-testid="google-secret"]').element.value).toBe('');
  await wrapper.get('[aria-label="启用 Google 登录"]').setValue(true);
  await wrapper.get('[data-testid="provider-google"]').trigger('submit');
  expect(wrapper.get('[role="alert"]').text()).toContain('启用前');
  expect(transport).toHaveBeenCalledTimes(1);
});

it('keeps the existing secret on blank input, disables double saves, clears replaced secrets and reports configuration only', async () => {
  let finish;
  const config = { ...provider(), enabled: true, secret_configured: true, client_id: 'old-id', source: 'database' };
  const transport = vi.fn(async request => request.kind === 'getOAuth' ? { items: [config] } : new Promise(resolve => { finish = () => resolve({ ...config, client_id: request.config.client_id }); }));
  setAdminApiTransport(transport);
  wrapper = mount(OAuthSettings); await flushPromises();
  await wrapper.get('[data-testid="google-client-id"]').setValue('new-id');
  await wrapper.get('[data-testid="google-current-password"]').setValue('test-password');
  await wrapper.get('form').trigger('submit'); await wrapper.get('form').trigger('submit');
  expect(transport).toHaveBeenCalledTimes(2);
  expect(transport.mock.calls[1][0].config.client_secret).toBe('');
  expect(wrapper.get('[data-testid="google-save"]').element.disabled).toBe(true);
  finish(); await flushPromises();
  await wrapper.get('[data-testid="google-secret"]').setValue('replacement-secret');
  await wrapper.get('[data-testid="google-current-password"]').setValue('test-password');
  await wrapper.get('form').trigger('submit'); finish(); await flushPromises();
  expect(wrapper.get('[data-testid="google-secret"]').element.value).toBe('');
  expect(wrapper.get('[role="status"]').text()).toContain('真实授权验证');
  expect(JSON.stringify(localStorage)).not.toContain('replacement-secret');
});

it('retains failed input and permits retry without touching the other provider', async () => {
  const save = vi.fn().mockRejectedValueOnce(new Error('回调地址格式错误')).mockResolvedValue({ ...provider(), client_id: 'test', secret_configured: true, source: 'database' });
  setAdminApiTransport(async request => request.kind === 'getOAuth' ? { items: [provider(), provider('github')] } : save(request));
  wrapper = mount(OAuthSettings); await flushPromises();
  await wrapper.get('[data-testid="google-client-id"]').setValue('test');
  await wrapper.get('[data-testid="google-secret"]').setValue('secret');
  await wrapper.get('[data-testid="google-current-password"]').setValue('test-password');
  await wrapper.get('[data-testid="provider-google"]').trigger('submit'); await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toContain('回调');
  expect(wrapper.get('[data-testid="google-secret"]').element.value).toBe('secret');
  expect(wrapper.get('[data-testid="google-save"]').element.disabled).toBe(false);
  await wrapper.get('[data-testid="provider-google"]').trigger('submit'); await flushPromises();
  expect(save.mock.calls.every(([call]) => call.provider === 'google')).toBe(true);
  expect(wrapper.get('[data-testid="github-client-id"]').element.value).toBe('');
});

it('shows a load failure with retry rather than editable empty configuration', async () => {
  const transport = vi.fn().mockRejectedValueOnce(new Error('网络失败')).mockResolvedValue({ items: [provider()] });
  setAdminApiTransport(transport); wrapper = mount(OAuthSettings); await flushPromises();
  expect(wrapper.find('form').exists()).toBe(false);
  await wrapper.get('button').trigger('click'); await flushPromises();
  expect(wrapper.find('form').exists()).toBe(true);
});
it('verifies only a saved provider using an official URL and refreshes status without losing drafts',async()=>{
  const config={...provider(),enabled:true,secret_configured:true,client_id:'id',revision:2,verification:{status:'unverified'}};const open=vi.spyOn(window,'open').mockReturnValue(null);
  setAdminApiTransport(r=>r.kind==='verifyOAuth'?{authorization_url:'https://github.com/attacker'}:{items:[config]});wrapper=mount(OAuthSettings);await flushPromises();const verify=()=>wrapper.findAll('button').find(b=>b.text()==='验证授权 ↗');await verify().trigger('click');await flushPromises();expect(open).not.toHaveBeenCalled();expect(wrapper.text()).toContain('安全要求');
  setAdminApiTransport(r=>r.kind==='verifyOAuth'?{authorization_url:'https://accounts.google.com/o/oauth2/v2/auth?state=test'}:{items:[{...config,verification:{status:'succeeded'}}]});await verify().trigger('click');await flushPromises();expect(open).toHaveBeenCalledOnce();expect(wrapper.text()).toContain('等待提供商授权');await wrapper.get('[data-testid=google-client-id]').setValue('draft');expect(verify().element.disabled).toBe(true);await wrapper.findAll('button').find(b=>b.text()==='刷新验证结果').trigger('click');await flushPromises();expect(wrapper.text()).toContain('真实授权已验证');expect(wrapper.get('[data-testid=google-client-id]').element.value).toBe('draft');
});
