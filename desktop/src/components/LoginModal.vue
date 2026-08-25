<template>
  <div class="modal-layer" data-testid="login-modal">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section class="modal create-modal" role="dialog" aria-modal="true" aria-labelledby="login-title">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">ACCOUNT</p>
          <h2 id="login-title">登录</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="create-body">
        <p data-testid="login-reason">{{ reason }}</p>
        <label class="field">
          <span>邮箱</span>
          <input v-model="email" type="email" data-testid="login-email" autocomplete="username">
        </label>
        <label class="field">
          <span>密码</span>
          <input v-model="password" type="password" data-testid="login-password" autocomplete="current-password">
        </label>
        <p v-if="error" data-testid="login-error">{{ error }}</p>
        <p v-if="busy" data-testid="oauth-wait">正在等待浏览器授权</p>
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
      </div>
      <footer class="modal-footer">
        <span class="create-location">Refresh 只写入系统钥匙串，不会进浏览器存储。</span>
        <div class="modal-actions">
          <button type="button" class="button ghost-button" @click="$emit('cancel')">取消</button>
          <button type="button" class="button primary-button" data-testid="login-submit" :disabled="busy" @click="submit">
            登录
          </button>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from "vue";
import { listOAuthProviders, loginOAuthSession, loginSession } from "../platform/session.js";

defineProps({
  reason: { type: String, required: true },
});
const emit = defineEmits(["cancel", "success"]);
const email = ref("");
const password = ref("");
const error = ref("");
const providers = ref([]);
const busy = ref(false);
const abort = new AbortController();

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

async function submit() {
  if (busy.value) return;
  error.value = "";
  try {
    await loginSession({ email: email.value, password: password.value });
    emit("success");
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function submitOAuth(provider) {
  if (busy.value) return;
  error.value = "";
  busy.value = true;
  try {
    await loginOAuthSession(provider, { signal: abort.signal });
    emit("success");
  } catch (caught) {
    if (abort.signal.aborted) return;
    error.value = caught instanceof Error ? caught.message : String(caught);
    busy.value = false;
  }
}
</script>
