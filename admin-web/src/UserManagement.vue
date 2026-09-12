<template>
  <div class="users-workspace">
    <form class="user-filters" @submit.prevent="load(0)">
      <label class="user-search">搜索用户<input v-model="query" data-testid="users-search" placeholder="邮箱或显示名称" maxlength="254"></label>
      <label>角色<select v-model="role" data-testid="users-role"><option value="">全部角色</option><option v-for="(label, key) in roles" :key="key" :value="key">{{ label }}</option></select></label>
      <label>状态<select v-model="status" data-testid="users-status"><option value="">全部状态</option><option value="active">正常</option><option value="disabled">已停用</option></select></label>
      <button type="submit" :disabled="loading || busy">查询</button>
    </form>
    <p v-if="error" role="alert" class="error-banner">{{ error }} <button :disabled="loading || busy" @click="load(offset)">重新加载列表</button></p>
    <p v-if="message" role="status" class="notice">{{ message }}</p>
    <section class="panel">
      <div class="panel-heading"><h2>账号列表</h2><span class="badge">{{ total }} 位用户</span></div>
      <p v-if="loading" role="status" class="empty-state">正在加载用户…</p>
      <div v-else class="users-table" data-testid="user-list">
        <div class="users-head"><span>用户</span><span>角色</span><span>状态</span><span></span></div>
        <div v-for="user in items" :key="user.email" class="users-row">
          <div class="user-identity"><strong>{{ user.display_name || user.email }}</strong><small v-if="user.display_name">{{ user.email }}</small></div>
          <span class="badge">{{ roles[user.role] || user.role }} · {{ user.role }}</span><span :class="['badge', user.disabled ? 'pending' : 'enabled']">{{ user.disabled ? '已停用' : '正常' }}</span>
          <button data-testid="user-detail" :disabled="busy" @click="showDetail(user.email)">详情</button>
        </div>
        <p v-if="!items.length" class="empty-state">{{ error ? '列表暂不可用，请重试。' : '没有符合条件的用户。' }}</p>
      </div>
      <footer class="users-pagination"><span>{{ total ? offset + 1 : 0 }}–{{ Math.min(offset + items.length, total) }} / {{ total }}</span><button :disabled="loading || busy || offset === 0" @click="load(offset - limit)">上一页</button><button data-testid="users-next" :disabled="loading || busy || offset + limit >= total" @click="load(offset + limit)">下一页</button></footer>
    </section>
    <section v-if="selected" class="panel user-detail" data-testid="user-detail-panel" aria-label="用户详情">
      <div class="panel-heading"><h2>{{ selected }}</h2><button :disabled="busy" @click="closeDetail">关闭详情</button></div>
      <p v-if="detailLoading" class="empty-state">正在加载详情…</p>
      <p v-else-if="detailError" role="alert" class="error-banner">{{ detailError }} <button @click="showDetail(selected)">重试</button></p>
      <template v-else-if="detail">
        <dl class="user-facts"><div><dt>显示名称</dt><dd>{{ detail.display_name || '未设置' }}</dd></div><div><dt>登录来源</dt><dd>{{ [detail.has_password ? '邮箱密码' : '', ...(detail.providers || [])].filter(Boolean).join('、') || '未绑定' }}</dd></div><div><dt>已提交投稿</dt><dd>{{ detail.publication_count }} 条</dd></div><div><dt>有效访问令牌</dt><dd>{{ detail.active_access_count }} 个（不是设备数量）</dd></div><div><dt>真实权益</dt><dd>{{ detail.pro ? 'Pro' : '免费' }}</dd></div><div><dt>模拟权益</dt><dd>{{ detail.mock_pro ? '测试 Pro（持久化模拟权益）' : '未开通' }}</dd></div><div class="bio"><dt>个人简介</dt><dd>{{ detail.bio || '未填写' }}</dd></div></dl>
        <p class="panel-note">不展示私人提示词、云同步正文、密码或令牌内容。</p>
        <div v-if="canManage" class="user-operations"><button data-testid="user-toggle" :disabled="busy" @click="prepare(detail.disabled ? 'enable' : 'disable')">{{ detail.disabled ? '启用账号' : '停用账号' }}</button><button :disabled="busy" data-testid="user-revoke" @click="prepare('revoke_sessions')">撤销全部会话</button><button v-if="!detail.disabled" :disabled="busy" data-testid="user-password-reset" @click="prepare('password_reset')">发送密码重置邮件</button><button v-if="session.permissions.roles" :disabled="busy" data-testid="user-role-change" @click="prepare('role')">调整角色</button></div>
        <p v-else class="panel-note">{{ selected === session.email ? '这是当前账号。修改密码与退出所有设备请前往账号安全。' : '运营管理员不能修改管理人员。请由所有者处理。' }}</p>
        <form v-if="action" class="user-confirm" @submit.prevent="confirmAction">
          <h3>{{ actions[action] }} · {{ selected }}</h3><p class="muted">{{ action === 'password_reset' ? '向账号邮箱发起重置验证。你无法查看验证码或设置对方密码；对方完成重置后旧会话才会撤销。' : action === 'enable' ? '启用后需要重新登录，旧会话不会恢复。' : '此操作会撤销目标账号的全部登录会话。' }}</p>
          <label v-if="action === 'role'">新角色<select v-model="newRole" data-testid="user-new-role" :disabled="busy"><option v-for="(label, key) in roles" :key="key" :value="key">{{ label }}</option></select></label>
          <label v-if="needsReason">操作原因<textarea v-model="reason" data-testid="user-reason" maxlength="1000" :disabled="busy" required placeholder="说明停用或重新启用该账号的依据" /></label>
          <label>你的当前密码<input v-model="currentPassword" type="password" data-testid="user-confirm-password" autocomplete="current-password" :disabled="busy" required></label>
          <p class="muted">用于确认本人操作。尚无本地密码的管理员暂不能执行此操作。</p>
          <p v-if="actionError" role="alert" class="error-message">{{ actionError }}</p>
          <div class="user-operations"><button class="primary" data-testid="user-confirm" :disabled="busy || !currentPassword || needsReason && !reason.trim()">{{ busy ? '正在提交…' : '确认操作' }}</button><button type="button" :disabled="busy" @click="action = ''; currentPassword = ''; reason = ''">取消</button></div>
        </form>
      </template>
    </section>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { listState } from './listState.js';
