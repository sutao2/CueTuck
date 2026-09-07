<template>
  <main v-if="!loggedIn" class="login-layout">
    <aside class="login-brand"><div class="brand"><span class="brand-mark">P</span>提示方舟 <small>ADMIN</small></div><div><p class="eyebrow">PROMPTARK CONSOLE</p><h1>让好用的提示词，<br>被更多人发现。</h1><p>管理社区内容、用户与登录服务。<br>一个清晰、有序的工作空间。</p></div><small>管理控制台 · 仅限管理员访问</small></aside>
    <div class="login-content"><form class="login" @submit.prevent="submitLogin"><span class="eyebrow">WELCOME BACK</span><h2>登录管理台</h2><p class="muted">使用管理员账号继续。</p>
      <label>邮箱<input v-model="email" type="email" data-testid="admin-email" autocomplete="username" placeholder="name@example.com" :disabled="busy" required></label>
      <label>密码<input v-model="password" type="password" data-testid="admin-password" autocomplete="current-password" placeholder="输入密码" :disabled="busy" required></label>
      <p v-if="error" role="alert" data-testid="admin-error" class="error-message">{{ error }}</p>
      <button type="submit" class="primary" data-testid="admin-login" :disabled="busy" @click.prevent="submitLogin">{{ busy ? '正在登录…' : '登录' }}</button>
      <div v-if="oauthProviders.length" class="login-divider">或使用第三方账号</div>
      <div class="oauth-login"><button v-for="name in oauthProviders" :key="name" type="button" :data-testid="`oauth-${name}`" :disabled="busy" @click="submitOAuth(name)">{{ name === 'google' ? 'Google 登录' : 'GitHub 登录' }}</button></div>
      <button v-if="oauthWaiting" type="button" @click="cancelOAuth">取消授权等待</button>
      <small class="muted login-footnote">普通用户请使用提示方舟客户端。关闭页面后需重新登录。</small>
    </form></div>
  </main>
  <main v-else class="admin-layout">
    <aside class="admin-sidebar">
      <div class="brand"><span class="brand-mark">P</span><div>提示方舟<small>管理控制台</small></div></div>
      <p class="nav-label">工作空间</p>
      <nav class="admin-nav" aria-label="管理导航">
        <button data-testid="nav-review" :aria-current="page === 'review' ? 'page' : undefined" @click="openReview"><span class="nav-symbol">▤</span>内容审核</button>
        <button data-testid="nav-users" :aria-current="page === 'users' ? 'page' : undefined" @click="openUsers"><span class="nav-symbol">♧</span>用户</button>
        <p class="nav-label">配置</p>
        <button data-testid="nav-oauth" :aria-current="page === 'oauth' ? 'page' : undefined" @click="openOAuth"><span class="nav-symbol">⌘</span>第三方登录</button>
        <button data-testid="nav-settings" :aria-current="page === 'settings' ? 'page' : undefined" @click="openSettings"><span class="nav-symbol">☷</span>站点设置</button>
      </nav>
      <div class="sidebar-account"><span class="account-avatar">{{ account[0]?.toUpperCase() }}</span><div><strong>管理员</strong><small data-testid="admin-account" :title="account">{{ account }}</small></div><button class="logout-button" data-testid="admin-logout" @click="logout" title="退出登录" aria-label="退出登录">↗</button></div>
    </aside>
    <div class="admin-workspace">
      <header class="workspace-bar"><span>控制台 <span class="breadcrumb-slash">/</span> {{ titles[page] }}</span><span class="environment-label">管理工作空间</span></header>
      <div class="workspace-content">
        <header class="page-heading"><div><h1>{{ titles[page] }}</h1><p>{{ descriptions[page] }}</p></div><button v-if="page === 'review'" @click="openReview" :disabled="loading">刷新列表</button></header>
        <p v-if="error" role="alert" data-testid="admin-error" class="error-banner">{{ error }}</p>
        <OAuthSettings v-if="oauthVisited" v-show="page === 'oauth'" />
        <template v-if="page !== 'oauth'">
          <p v-if="loading" role="status" class="empty-state">正在加载…</p>
          <template v-else>
            <section v-if="page === 'review'" class="panel">
              <div class="panel-heading"><h2>待审核内容</h2><span class="badge">{{ items.length }} 条</span></div>
              <ul data-testid="review-list" class="review-list"><li v-for="item in items" :key="item.id" class="review-row">
                <div class="review-summary"><strong>{{ item.title || item.source_id }}</strong><div><span class="badge">{{ item.kind === 'collection' ? '合集' : '提示词' }}</span><span class="muted">{{ item.source_id }}</span><span class="badge pending">{{ item.status }}</span></div></div>
                <div class="review-actions"><button data-testid="review-reject" :disabled="reviewBusy.includes(item.id)" @click="review(item.id, false)">驳回</button><button class="primary" data-testid="review-approve" :disabled="reviewBusy.includes(item.id)" @click="review(item.id, true)">通过</button></div>
                <details class="review-content" data-testid="review-content"><summary>查看内容快照</summary><template v-if="item.kind === 'collection'"><article v-for="(member, index) in item.members" :key="index"><h3>{{ member.title }}</h3><p class="muted">{{ member.category_id }} · {{ member.model }}</p><pre>{{ member.content }}</pre></article></template><pre v-else>{{ item.content || '未提供正文快照' }}</pre></details>
              </li><li v-if="items.length === 0" class="empty-state"><span class="empty-icon">✓</span><h3>没有待审发布</h3><p>新投稿会出现在这里，你可以查看内容后通过或驳回。</p></li></ul>
            </section>
            <section v-else-if="page === 'users'" class="panel"><div class="panel-heading"><h2>账号列表</h2><span class="badge">{{ users.length }} 位用户</span></div><div class="user-table" data-testid="user-list"><div class="user-table-head"><span>邮箱</span><span>角色</span></div><div v-for="user in users" :key="user.email" class="user-row"><span>{{ user.email }}</span><span class="badge">{{ user.role }}</span></div><p v-if="!users.length" class="empty-state">暂无用户</p></div><p class="panel-note">当前仅提供账号查看，不在此页面修改密码或权限。</p></section>
            <section v-else class="panel" data-testid="settings-panel"><div class="panel-heading"><h2>广场访问</h2></div><div class="setting-row"><div><strong>允许匿名浏览广场</strong><p class="muted">关闭后，访客需要登录才能浏览社区内容。本地库不受影响。</p></div><label class="switch"><input v-model="squarePublic" type="checkbox" data-testid="setting-square-public" aria-label="允许匿名浏览广场" :disabled="settingsBusy"><span></span></label></div><footer class="provider-footer"><p class="success-message" role="status">{{ settingsMessage }}</p><button class="primary" data-testid="settings-save" :disabled="settingsBusy" @click="saveSettings">{{ settingsBusy ? '正在保存…' : '保存设置' }}</button></footer></section>
          </template>
        </template>
      </div>
    </div>
  </main>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue';
