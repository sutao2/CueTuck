<template>
  <div class="update-status" aria-live="polite">
    <p data-testid="update-note">{{ label }}</p>
    <template v-if="['downloading','verifying','ready'].includes(state.phase)">
      <progress :value="state.total ? state.downloaded : undefined" :max="state.total || 1" aria-label="更新下载进度" />
      <small>{{ bytes(state.downloaded) }}<template v-if="state.total"> / {{ bytes(state.total) }}</template></small>
    </template>
    <p v-if="state.error" role="alert">{{ state.error }}</p>
    <button v-if="state.phase === 'available'" class="button" @click="downloadUpdate(channel)">{{ state.error ? '重试下载' : '下载更新' }}</button>
    <template v-if="state.phase === 'ready'">
      <p>更新已下载并通过签名验证。安装将关闭并重新启动应用，请先保存正在编辑的内容。</p>
      <button class="button" :disabled="dirty" @click="confirming = true">安装并重启</button>
      <small v-if="dirty">请先保存设置中未保存的修改。</small>
    </template>
    <small v-if="state.checkedAt">上次检查：{{ state.checkedAt }}</small>
    <details v-if="state.notes"><summary>发行说明 · {{ state.version }}</summary><p class="release-body" data-testid="release-notes">{{ state.notes }}</p></details>
    <div v-if="confirming" class="update-confirm" role="alertdialog" aria-modal="true" aria-label="安装更新" @keydown.esc.stop.prevent="confirming = false" @keydown.tab.prevent="focusOther">
      <section><h3>安装更新并重启？</h3><p>请确认已保存主窗口和启动器中的编辑内容。</p><button ref="cancel" class="button ghost-button" @click="confirming = false">稍后</button><button ref="install" class="button" @click="confirming = false; installUpdate()">安装并重启</button></section>
    </div>
  </div>
</template>
<script setup>
import { computed, nextTick, ref, watch } from 'vue';
import { updateState as state, downloadUpdate, installUpdate } from '../platform/updates.js';
defineProps({ channel: String, dirty: Boolean });
const confirming = ref(false), cancel = ref(null), install = ref(null);
let returnFocus;
watch(confirming, async value => { if(value) { returnFocus = document.activeElement; await nextTick(); cancel.value?.focus(); } else returnFocus?.focus(); });
function focusOther() { (document.activeElement === cancel.value ? install.value : cancel.value)?.focus(); }
const label = computed(() => ({ idle: '可手动检查新版本', checking: '正在检查更新…', current: '当前已是此通道最新版本', available: `发现新版本 ${state.version}`, downloading: `正在下载 ${state.version}`, verifying: '正在验证更新签名…', ready: `更新 ${state.version} 已就绪`, installing: '正在安装，即将重新启动…', error: '检查失败，请重试' }[state.phase]));
const bytes = value => `${((value || 0) / 1048576).toFixed(1)} MB`;
</script>
<style scoped>
.update-status { display:flex; flex-direction:column; align-items:flex-start; gap:12px; padding:20px 0; }
progress { width:100%; height:8px; accent-color:var(--text-primary); }
.release-body { white-space:pre-wrap; overflow-wrap:anywhere; max-height:360px; overflow:auto; }
.update-confirm { position:fixed; inset:0; z-index:2200; background:#0006; display:grid; place-items:center; padding:24px; }
.update-confirm section { background:var(--bg-primary,white); border:1px solid var(--border-color,#ddd); padding:24px; border-radius:16px; max-width:480px; }
.update-confirm button { margin:16px 8px 0 0; }
</style>