import { getAdminUser, listAdminUsers, manageAdminUser, resetUserPassword } from './adminApi.js';
import { getAdminSession } from './session.js';
const emit = defineEmits(['busy-change']);
const session = getAdminSession();
const roles = { owner: '所有者', admin: '运营管理员', reviewer: '审核员', user: '普通用户' };
const actions = { disable: '停用账号', enable: '启用账号', revoke_sessions: '撤销全部会话', role: '调整角色', password_reset: '发送密码重置邮件' };
const query = ref(''), role = ref(''), status = ref(''), offset = ref(0), total = ref(0), items = ref([]), loading = ref(false), error = ref(''), message = ref('');
const selected = ref(''), detail = ref(null), detailLoading = ref(false), detailError = ref(''), action = ref(''), newRole = ref('user'), currentPassword = ref(''), actionError = ref(''), busy = ref(false);
const reason = ref('');
const needsReason = computed(() => ['disable','enable'].includes(action.value));
const limit = 25;
let listVersion = 0, detailVersion = 0;
let appliedFilters = { q: '', role: '', status: '' };
const memory = listState('users'), previous = memory.read();
if (previous.filters) { appliedFilters = previous.filters; query.value = appliedFilters.q; role.value = appliedFilters.role; status.value = appliedFilters.status; }
offset.value = previous.offset || 0;
const canManage = computed(() => detail.value && selected.value !== session.email && (session.permissions.roles || detail.value.role === 'user'));
defineExpose({ hasUnsavedChanges: computed(() => Boolean(action.value || currentPassword.value)), isBusy: busy });
async function load(start = 0) {
  if (busy.value) return;
  const version = ++listVersion;
  if (start === 0) appliedFilters = { q: query.value, role: role.value, status: status.value };
  loading.value = true; error.value = '';
  try {
    const result = await listAdminUsers({ ...appliedFilters, offset: start, limit });
    if (version !== listVersion) return;
    items.value = result.items ?? []; total.value = result.total ?? items.value.length; offset.value = start;
    memory.save({ filters: appliedFilters, offset: start });
  } catch (caught) { if (version === listVersion) { error.value = caught.message; items.value = []; total.value = 0; } }
  finally { if (version === listVersion) loading.value = false; }
}
async function showDetail(email) {
  if (busy.value) return;
  const version = ++detailVersion;
  selected.value = email; detail.value = null; detailLoading.value = true; detailError.value = ''; action.value = ''; currentPassword.value = '';
  try { const result = await getAdminUser(email); if (version === detailVersion) detail.value = result; }
  catch (caught) { if (version === detailVersion) detailError.value = caught.message; }
  finally { if (version === detailVersion) detailLoading.value = false; }
}
function closeDetail() { ++detailVersion; selected.value = ''; detail.value = null; action.value = ''; currentPassword.value = ''; }
function prepare(value) { action.value = value; newRole.value = detail.value.role; currentPassword.value = ''; reason.value = ''; actionError.value = ''; message.value = ''; }
async function confirmAction() {
  if (busy.value || !canManage.value || !currentPassword.value) return;
  if (needsReason.value && !reason.value.trim()) { actionError.value = '请填写操作原因'; return; }
  busy.value = true; emit('busy-change', true); actionError.value = '';
  let completed = false;
  try {
    if (action.value === 'password_reset') await resetUserPassword(selected.value, {current_password:currentPassword.value});
    else await manageAdminUser(selected.value, { action: action.value, current_password: currentPassword.value, ...(needsReason.value ? { reason: reason.value.trim() } : {}), ...(action.value === 'role' ? { role: newRole.value } : {}) });
    message.value = action.value === 'password_reset' ? '重置请求已受理，邮件状态请由所有者在邮件服务中核对。' : `${actions[action.value]}已完成`; currentPassword.value = ''; action.value = ''; completed = true;
  } catch (caught) { actionError.value = caught.message; }
  finally { busy.value = false; emit('busy-change', false); }
  if (completed) await Promise.all([load(offset.value), showDetail(selected.value)]);
}
onMounted(() => load(offset.value));
onUnmounted(() => { ++listVersion; ++detailVersion; currentPassword.value = ''; });
</script>

