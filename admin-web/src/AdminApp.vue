<template>
  <main v-if="!loggedIn" class="login-layout">
    <header class="login-brand"><div class="brand"><img :src="appIcon" alt="" class="brand-mark">唤词 <small>管理控制台</small></div></header>
    <div class="login-content"><IdentityForm v-if="identityMode" :mode="identityMode" :request="adminIdentityRequest" :initial-email="email" @back="identityMode=''" @done="identityDone" /><form v-else class="login" @submit.prevent="submitLogin"><h2>登录管理台</h2><p class="muted">使用管理员账号继续。</p>
      <label>邮箱<input v-model="email" type="email" data-testid="admin-email" autocomplete="username" placeholder="name@example.com" :disabled="busy" required></label>
      <label>密码<input v-model="password" type="password" data-testid="admin-password" autocomplete="current-password" placeholder="输入密码" :disabled="busy" required></label>
      <p v-if="loginNotice" role="status" class="success-message">{{ loginNotice }}</p>
      <p v-if="error" role="alert" data-testid="admin-error" class="error-message">{{ error }}</p>
      <button type="submit" class="primary" data-testid="admin-login" :disabled="busy" @click.prevent="submitLogin">{{ busy ? '正在登录…' : '登录' }}</button>
      <div v-if="oauthProviders.length" class="login-divider">或使用第三方账号</div>
      <div class="oauth-login"><button v-for="name in oauthProviders" :key="name" type="button" :data-testid="`oauth-${name}`" :disabled="busy" @click="submitOAuth(name)">{{ name === 'google' ? 'Google 登录' : 'GitHub 登录' }}</button></div>
      <button v-if="oauthWaiting" type="button" @click="cancelOAuth">取消授权等待</button>
      <div class="oauth-login"><button type="button" :disabled="busy" @click="identityMode='reset'">忘记密码</button><button type="button" :disabled="busy" @click="identityMode='invitation'">接受管理员邀请</button></div>
      <small class="muted login-footnote">普通用户请使用唤词客户端。关闭页面后需重新登录。</small>
    </form></div>
  </main>
  <main v-else class="admin-layout">
    <aside class="admin-sidebar">
      <div class="brand"><img :src="appIcon" alt="" class="brand-mark"><div>唤词<small>管理控制台</small></div></div>
      <nav aria-label="管理导航"><fieldset class="admin-nav" :disabled="writeBusy">
        <section v-for="group in visibleGroups" :key="group.label" class="nav-group" :aria-label="group.label">
          <p class="nav-label">{{ group.label }}</p>
          <button v-for="item in group.items" :key="item.page" :data-testid="'nav-' + item.page" :aria-current="page === item.page ? 'page' : undefined" @click="navigate(item.page)">
            <AppIcon :name="item.icon" /><span>{{ titles[item.page] }}</span>
          </button>
        </section>
      </fieldset></nav>
      <div class="sidebar-account"><span class="account-avatar">{{ account[0]?.toUpperCase() }}</span><div><strong>{{ roleNames[accountRole] || '管理员' }}</strong><small data-testid="admin-account" :title="account">{{ account }}</small></div><button class="logout-button" :disabled="writeBusy" data-testid="admin-logout" @click="logout" title="退出登录" aria-label="退出登录"><AppIcon name="logout" /></button></div>
    </aside>
    <div ref="workspace" class="admin-workspace">
      <header class="workspace-bar"><span class="workspace-title"><AppIcon :name="pageIcon" />{{ titles[page] }}</span><span class="environment-label">唤词 / 管理控制台</span></header>
      <div class="workspace-content" :class="{ 'configuration-page': configurationPages.includes(page) }">
        <header class="page-heading"><div><h1>{{ titles[page] }}</h1><p>{{ descriptions[page] }}</p></div></header>
        <p v-if="error" role="alert" data-testid="admin-error" class="error-banner">{{ error }}</p>
        <OAuthSettings v-if="page === 'oauth'" ref="oauthForm" />
        <ReportManagement v-if="page === 'reports'" ref="riskForm" @busy-change="securityBusy = $event" />
        <SafetyRules v-if="page === 'rules'" ref="riskForm" @busy-change="securityBusy = $event" />
        <ModerationSettings v-if="page === 'moderation'" ref="riskForm" @busy-change="securityBusy = $event" />
        <fieldset v-if="page === 'ai-models' || page === 'ai-skills'" :disabled="aiJobsBusy" style="border:0;padding:0;margin:0;min-width:0"><AiSettings :key="page" ref="riskForm" :mode="page === 'ai-models' ? 'models' : 'skills'" @busy-change="securityBusy = $event" /></fieldset>
        <fieldset v-if="page === 'ai-models'" :disabled="securityBusy" style="border:0;padding:0;margin:0;min-width:0"><AiJobs @busy-change="aiJobsBusy = $event" /></fieldset>
        <MailSettings v-if="page === 'mail'" ref="riskForm" @busy-change="securityBusy = $event" />
        <NotificationSettings v-if="page === 'notifications'" ref="riskForm" @busy-change="securityBusy = $event" />
        <IdentitySettings v-if="page === 'identity'" ref="riskForm" @busy-change="securityBusy = $event" />
        <AccountSecurity v-if="page === 'security'" ref="securityForm" @signed-out="securitySignedOut" @busy-change="securityBusy = $event" />
        <UserManagement v-if="page === 'users'" ref="usersForm" @busy-change="securityBusy = $event" />
        <ReviewManagement v-if="page === 'review'" ref="reviewForm" @busy-change="securityBusy = $event" />
        <ContentManagement v-if="page === 'content'" ref="contentForm" @busy-change="securityBusy = $event" />
        <CatalogManagement v-if="page === 'categories' || page === 'models'" :key="page" :kind="page" ref="catalogForm" @busy-change="securityBusy = $event" />
        <SiteSettings v-if="page === 'settings'" ref="riskForm" @busy-change="securityBusy=$event" @registration="navigate('identity')" />
        <MockBilling v-if="page === 'billing'" ref="riskForm" @busy-change="securityBusy=$event" />
        <Operations v-if="['overview','audit','system'].includes(page)" :key="page" :mode="page" @busy-change="securityBusy=$event" />
      </div>
    </div>
  </main>
