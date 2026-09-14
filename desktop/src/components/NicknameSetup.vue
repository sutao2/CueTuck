<template>
  <Teleport to="body">
    <div class="nickname-backdrop">
      <section v-dialog-focus="() => {}" class="nickname-dialog" role="dialog" aria-modal="true" aria-labelledby="nickname-title" :aria-busy="busy">
        <div class="nickname-avatar" aria-hidden="true">{{ name.trim().slice(0, 1) || '你' }}</div>
        <h2 id="nickname-title">让大家认识你</h2>
        <p class="nickname-description">设置一个公开昵称，用于广场中的作者署名。之后可在「账号与广场」修改。</p>
        <p v-if="loading" role="status">正在读取账号资料…</p>
        <form v-else-if="loaded" @submit.prevent="save">
          <label class="field">
            <span>昵称</span>
            <input ref="nameInput" v-model="name" data-testid="nickname-input" autocomplete="nickname" placeholder="你希望大家怎么称呼你？" required :disabled="busy" aria-describedby="nickname-note nickname-error">
          </label>
          <p id="nickname-note" class="nickname-note">昵称会公开显示，请勿填写邮箱或其他私人信息。</p>
          <p id="nickname-error" role="alert">{{ error }}</p>
          <button class="button primary-button nickname-save" type="submit" :disabled="busy || !name.trim()" data-testid="nickname-save">{{ saving ? '正在保存…' : '保存并继续' }}</button>
        </form>
        <div v-else>
          <p role="alert">{{ error }}</p>
          <button class="button" :disabled="busy" @click="load">重新读取</button>
        </div>
        <button class="button ghost-button nickname-exit" type="button" :disabled="busy" @click="emit('logout')">退出登录，使用本地库</button>
      </section>
    </div>
  </Teleport>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { getMe, putMe } from '../platform/session.js';
import { vDialogFocus } from '../lib/dialogFocus.js';

const emit = defineEmits(['ready', 'logout']);
const name = ref(''), bio = ref(''), error = ref('');
const loading = ref(true), loaded = ref(false), saving = ref(false), nameInput = ref(null);
const busy = computed(() => loading.value || saving.value);
let disposed = false;
onUnmounted(() => { disposed = true; });
onMounted(load);

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const profile = await getMe();
    if (disposed) return;
    name.value = (profile.display_name ?? profile.displayName ?? '').trim();
    bio.value = profile.bio ?? '';
    if (name.value) { emit('ready'); return; }
    loaded.value = true;
  } catch (caught) {
    if (!disposed) error.value = `读取资料失败：${caught.message || caught}`;
  } finally {
    loading.value = false;
    await nextTick();
    if (!disposed) nameInput.value?.focus();
  }
}

async function save() {
  if (busy.value || !name.value.trim()) return;
  saving.value = true;
  error.value = '';
  try {
    const profile = await putMe({ displayName: name.value.trim(), bio: bio.value });
    if (disposed) return;
    if (!(profile.display_name ?? profile.displayName ?? '').trim()) throw Error('昵称未保存，请重试');
    emit('ready');
  } catch (caught) {
    if (!disposed) error.value = `保存失败：${caught.message || caught}`;
  } finally { saving.value = false; }
}
</script>

<style scoped>
.nickname-backdrop { position: fixed; inset: 0; z-index: 2000; display: grid; place-items: center; padding: 24px; background: #0006; }
.nickname-dialog { width: min(420px, 100%); max-height: calc(100dvh - 48px); overflow: auto; box-sizing: border-box; padding: 32px; border: 1px solid var(--line); border-radius: 20px; background: var(--bg); color: var(--text); box-shadow: 0 20px 70px #0003; }
.nickname-avatar { display: grid; place-items: center; width: 48px; height: 48px; margin-bottom: 20px; border-radius: 50%; background: var(--hover); font-size: 20px; font-weight: 600; }
h2 { margin: 0 0 12px; font-size: 24px; }
.nickname-description { margin: 0 0 24px; color: var(--muted); line-height: 1.7; font-size: 14px; }
.nickname-note { color: var(--muted); font-size: 12px; line-height: 1.6; }
[role="alert"] { color: var(--danger, #b42318); font-size: 13px; }
.nickname-save, .nickname-exit { width: 100%; justify-content: center; }
.nickname-exit { margin-top: 12px; font-size: 13px; }
</style>
