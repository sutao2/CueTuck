<template>
  <slot v-if="!native || status && !status.locked" />
  <main v-else-if="!launcher" class="lock-page">
    <header data-tauri-drag-region><span v-if="status?.locked" class="lock-indicator" :class="{windows}" role="img" :aria-label="tr('已锁定')"><AppIcon name="lock" /></span><WindowsWindowControls v-if="windows" /></header>
    <section class="lock-card" aria-labelledby="unlock-title">
      <img class="lock-brand" src="../assets/app-icon.png" alt="CueTuck" width="56" height="56">
      <h1 id="unlock-title">{{ tr(status?.locked ? '解锁工作台' : '正在检查应用锁') }}</h1>
      <template v-if="status?.locked">
        <template v-if="status.device && !passwordFirst">
          <button class="button primary-button device-unlock" :disabled="busy || !available" @click="unlockWithDevice"><AppIcon name="unlock" />{{ tr('使用 {0} 解锁', [deviceName]) }}</button>
          <button type="button" class="unlock-alternative" :disabled="busy" @click="showPassword">{{ tr('使用解锁口令') }}</button>
        </template>
        <template v-else>
          <form class="unlock-entry" @submit.prevent="perform({action:'unlock', password:current})">
            <input ref="unlockInput" v-model="current" type="password" required autocomplete="current-password" :aria-label="tr('解锁口令')" :placeholder="tr('解锁口令')" :aria-describedby="error ? 'unlock-error' : undefined" :disabled="busy">
            <button class="button primary-button" type="submit" :disabled="busy">{{ tr('解锁') }}</button>
          </form>
          <button v-if="status.device && available" type="button" class="unlock-alternative" :disabled="busy" @click="unlockWithDevice">{{ tr('使用 {0} 解锁', [deviceName]) }}</button>
        </template>
      </template>
      <button v-else-if="error" class="button" @click="initialize">{{ tr('重试') }}</button>
      <p v-if="error" id="unlock-error" role="alert">{{ tr(error) }}</p>
    </section>
  </main>

  <dialog v-if="manage && status && !status.locked" ref="dialog" class="lock-dialog" aria-labelledby="lock-title" @keydown.stop @cancel="cancelSettings">
    <div class="lock-heading"><h2 id="lock-title">{{ tr('应用锁') }}</h2><button class="button" :disabled="busy" @click="closeSettings">{{ tr('关闭') }}</button></div>
    <template v-if="manage==='discard'">
      <p>{{ tr('资料尚未保存。锁定将丢弃本次编辑。') }}</p>
      <button class="button" @click="closeSettings">{{ tr('继续编辑') }}</button>
      <button class="button primary-button" :disabled="busy" @click="perform({action:'lock'})">{{ tr('丢弃并锁定') }}</button>
      <p v-if="error" role="alert">{{ tr(error) }}</p>
    </template>
    <template v-else>
    <p>{{ tr('启动时需要解锁，也可以点击工作台右上角的小锁立即锁定。') }}</p>
    <p v-if="!status.enabled">{{ tr('请记住解锁口令。启用期间，本机 MCP 工具将暂停读取。') }}</p>
    <form @submit.prevent="saveSettings">
      <label v-if="status.enabled">{{ tr('操作') }}
        <SearchableSelect v-model="mode" :aria-label="tr('操作')" :options="actions" :searchable="false" :disabled="busy" @change="clearForm" />
      </label>
      <label v-if="status.enabled">{{ tr('当前口令') }}<input v-model="current" type="password" required autocomplete="current-password" :disabled="busy"></label>
      <template v-if="!status.enabled || mode==='change'">
        <label>{{ tr('新口令（至少 8 个字符）') }}<input v-model="password" type="password" required minlength="8" autocomplete="new-password" :disabled="busy"></label>
        <label>{{ tr('再次输入新口令') }}<input v-model="confirmation" type="password" required autocomplete="new-password" :disabled="busy"></label>
      </template>
      <label v-if="!status.enabled && available" class="device-choice"><input type="checkbox" v-model="enableDevice" :disabled="busy">{{ tr('启用 {0} 解锁', [deviceName]) }}</label>
      <p v-if="status.enabled && !available && !status.device">{{ tr('此设备当前未提供系统解锁。') }}</p>
      <p v-if="error" role="alert">{{ tr(error) }}</p>
      <button class="button primary-button" :disabled="busy">{{ tr(status.enabled ? '保存更改' : '启用应用锁') }}</button>
    </form>
    </template>
  </dialog>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, provide, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { tr } from '../platform/interfaceLanguage.js';
