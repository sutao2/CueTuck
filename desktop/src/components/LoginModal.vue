<template>
  <div class="modal-layer" data-testid="login-modal">
    <div class="modal-backdrop" @click="close"></div>
    <section v-dialog-focus="close" class="modal login-modal" role="dialog" aria-modal="true" aria-labelledby="login-title" aria-describedby="login-description" :aria-busy="busy">
      <header class="modal-header">
        <div>
          <div class="login-mark" aria-hidden="true"><AppIcon name="library" /></div>
          <h2 id="login-title">登录提示方舟</h2>
          <p id="login-description" class="login-description" data-testid="login-reason">{{ reason && reason !== '登录' ? reason : '同步你的灵感，收藏与分享好用的提示词。' }}</p>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" :disabled="pending === 'email'" @click="close">×</button>
      </header>
      <form class="create-body login-form" @submit.prevent="submit">
        <label class="field">
          <span>邮箱</span>
          <input v-model="email" type="email" data-testid="login-email" autocomplete="username" placeholder="you@example.com" required :disabled="busy" :aria-invalid="error ? true : undefined" aria-describedby="login-error">
        </label>
        <label class="field">
          <span>密码</span>
          <input v-model="password" type="password" data-testid="login-password" autocomplete="current-password" placeholder="输入账号密码" required :disabled="busy" :aria-invalid="error ? true : undefined" aria-describedby="login-error">
        </label>
        <p v-if="error" id="login-error" role="alert" data-testid="login-error">{{ error }}</p>
        <button type="submit" class="button primary-button login-submit" data-testid="login-submit" :disabled="busy">{{ pending === 'email' ? '正在登录…' : '登录' }}</button>
        <p v-if="pending === 'oauth'" role="status" class="use-hint" data-testid="oauth-wait">正在等待浏览器授权，可关闭此窗口取消。</p>
        <div v-if="providers.length" class="login-divider"><span>或使用以下方式</span></div>
        <div v-if="providers.length" class="oauth-row">
          <button
            v-for="name in providers"
            :key="name"
            type="button"
            class="button ghost-button"
            :data-testid="`oauth-${name}`"
            :disabled="busy"
            @click="submitOAuth(name)"
          >
            {{ name === "google" ? "Google 登录" : "GitHub 登录" }}
          </button>
        </div>
        <p class="login-privacy" data-testid="login-token-note"><AppIcon name="shield" />{{ tokenNote }}</p>
        <button type="button" class="login-later" :disabled="pending === 'email'" @click="close">暂不登录，继续使用本地库</button>
      </form>
    </section>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import AppIcon from "./AppIcon.vue";
import { vDialogFocus } from "../lib/dialogFocus.js";
import { listOAuthProviders, loginOAuthSession, loginSession } from "../platform/session.js";

defineProps({
  reason: { type: String, required: true },
});
const emit = defineEmits(["cancel", "success"]);
const email = ref("");
const password = ref("");
const error = ref("");
const providers = ref([]);
const pending = ref("");
const busy = computed(() => Boolean(pending.value));
const abort = new AbortController();

function usesSystemKeychain() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

const tokenNote = usesSystemKeychain()
  ? "Refresh 只写入系统钥匙串，不会进浏览器存储。"
  : "Refresh 不进 Web Storage。浏览器预览不写入系统钥匙串。";

onMounted(async () => {
  try {
    const payload = await listOAuthProviders();
    providers.value = (payload.items ?? []).filter(
      (name) => name === "google" || name === "github",
    );
  } catch {
    providers.value = [];
  }
});

onUnmounted(() => abort.abort());

function close() {
  if (pending.value === 'email') return;
  abort.abort();
  emit('cancel');
}

async function submit() {
  if (busy.value) return;
  error.value = "";
  pending.value = 'email';
  try {
    await loginSession({ email: email.value.trim(), password: password.value });
    emit("success");
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally { pending.value = ''; }
}

async function submitOAuth(provider) {
  if (busy.value) return;
  error.value = "";
  pending.value = 'oauth';
  try {
    await loginOAuthSession(provider, { signal: abort.signal });
    emit("success");
  } catch (caught) {
    if (abort.signal.aborted) return;
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally { pending.value = ''; }
}
</script>
