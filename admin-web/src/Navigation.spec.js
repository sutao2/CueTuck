import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import AdminApp from './AdminApp.vue';
import { getAdminSession, loginAdmin, resetAdminSession, setAdminTransport, setOAuthProviderList } from './session.js';
import { getAccountSecurity, resetAdminApi, setAdminApiTransport } from './adminApi.js';
import { requestedPage, permittedPage, adminPages, adminNavGroups } from './navigation.js';

let wrappers;
it('groups every page exactly once', () => {
  expect(adminNavGroups.flatMap(group => group.items.map(item => item.page)).sort()).toEqual([...adminPages].sort());
});
it.each(['owner','admin','reviewer'])('renders complete permitted navigation without empty groups for %s', async role => {
  await login(role);
  const w=mountApp(); await flushPromises();
  const permissions=getAdminSession().permissions;
  expect(w.findAll('nav button').map(button=>button.attributes('data-testid').slice(4)).sort()).toEqual(adminPages.filter(page=>permittedPage(page,permissions)).sort());
  for(const group of w.findAll('.nav-group')) expect(group.findAll('button').length).toBeGreaterThan(0);
  expect(w.findAll('nav [aria-current="page"]')).toHaveLength(1);
});
const mountApp = () => { const w=mount(AdminApp); wrappers.push(w); return w; };
async function login(role='owner', token='acc.owner') {
  setAdminTransport(async () => ({ email:'owner@example.com', access_token:token, role }));
  await loginAdmin({email:'owner@example.com', password:'password'});
}
beforeEach(async () => {
  wrappers=[]; window.history.replaceState(null,'','/'); resetAdminSession(); resetAdminApi(); setOAuthProviderList([]);
  await login();
  setAdminApiTransport(async r => r.kind==='security' ? { has_password:true,active_access_count:1 } : r.kind==='siteConfig' ? {revision:0,name:'唤词',square_public:true} : r.kind==='getOAuth' ? {items:[{provider:'google',enabled:false,client_id:'',redirect_uri:'',secret_configured:false}]} : {items:[],total:0});
});
afterEach(() => { wrappers.forEach(w=>w.unmount()); vi.restoreAllMocks(); vi.unstubAllGlobals(); window.history.replaceState(null,'','/'); });