</template>

<script setup>
import AiJobs from './AiJobs.vue';
import AppIcon from '../../desktop/src/components/AppIcon.vue';
import appIcon from '../../desktop/src/assets/app-icon.png';
import { adminNavGroups } from './navigation.js';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import OAuthSettings from './OAuthSettings.vue';
import AccountSecurity from './AccountSecurity.vue';
import UserManagement from './UserManagement.vue';
import ReviewManagement from './ReviewManagement.vue';
import ContentManagement from './ContentManagement.vue';
import CatalogManagement from './CatalogManagement.vue';
import ReportManagement from './ReportManagement.vue';
import SafetyRules from './SafetyRules.vue';
import ModerationSettings from './ModerationSettings.vue';
import AiSettings from './AiSettings.vue';
import MailSettings from './MailSettings.vue';
import NotificationSettings from './NotificationSettings.vue';
import IdentitySettings from './IdentitySettings.vue';
import SiteSettings from './SiteSettings.vue';
import MockBilling from './MockBilling.vue';
import Operations from './Operations.vue';
import IdentityForm from '../../shared/IdentityForm.vue';
import { adminIdentityRequest } from './identity.js';
import { clearAdminSession, getAdminSession, listOAuthProviders, loginAdmin, loginAdminOAuth, logoutAdmin } from './session.js';
import { permittedPage, requestedPage } from './navigation.js';
import './admin.css';
const titles = { review: '内容审核', content: '广场内容', users: '用户', categories: '分类管理', models: '模型管理', oauth: '第三方登录', settings: '站点设置', security: '账号安全' };
const descriptions = { review: '检查社区投稿，维护广场内容质量。', content: '管理公开展示、推荐排序与上下架，保留作者原始内容。', users: '检索账号、查看公开资料，安全地管理状态与权限。', oauth: '连接登录提供商，让用户使用已有账号登录唤词。', settings: '管理社区的访问方式与公开范围。', security: '管理本人的登录密码与所有设备的会话。' };
const permissions = ref({}), accountRole = ref(''), workspace = ref(null);
const visibleGroups = computed(() => adminNavGroups.map(group => ({ ...group, items: group.items.filter(item => permittedPage(item.page, permissions.value)) })).filter(group => group.items.length));
const pageIcon = computed(() => adminNavGroups.flatMap(group => group.items).find(item => item.page === page.value)?.icon || 'settings');
const configurationPages = ['oauth','settings','security','moderation','mail','notifications','identity','ai-skills'];
titles.reports='举报与风控'; titles.rules='安全规则';
titles.moderation='自动审核'; descriptions.moderation='设置投稿限额、初筛检查与人工复核边界。';
titles['ai-models']='审核模型'; descriptions['ai-models']='加密接口配置、连接测试与真实审核结果。';
titles['ai-skills']='审核 Skills'; descriptions['ai-skills']='配置版本化审核指令、适用范围和模型路由。';
titles.mail='邮件服务'; descriptions.mail='配置加密 SMTP，查看投递状态和失败重试。';
titles.notifications='通知与日志'; descriptions.notifications='配置高风险邮件与 Webhook，查看投递结果和日志保留策略。';
titles.identity='注册与邀请'; descriptions.identity='管理新账号入口和管理员邀请，不公开管理员注册。';
titles.billing='模拟账单'; descriptions.billing='查看账号权益、模拟订单与测试码，不操作真实支付。';
titles.overview='概览'; descriptions.overview='查看真实记录与当前状态，清晰区分累计和新增。';
titles.audit='操作审计'; descriptions.audit='检索成功操作与失败请求，追溯脱敏记录。';
titles.system='系统状态'; descriptions.system='检查服务实际连通性，确认备份恢复所需条件。';
descriptions.reports='核查公开内容举报，分派处理并追溯每次处置。';
descriptions.rules='维护风险词库和命中解释，为审核提供可验证的依据。';
descriptions.categories = '统一管理广场的分类层级、展示顺序与使用状态。';
descriptions.models = '维护提示词适用的模型字典，供广场筛选与发布使用。';
const roleNames = { owner: '所有者', admin: '运营管理员', reviewer: '审核员' };
function applyIdentity(session) { permissions.value = session.permissions ?? {}; accountRole.value = session.role; }
const loginNotice = ref('');
const identityMode = ref('');
function identityDone(value) { email.value=value; password.value=''; identityMode.value=''; loginNotice.value='验证完成，请使用新密码登录。'; }
const securityBusy = ref(false);
const aiJobsBusy = ref(false);
const oauthForm = ref(null), securityForm = ref(null), usersForm = ref(null), reviewForm = ref(null);
const contentForm = ref(null), catalogForm = ref(null);
const riskForm = ref(null);
const email = ref(''), password = ref(''), error = ref(''), loggedIn = ref(false), account = ref('');
const page = ref('review');
const oauthProviders = ref([]), busy = ref(false), oauthWaiting = ref(false);
const writeBusy = computed(() => securityBusy.value || aiJobsBusy.value || Boolean(oauthForm.value?.isBusy));
const hasUnsavedChanges = computed(() => Boolean(riskForm.value?.hasUnsavedChanges || catalogForm.value?.hasUnsavedChanges || oauthForm.value?.hasUnsavedChanges || securityForm.value?.hasUnsavedChanges || usersForm.value?.hasUnsavedChanges || reviewForm.value?.hasUnsavedChanges || contentForm.value?.hasUnsavedChanges));
let loginAbort = new AbortController();
function canLeave() { return !writeBusy.value && (!hasUnsavedChanges.value || window.confirm('有未保存的修改，确定放弃并离开吗？')); }
function beforeUnload(event) { if (loggedIn.value && (writeBusy.value || hasUnsavedChanges.value)) { event.preventDefault(); event.returnValue = ''; } }
function onHistory() {
  if (!loggedIn.value) return;
  if (requestedPage() === page.value) return;
  navigate(requestedPage(), { history: true });
}
function onExpired() { securitySignedOut('登录已失效，请重新登录。'); }
onMounted(async () => {
  window.addEventListener('popstate', onHistory);
  window.addEventListener('hashchange', onHistory);
  window.addEventListener('beforeunload', beforeUnload);
  window.addEventListener('promptark:session-expired', onExpired);
  const session = getAdminSession();
  applyIdentity(session);
  loggedIn.value = session.loggedIn; account.value = session.email ?? '';
  if (session.loggedIn) await navigate(requestedPage(), { replace: true, force: true }); else await loadProviders();
});
onUnmounted(() => {
  loginAbort.abort();
  window.removeEventListener('popstate', onHistory);
  window.removeEventListener('hashchange', onHistory);
  window.removeEventListener('beforeunload', beforeUnload);
  window.removeEventListener('promptark:session-expired', onExpired);
});
async function loadProviders() { oauthProviders.value = ((await listOAuthProviders()).items ?? []).filter(name => name === 'google' || name === 'github'); }
async function submitLogin() {
  if (busy.value) return;
  busy.value = true; error.value = ''; loginNotice.value = '';
  try { const session = await loginAdmin({ email: email.value, password: password.value }); applyIdentity(session); loggedIn.value = true; account.value = session.email; password.value = ''; await navigate(requestedPage(), { replace: true, force: true }); }
  catch (caught) { error.value = caught.message; }
  finally { busy.value = false; }
}
async function submitOAuth(provider) {
  if (busy.value) return;
  busy.value = true; oauthWaiting.value = true; error.value = '';
  loginAbort.abort(); loginAbort = new AbortController();
  const signal = loginAbort.signal;
  try { const session = await loginAdminOAuth(provider, { signal }); if (signal.aborted) return; applyIdentity(session); loggedIn.value = true; account.value = session.email; await navigate(requestedPage(), { replace: true, force: true }); }
  catch (caught) { if (!signal.aborted) error.value = caught.message; }
  finally { if (!signal.aborted) { busy.value = false; oauthWaiting.value = false; } }
}
function cancelOAuth() { loginAbort.abort(); busy.value = false; oauthWaiting.value = false; }
async function logout() {
  if (!canLeave()) return;
  cancelOAuth(); loggedIn.value = false; account.value = ''; error.value = '';
  if (!await logoutAdmin()) error.value = '本机已退出，但服务端注销失败，请检查网络。';
  await loadProviders();
}
function securitySignedOut(message) {
  aiJobsBusy.value = false;
  securityBusy.value = false;
  cancelOAuth(); clearAdminSession(); loggedIn.value = false; account.value = ''; password.value = '';
  error.value = ''; loginNotice.value = message;
  loadProviders();
}
async function navigate(target, options = {}) {
  if (!options.force && !canLeave()) {
    if (options.history) window.history.replaceState(null, '', `#/${page.value}`);
    return;
  }
  const allowed = permittedPage(target, permissions.value);
  target = allowed ? target : 'review';
  const address = `#/${target}`;
  if (window.location.hash !== address) window.history[options.replace || options.history ? 'replaceState' : 'pushState'](null, '', address);
  if (page.value !== target) workspace.value?.scrollTo?.({ top: 0, left: 0 });
  page.value = target; error.value = allowed ? '' : '当前账号没有该页面的访问权限';
}
</script>