<style scoped>
.users-workspace { display: grid; gap: 20px; }
.user-filters { display: flex; align-items: end; gap: 12px; flex-wrap: wrap; }
.user-search { flex: 1; min-width: 180px; }
select { display: block; width: 100%; margin-top: 8px; padding: 10px 28px 10px 12px; border: 1px solid var(--line-strong); border-radius: 7px; color: inherit; background: white; font: inherit; }
.users-head, .users-row { display: grid; grid-template-columns: minmax(0, 1fr) 145px 75px 62px; gap: 12px; align-items: center; padding: 15px 24px; border-bottom: 1px solid var(--line); }
.users-head { font-size: 12px; color: #818b91; background: #fafbfc; }
.user-identity { overflow-wrap: anywhere; } .user-identity strong { font-size: 13px; font-weight: 500; } .user-identity small { display: block; color: #818b91; }
.users-row .badge { justify-self: start; }
.users-pagination { display: flex; align-items: center; justify-content: end; gap: 12px; padding: 16px 24px; color: #818b91; font-size: 12px; }
.user-detail .panel-heading h2 { overflow-wrap: anywhere; min-width: 0; } .user-detail .panel-heading button { flex-shrink: 0; }
.user-facts { display: grid; grid-template-columns: 1fr 1fr; gap: 22px; padding: 24px; margin: 0; }
.user-facts dt { color: #818b91; font-size: 12px; } .user-facts dd { margin: 5px 0 0; overflow-wrap: anywhere; } .bio { grid-column: 1 / -1; }
.panel-note { padding: 0 24px 20px; font-size: 12px; }
.user-operations { display: flex; gap: 10px; flex-wrap: wrap; padding: 0 24px 24px; }
.user-confirm { border-top: 1px solid var(--line); padding: 24px; display: grid; gap: 14px; background: #fafbfc; }
.user-confirm .user-operations { padding: 0; } .user-confirm label { max-width: 440px; } .user-confirm h3 { overflow-wrap: anywhere; }
@media(max-width: 760px) { .users-head { display: none; } .users-row { grid-template-columns: 1fr auto; padding: 18px; } .user-identity { grid-column: 1 / -1; } .users-row button { grid-column: 2; grid-row: 2 / 4; } .users-row .badge { justify-self: start; } .user-facts { grid-template-columns: 1fr; } .users-pagination { padding: 14px; } }
</style>
