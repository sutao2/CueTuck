<template>
  <div class="oauth-settings">
    <div class="notice"><span class="notice-icon">i</span><div><strong>一次配置，所有客户端生效</strong><p>先在提供商控制台创建 OAuth 应用，再将凭据填入这里。保存后立即生效，无需重启服务。</p></div></div>
    <p v-if="loading" role="status" class="empty-state">正在读取登录配置…</p>
    <div v-else-if="loadError" class="empty-state"><p role="alert">{{ loadError }}</p><button @click="load">重新加载</button></div>
    <template v-else>
      <form v-for="item in items" :key="item.provider" class="provider-card" :data-testid="`provider-${item.provider}`" @submit.prevent="save(item)">
        <header class="provider-header">
          <span class="provider-logo" :class="item.provider">{{ item.provider === 'google' ? 'G' : 'GH' }}</span>
          <div class="provider-title"><h2>{{ names[item.provider] }}</h2><p>{{ item.provider === 'google' ? '使用 Google 账号登录' : '使用 GitHub 账号登录' }}</p></div>
          <span class="badge" :class="{ enabled: item.savedEnabled }">{{ item.savedEnabled ? '已启用' : '未启用' }}</span>
        </header>
        <fieldset :disabled="item.busy">
          <div class="provider-enable"><div><strong>启用 {{ names[item.provider] }} 登录</strong><p>关闭后将不再展示此登录方式，已保存的凭据会保留。</p></div><label class="switch"><input type="checkbox" v-model="item.enabled" :aria-label="`启用 ${names[item.provider]} 登录`"><span></span></label></div>
          <div class="provider-fields">
            <label>Client ID<input v-model="item.client_id" :data-testid="`${item.provider}-client-id`" :aria-label="`${names[item.provider]} Client ID`" autocomplete="off" placeholder="粘贴应用的 Client ID" maxlength="1024"></label>
            <label>Client Secret <span class="field-note">{{ item.secret_configured ? '已安全保存 · 留空保持不变' : '尚未设置' }}</span><input type="password" v-model="item.client_secret" :data-testid="`${item.provider}-secret`" :aria-label="`${names[item.provider]} Client Secret`" autocomplete="new-password" :placeholder="item.secret_configured ? '输入新密钥以替换' : '粘贴应用的 Client Secret'" maxlength="4096"></label>
            <div class="wide-field"><label :for="`${item.provider}-callback`">授权回调地址</label><div class="input-action"><input :id="`${item.provider}-callback`" v-model="item.redirect_uri" :data-testid="`${item.provider}-redirect`" :aria-describedby="`${item.provider}-callback-help`" autocomplete="off" spellcheck="false" placeholder="https://api.example.com/v1/session/oauth/callback" maxlength="2048"><button type="button" @click="copyCallback(item)" :aria-label="`复制 ${names[item.provider]} 回调地址`">复制</button></div><small :id="`${item.provider}-callback-help`">需与提供商控制台登记的地址完全一致。这是后端接口，不是管理端首页。</small></div>
          </div>
        </fieldset>
        <footer class="provider-footer"><div><a :href="guides[item.provider]" target="_blank" rel="noopener noreferrer">配置指引 ↗</a><span class="source-label">{{ item.source === 'environment' ? '当前来自环境变量' : item.source === 'database' ? '管理端配置' : '尚未配置' }}</span></div><button class="primary" type="submit" :disabled="item.busy" :data-testid="`${item.provider}-save`">{{ item.busy ? '正在保存…' : '保存配置' }}</button></footer>
        <p v-if="item.error" role="alert" class="form-message error-message">{{ item.error }}</p>
        <p v-if="item.message" role="status" class="form-message success-message">{{ item.message }}</p>
      </form>
      <p class="security-note">密钥仅在后端加密保存，不会回显或写入浏览器存储。保存配置不代表已通过真实授权；请在登录页完成一次授权验证。第三方账号不会自动获得管理员权限。</p>
    </template>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue';
import { getOAuthSettings, putOAuthSettings } from './adminApi.js';
const names = { google: 'Google', github: 'GitHub' };
const guides = { google: 'https://developers.google.com/identity/protocols/oauth2/web-server', github: 'https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/creating-an-oauth-app' };
const items = ref([]), loading = ref(true), loadError = ref('');
async function load() {
  loading.value = true; loadError.value = '';
  try { items.value = (await getOAuthSettings()).items.map(item => ({ ...item, savedEnabled: item.enabled, client_secret: '', busy: false, error: '', message: '' })); }
  catch (error) { loadError.value = error.message; }
  finally { loading.value = false; }
}
async function save(item) {
  if (item.busy) return;
  item.error = ''; item.message = '';
  if (item.enabled && (!item.client_id.trim() || (!item.client_secret.trim() && !item.secret_configured) || !item.redirect_uri.trim())) { item.error = '启用前请填写 Client ID、Client Secret 和回调地址'; return; }
  item.busy = true;
  try {
    const saved = await putOAuthSettings(item.provider, { enabled: item.enabled, client_id: item.client_id.trim(), client_secret: item.client_secret.trim(), redirect_uri: item.redirect_uri.trim() });
    Object.assign(item, saved, { savedEnabled: saved.enabled, client_secret: '', message: saved.enabled ? '配置已保存并启用，请在登录页完成真实授权验证。' : '配置已保存，此登录方式已关闭。' });
  } catch (error) { item.error = error.message; }
  finally { item.busy = false; }
}
async function copyCallback(item) {
  item.error = ''; item.message = '';
  try { await navigator.clipboard.writeText(item.redirect_uri); item.message = '回调地址已复制'; }
  catch { item.error = '复制失败，请手动选中回调地址复制'; }
}
onMounted(load);
</script>