import OAuthSettings from './OAuthSettings.vue';
import { approvePublication, getAdminSettings, listAdminUsers, listPendingPublications, putAdminSettings, rejectPublication } from './adminApi.js';
import { getAdminSession, listOAuthProviders, loginAdmin, loginAdminOAuth, logoutAdmin } from './session.js';
import './admin.css';
const titles = { review: '内容审核', users: '用户', oauth: '第三方登录', settings: '站点设置' };
const descriptions = { review: '检查社区投稿，维护广场内容质量。', users: '查看已注册账号与管理员角色。', oauth: '连接登录提供商，让用户使用已有账号登录提示方舟。', settings: '管理社区的访问方式与公开范围。' };
const email = ref(''), password = ref(''), error = ref(''), loggedIn = ref(false), account = ref('');
const items = ref([]), users = ref([]), reviewBusy = ref([]), page = ref('review'), squarePublic = ref(true);
const oauthProviders = ref([]), busy = ref(false), oauthWaiting = ref(false), oauthVisited = ref(false), loading = ref(false), settingsBusy = ref(false), settingsMessage = ref('');
let loginAbort = new AbortController();
onMounted(async () => {
  const session = getAdminSession();
  loggedIn.value = session.loggedIn; account.value = session.email ?? '';
  if (session.loggedIn) await openReview(); else await loadProviders();
});
onUnmounted(() => loginAbort.abort());
async function loadProviders() { oauthProviders.value = ((await listOAuthProviders()).items ?? []).filter(name => name === 'google' || name === 'github'); }
async function submitLogin() {
  if (busy.value) return;
  busy.value = true; error.value = '';
  try { const session = await loginAdmin({ email: email.value, password: password.value }); loggedIn.value = true; account.value = session.email; password.value = ''; await openReview(); }
  catch (caught) { error.value = caught.message; }
  finally { busy.value = false; }
}
async function submitOAuth(provider) {
  if (busy.value) return;
  busy.value = true; oauthWaiting.value = true; error.value = '';
  loginAbort.abort(); loginAbort = new AbortController();
  const signal = loginAbort.signal;
  try { const session = await loginAdminOAuth(provider, { signal }); if (signal.aborted) return; loggedIn.value = true; account.value = session.email; await openReview(); }
  catch (caught) { if (!signal.aborted) error.value = caught.message; }
  finally { if (!signal.aborted) { busy.value = false; oauthWaiting.value = false; } }
}
function cancelOAuth() { loginAbort.abort(); busy.value = false; oauthWaiting.value = false; }
async function logout() {
  cancelOAuth(); loggedIn.value = false; account.value = ''; oauthVisited.value = false; items.value = []; users.value = []; error.value = '';
  if (!await logoutAdmin()) error.value = '本机已退出，但服务端注销失败，请检查网络。';
  await loadProviders();
}
async function openReview() {
  page.value = 'review'; error.value = ''; loading.value = true;
  try { items.value = (await listPendingPublications()).items ?? []; }
  catch (caught) { error.value = caught.message; }
  finally { loading.value = false; }
}
function openOAuth() { page.value = 'oauth'; error.value = ''; oauthVisited.value = true; }
async function openUsers() {
  page.value = 'users'; error.value = ''; loading.value = true;
  try { users.value = (await listAdminUsers()).items ?? []; }
  catch (caught) { error.value = caught.message; }
  finally { loading.value = false; }
}
async function openSettings() {
  page.value = 'settings'; error.value = ''; settingsMessage.value = ''; loading.value = true;
  try { squarePublic.value = Boolean((await getAdminSettings()).square_public); }
  catch (caught) { error.value = caught.message; }
  finally { loading.value = false; }
}
async function saveSettings() {
  if (settingsBusy.value) return;
  settingsBusy.value = true; error.value = ''; settingsMessage.value = '';
  try { squarePublic.value = Boolean((await putAdminSettings(squarePublic.value)).square_public); settingsMessage.value = '设置已保存'; }
  catch (caught) { error.value = caught.message; }
  finally { settingsBusy.value = false; }
}
async function review(id, approved) {
  if (reviewBusy.value.includes(id)) return;
  reviewBusy.value = [...reviewBusy.value, id]; error.value = '';
  try { if (approved) await approvePublication(id); else await rejectPublication(id); items.value = items.value.filter(item => item.id !== id); }
  catch (caught) { error.value = caught.message; }
  finally { reviewBusy.value = reviewBusy.value.filter(value => value !== id); }
}
</script>