import AppIcon from './AppIcon.vue';
import SearchableSelect from './SearchableSelect.vue';
import WindowsWindowControls from './WindowsWindowControls.vue';

const props = defineProps({ launcher: Boolean });
const native = Boolean(window.__TAURI_INTERNALS__);
const windows = /Windows/i.test(navigator.userAgent);
const deviceName = windows ? 'Windows Hello' : 'Touch ID';
const status = ref(null), available = ref(false), manage = ref(false), busy = ref(false), error = ref('');
const current = ref(''), password = ref(''), confirmation = ref(''), mode = ref('change'), enableDevice = ref(false);
const actions = computed(()=>[
  {value:'change',label:tr('更改解锁口令')},
  ...(available.value || status.value?.device ? [{value:'device',label:tr(status.value?.device ? '关闭 {0} 解锁' : '启用 {0} 解锁',[deviceName])}] : []),
  {value:'disable',label:tr('关闭应用锁')},
]);
const unlockInput = ref(null), dialog = ref(null);
const pendingDevice = ref(false), passwordFirst = ref(false);
let stop, subscribing = false, revision = 0, disposed = false;
function clearForm() { current.value=''; password.value=''; confirmation.value=''; error.value=''; }
function closeSettings() { dialog.value?.close(); manage.value=false; clearForm(); }
function cancelSettings(event) { if (busy.value) event.preventDefault(); else closeSettings(); }
async function probeDevice() {
  try { available.value = await invoke('app_lock_device_available'); } catch { available.value=false; }
}
async function showPassword() {
  pendingDevice.value=false; passwordFirst.value=true;
  await nextTick(); unlockInput.value?.focus();
}
async function unlockWithDevice() {
  pendingDevice.value=false; passwordFirst.value=false; current.value='';
  await perform({action:'unlock_device'});
  if (status.value?.locked) await showPassword();
}
function tryPreferredDevice() {
  if (pendingDevice.value && available.value && !busy.value && status.value?.locked && !document.hidden && document.hasFocus()) {
    void unlockWithDevice();
  }
}
watch([pendingDevice, available, busy], tryPreferredDevice, {flush:'post'});
async function refresh() {
  const token = ++revision;
  try {
    const next = await invoke('app_lock_status');
    if (typeof next?.locked !== 'boolean') throw new Error();
    if (token !== revision || disposed) return;
    const entering = next.locked && !status.value?.locked;
    status.value = next;
    if (!next.locked) pendingDevice.value=false;
    if (entering) {
      closeSettings();
      if (!props.launcher) {
        available.value=false; passwordFirst.value=false; pendingDevice.value=next.device;
        if (next.device) await probeDevice();
        await nextTick();
        if (!available.value && status.value?.locked) await showPassword();
      }
    }
  } catch {
    if (token === revision && !disposed) {
      closeSettings(); status.value=null; error.value='无法读取应用锁状态';
    }
  }
}
async function initialize() {
  if (!native) return;
  if (!stop) {
    if (subscribing) return;
    subscribing=true;
    try {
      const off = await listen('app-lock-changed', refresh);
      if (disposed) { off(); return; }
      stop = off;
    } catch { error.value='无法读取应用锁状态'; return; }
    finally { subscribing=false; }
  }
  await refresh();
}
watch(() => status.value === null && error.value === '无法读取应用锁状态', (unavailable, _, onCleanup) => {
  if (!unavailable) return;
  const timer = setTimeout(initialize, 400);
  onCleanup(() => clearTimeout(timer));
});
async function openSettings(view='settings') {
  clearForm(); mode.value='change'; enableDevice.value=false; manage.value=view;
  await nextTick(); dialog.value?.showModal(); void probeDevice();
}
async function perform(request) {
  if (busy.value) return;
  busy.value=true; error.value='';
  try {
    await invoke('app_lock_action', {request});
    closeSettings();
    await refresh();
  } catch (e) {
    current.value=''; error.value=String(e.message || e);
    if (!manage.value && !status.value?.locked) {
      const message=error.value; await openSettings(); error.value=message;
    }
  } finally { busy.value=false; }
}
function saveSettings() {
  if (!status.value.enabled || mode.value==='change') {
    if (password.value !== confirmation.value) { error.value='两次输入的口令不一致'; return; }
    return perform(status.value.enabled ? {action:'change',current:current.value,password:password.value} : {action:'enable',password:password.value,device:enableDevice.value});
  }
  return perform(mode.value==='device' ? {action:'set_device',current:current.value,enabled:!status.value.device} : {action:'disable',current:current.value});
}
if (native && !props.launcher) provide('appLock', {
  enabled:computed(()=>Boolean(status.value?.enabled)), openSettings, lock:dirty=>dirty===true ? openSettings('discard') : perform({action:'lock'}),
});
onMounted(()=>{ if (native) { void initialize(); window.addEventListener('focus', onFocus); document.addEventListener('visibilitychange', tryPreferredDevice); } });
function onFocus() { if (stop) void refresh().then(tryPreferredDevice); }
onUnmounted(()=>{ disposed=true; ++revision; stop?.(); window.removeEventListener('focus',onFocus); document.removeEventListener('visibilitychange',tryPreferredDevice); });
</script>

