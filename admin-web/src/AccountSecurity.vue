<template>
  <section class="security-page">
    <p v-if="loading" class="empty-state" role="status">正在读取账号安全信息…</p>
    <div v-else-if="loadError" class="panel empty-state"><p role="alert">{{ loadError }}</p><button @click="load">重新加载</button></div>
    <template v-else>
      <section class="panel">
        <header class="panel-heading"><h2>修改密码</h2><span class="badge">{{ info.has_password ? '已设置密码' : '第三方登录账号' }}</span></header>
        <form v-if="info.has_password" class="password-form" @submit.prevent="changePassword">
          <p class="muted">修改后将退出所有设备，包括当前管理台。请使用新密码重新登录。</p>
          <fieldset :disabled="Boolean(busy)">
            <label>当前密码<input v-model="currentPassword" type="password" autocomplete="current-password" required data-testid="security-current-password"></label>
            <label>新密码<input v-model="newPassword" type="password" autocomplete="new-password" required aria-describedby="password-help" data-testid="security-new-password"></label>
            <small id="password-help" class="muted">12–128 个字符，请勿与当前密码相同。</small>
            <label>确认新密码<input v-model="confirmation" type="password" autocomplete="new-password" required data-testid="security-confirm-password"></label>
          </fieldset>
          <p v-if="passwordError" class="error-message" role="alert">{{ passwordError }}</p>
          <button class="primary" type="submit" :disabled="Boolean(busy)" data-testid="security-password-save">{{ busy === 'password' ? '正在修改…' : '修改密码并退出' }}</button>
        </form>
        <p v-else class="panel-note">此账号尚未设置本地密码，请继续使用第三方登录。密码找回与设置流程将在后续账号功能中提供。</p>
      </section>
      <section class="panel sessions-panel">
        <header class="panel-heading"><h2>登录会话</h2><span class="badge">{{ info.active_access_count }} 个有效访问令牌</span></header>
        <div class="session-actions">
          <p class="muted">撤销此账号所有设备的登录凭据，不修改密码，也不影响其他用户。令牌数量不等同于物理设备数量。</p>
          <button v-if="!confirmRevoke" type="button" :disabled="Boolean(busy)" data-testid="security-revoke" @click="confirmRevoke = true">退出所有设备</button>
          <div v-else class="revoke-confirmation" role="group" aria-label="确认退出所有设备">
            <p>当前管理台也会退出，确定继续吗？</p>
            <div><button :disabled="Boolean(busy)" @click="confirmRevoke = false">取消</button><button class="danger" :disabled="Boolean(busy)" @click="revoke" data-testid="security-revoke-confirm">{{ busy === 'revoke' ? '正在退出…' : '确认退出所有设备' }}</button></div>
          </div>
          <p v-if="revokeError" class="error-message" role="alert">{{ revokeError }}</p>
        </div>
      </section>
    </template>
  </section>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue';
import { getAccountSecurity, changeAdminPassword, revokeAdminSessions } from './adminApi.js';
const emit = defineEmits(['signed-out', 'busy-change']);
const loading = ref(true), loadError = ref(''), info = ref(null), busy = ref('');
const currentPassword = ref(''), newPassword = ref(''), confirmation = ref('');
const passwordError = ref(''), revokeError = ref(''), confirmRevoke = ref(false);
defineExpose({ hasUnsavedChanges: computed(() => Boolean(currentPassword.value || newPassword.value || confirmation.value)), isBusy: computed(() => Boolean(busy.value)) });
async function load() {
  loading.value = true; loadError.value = '';
  try { info.value = await getAccountSecurity(); }
  catch (error) { loadError.value = error.message; }
  finally { loading.value = false; }
}
function signedOut(message) {
  currentPassword.value = ''; newPassword.value = ''; confirmation.value = '';
  emit('signed-out', message);
}
async function changePassword() {
  if (busy.value) return;
  passwordError.value = '';
  const length = [...newPassword.value].length;
  if (!currentPassword.value || length < 12 || length > 128 || new TextEncoder().encode(newPassword.value).length > 512) { passwordError.value = '请填写当前密码，新密码须为 12–128 个字符'; return; }
  if (newPassword.value === currentPassword.value) { passwordError.value = '新密码不能与当前密码相同'; return; }
  if (newPassword.value !== confirmation.value) { passwordError.value = '两次输入的新密码不一致'; return; }
  busy.value = 'password'; emit('busy-change', true);
  try {
    await changeAdminPassword(currentPassword.value, newPassword.value);
    signedOut('密码已修改，所有设备已退出，请使用新密码登录。');
  } catch (error) { passwordError.value = error.message; }
  finally { busy.value = ''; emit('busy-change', false); }
}
async function revoke() {
  if (busy.value || !confirmRevoke.value) return;
  busy.value = 'revoke'; revokeError.value = ''; emit('busy-change', true);
  try { await revokeAdminSessions(); signedOut('所有设备已退出，请重新登录。'); }
  catch (error) { revokeError.value = error.message; }
  finally { busy.value = ''; emit('busy-change', false); }
}
onMounted(load);
</script>

<style scoped>
.security-page { max-width: 760px; }
.password-form, .session-actions { padding: 24px; display: grid; gap: 18px; }
.password-form fieldset { display: grid; gap: 16px; max-width: 480px; }
.password-form > button, .session-actions > button { justify-self: start; }
.password-form .muted, .session-actions .muted { font-size: 13px; }
.sessions-panel { margin-top: 24px; }
.revoke-confirmation { padding: 16px; background: #fff7f5; border: 1px solid #efdcd6; border-radius: 8px; }
.revoke-confirmation > div { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 14px; }
.danger { color: #a34545; border-color: #dfb8ae; }
</style>