it('opens a direct page and handles browser navigation without persisting tokens', async () => {
  window.history.replaceState(null,'','#/users');
  const w=mountApp(); await flushPromises(); expect(w.find('[data-testid="user-list"]').exists()).toBe(true);
  const scrollTo=vi.fn(); w.get('.admin-workspace').element.scrollTo=scrollTo;
  await w.get('[data-testid="nav-security"]').trigger('click'); await flushPromises();
  expect(scrollTo).toHaveBeenCalledWith({top:0,left:0});
  expect(location.hash).toBe('#/security');
  window.history.replaceState(null,'','#/users'); window.dispatchEvent(new PopStateEvent('popstate')); await flushPromises();
  expect(w.find('[data-testid="user-list"]').exists()).toBe(true);
  expect(Object.keys(sessionStorage).length).toBe(0); expect(Object.keys(localStorage).length).toBe(0);
  expect(requestedPage('#/../../credentials')).toBe('review');
});
it('preserves requested destination while login is required and rejects unauthorized routes', async () => {
  resetAdminSession(); setOAuthProviderList([]);
  setAdminTransport(async () => ({email:'reviewer@example.com',access_token:'acc.reviewer',role:'reviewer'}));
  window.history.replaceState(null,'','#/users');
  const w=mountApp(); await flushPromises();
  expect(w.find('[data-testid="admin-login"]').exists()).toBe(true);
  await w.get('[data-testid="admin-email"]').setValue('reviewer@example.com');
  await w.get('[data-testid="admin-password"]').setValue('password');
  await w.get('[data-testid="admin-login"]').trigger('click'); await flushPromises();
  expect(location.hash).toBe('#/review'); expect(w.text()).toContain('没有该页面');
  expect(w.find('[data-testid="user-list"]').exists()).toBe(false);
});
it('asks before discarding secret drafts and blocks leaving while saving', async () => {
  window.history.replaceState(null,'','#/security');
  const w=mountApp(); await flushPromises();
  await w.get('[data-testid="security-current-password"]').setValue('private-current');
  const confirm=vi.spyOn(window,'confirm').mockReturnValue(false);
  await w.get('[data-testid="nav-users"]').trigger('click'); await flushPromises();
  expect(location.hash).toBe('#/security'); expect(confirm).toHaveBeenCalledOnce();
  const event=new Event('beforeunload',{cancelable:true}); window.dispatchEvent(event); expect(event.defaultPrevented).toBe(true);
  confirm.mockReturnValue(true); await w.get('[data-testid="nav-users"]').trigger('click'); await flushPromises();
  expect(location.hash).toBe('#/users');
  await w.get('[data-testid="nav-security"]').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="security-current-password"]').element.value).toBe('');
});
it('does not show settings save after a load failure and permits retry', async () => {
  setAdminApiTransport(async () => {throw new Error('配置不可用');});
  window.history.replaceState(null,'','#/settings');
  const w=mountApp(); await flushPromises();
  expect(w.text()).toContain('配置不可用'); expect(w.find('[data-testid="settings-save"]').exists()).toBe(false);
  setAdminApiTransport(async () => ({revision:0,name:'唤词',square_public:false}));
  await w.get('.site-settings .panel-heading button').trigger('click'); await flushPromises();
  expect(w.get('[data-testid="setting-square-public"]').element.checked).toBe(false);
});
it('automatically returns to login on 401 and discards secret drafts', async () => {
  window.history.replaceState(null,'','#/security');
  const w=mountApp(); await flushPromises();
  await w.get('[data-testid="security-current-password"]').setValue('private-current');
  resetAdminApi(); vi.stubGlobal('fetch',vi.fn(async () => ({status:401,ok:false})));
  await expect(getAccountSecurity()).rejects.toThrow('登录已失效'); await flushPromises();
  expect(getAdminSession().loggedIn).toBe(false); expect(w.find('[data-testid="admin-login"]').exists()).toBe(true);
  expect(w.find('[data-testid="security-current-password"]').exists()).toBe(false);
});
it('ignores a stale 401 after another account has logged in', async () => {
  resetAdminApi(); let finish;
  vi.stubGlobal('fetch',vi.fn(() => new Promise(resolve=>{finish=resolve;})));
  const pending=getAccountSecurity();
  await login('owner','acc.new'); finish({status:401,ok:false});
  await expect(pending).rejects.toThrow('会话已变化'); expect(getAdminSession().accessToken).toBe('acc.new');
});
it('distinguishes forbidden access from an expired session and explains throttling', async () => {
  resetAdminApi(); vi.stubGlobal('fetch',vi.fn(async () => ({status:403,ok:false})));
  await expect(getAccountSecurity()).rejects.toThrow('没有此操作权限'); expect(getAdminSession().loggedIn).toBe(true);
  fetch.mockResolvedValue({status:429,ok:false});
  await expect(getAccountSecurity()).rejects.toThrow('一分钟后重试');
});
it('covers every implemented page with the fixed role navigation matrix',()=>{
  const staff=['review','security'];
  const operations=[...staff,'overview','users','content','billing','categories','models','reports','rules'];
  expect(adminPages.filter(p=>permittedPage(p,{users:false,configuration:false}))).toEqual(adminPages.filter(p=>staff.includes(p)));
  expect(adminPages.filter(p=>permittedPage(p,{users:true,configuration:false}))).toEqual(adminPages.filter(p=>operations.includes(p)));
  expect(adminPages.every(p=>permittedPage(p,{users:true,configuration:true}))).toBe(true);
  for(const p of adminPages)expect(requestedPage(`#/${p}`)).toBe(p);
});