<style scoped>
.lock-page { height:100%; overflow:auto; display:flex; flex-direction:column; padding:calc(var(--titlebar-height) + 16px) 16px 32px; background:var(--sidebar); color:var(--text); }
.lock-page header { position:fixed; inset:0 0 auto; height:var(--titlebar-height); }
.lock-card { flex:none; display:grid; width:min(100%, 352px); margin:auto; padding:32px 24px; border:1px solid var(--line); border-radius:20px; background:var(--surface); box-shadow:0 2px 8px rgba(0,0,0,.04); }
.lock-brand { justify-self:center; }
.lock-card h1 { margin:12px 0 24px; color:var(--text); font-size:20px; font-weight:600; line-height:28px; text-align:center; text-wrap:balance; }
.unlock-entry { display:flex; align-items:center; gap:4px; padding:4px; border:1px solid var(--control-border); border-radius:var(--radius-control); }
.unlock-entry:focus-within { border-color:var(--muted); box-shadow:0 0 0 1px var(--muted); }
@media (forced-colors:active) {
  .unlock-entry:focus-within { outline:2px solid Highlight; }
}
.unlock-entry input { flex:1; min-width:0; padding:6px 8px; border:0; outline:0; background:transparent; color:var(--text); font:inherit; }
.unlock-entry input::placeholder { color:var(--muted); }
.unlock-entry .button { min-height:32px; padding:4px 12px; border-radius:8px; }
.device-unlock { min-height:40px; gap:8px; border-radius:var(--radius-control); }
.device-unlock .app-icon { width:16px; height:16px; flex:none; }
.lock-card [role=alert] { margin:12px 0 0; font-size:13px; overflow-wrap:anywhere; }
.lock-dialog { width:min(400px,calc(100vw - 32px)); }
.lock-dialog form,.lock-dialog label { display:grid; gap:10px; }
.lock-heading h2 { margin:0; color:var(--text); }
.unlock-alternative { justify-self:end; margin-top:8px; padding:4px 2px; color:var(--muted); background:none; border:0; font:inherit; font-size:13px; cursor:pointer; }
.unlock-alternative:hover,.unlock-alternative:focus-visible { color:var(--text); text-decoration:underline; }
.unlock-alternative:disabled { cursor:default; opacity:.6; }
.lock-dialog { padding:24px; border:1px solid var(--line); border-radius:16px; box-shadow:var(--shadow-menu); background:var(--surface); color:var(--text); }
.lock-dialog::backdrop { background:rgba(0,0,0,.3); }
.lock-heading { display:flex; justify-content:space-between; align-items:center; }
.lock-dialog p { color:var(--muted); line-height:1.5; }
.lock-dialog :is(input,select) { width:100%; padding:9px; background:var(--surface); color:var(--text); border:1px solid var(--control-border); border-radius:var(--radius-control); font:inherit; }
.lock-dialog label.device-choice { display:flex; align-items:center; }
.lock-dialog .device-choice input { width:auto; }
.lock-page [role=alert],.lock-dialog [role=alert] { color:var(--danger); }
.lock-indicator { position:absolute; right:18px; top:50%; transform:translateY(-50%); display:flex; color:var(--muted); }
.lock-indicator.windows { right:148px; }
.lock-indicator .app-icon { width:16px; height:16px; }
.lock-dialog form { gap:14px; }
.lock-dialog label { gap:6px; font-size:13px; }
.lock-heading h2 { font-size:19px; }
</style>
