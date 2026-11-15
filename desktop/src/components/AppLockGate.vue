<template>
  <slot v-if="!native || status && !status.locked" />
  <main v-else-if="!launcher" class="lock-page">
    <header data-tauri-drag-region><WindowsWindowControls v-if="windows" /></header>
    <section class="lock-card">
      <img src="../assets/app-icon.png" alt="CueTuck" width="44" height="44">
      <template v-if="status?.locked">
        <h1>{{ tr('解锁工作台') }}</h1>
        <form @submit.prevent="perform({action:'unlock', password:current})">
          <label>{{ tr('解锁口令') }}<input ref="unlockInput" v-model="current" type="password" required autocomplete="current-password" :disabled="busy"></label>
          <button class="button primary-button" :disabled="busy">{{ tr('解锁') }}</button>
        </form>
        <button v-if="status.device && available" class="button" :disabled="busy" @click="perform({action:'unlock_device'})">{{ tr('使用 {0} 解锁', [deviceName]) }}</button>
      </template>
      <template v-else><h1>{{ tr('正在检查应用锁') }}</h1><button v-if="error" class="button" @click="initialize">{{ tr('重试') }}</button></template>
      <p v-if="error" role="alert">{{ tr(error) }}</p>
    </section>
  </main>

  <dialog v-if="manage && status && !status.locked" ref="dialog" class="lock-dialog" aria-labelledby="lock-title" @keydown.stop @cancel="cancelSettings">
    <div class="lock-heading"><h2 id="lock-title">{{ tr('应用锁') }}</h2><button class="button" :disabled="busy" @click="closeSettings">{{ tr('关闭') }}</button></div>
    <template v-if="manage==='discard'">
      <p>{{ tr('片段尚未保存。锁定将丢弃本次编辑。') }}</p>
      <button class="button" @click="closeSettings">{{ tr('继续编辑') }}</button>
      <button class="button primary-button" :disabled="busy" @click="perform({action:'lock'})">{{ tr('丢弃并锁定') }}</button>
      <p v-if="error" role="alert">{{ tr(error) }}</p>
    </template>
    <template v-else>
    <p>{{ tr('启用后，每次启动都需解锁。你也可以随时在侧栏锁定工作台。') }}</p>
    <p v-if="!status.enabled">{{ tr('请记住解锁口令。启用期间，本机 MCP 工具将暂停读取。') }}</p>
    <form @submit.prevent="saveSettings">
      <label v-if="status.enabled">{{ tr('操作') }}
        <select v-model="mode" :disabled="busy" @change="clearForm">
          <option value="change">{{ tr('更改解锁口令') }}</option>
          <option v-if="available || status.device" value="device">{{ tr(status.device ? '关闭 {0} 解锁' : '启用 {0} 解锁', [deviceName]) }}</option>
          <option value="disable">{{ tr('关闭应用锁') }}</option>
        </select>
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
import { computed, nextTick, onMounted, onUnmounted, provide, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { tr } from '../platform/interfaceLanguage.js';
import WindowsWindowControls from './WindowsWindowControls.vue';

const props = defineProps({ launcher: Boolean });
const native = Boolean(window.__TAURI_INTERNALS__);
const windows = /Windows/i.test(navigator.userAgent);
const deviceName = windows ? 'Windows Hello' : 'Touch ID';
const status = ref(null), available = ref(false), manage = ref(false), busy = ref(false), error = ref('');
const current = ref(''), password = ref(''), confirmation = ref(''), mode = ref('change'), enableDevice = ref(false);
const unlockInput = ref(null), dialog = ref(null);
let stop, revision = 0, disposed = false;
function clearForm() { current.value=''; password.value=''; confirmation.value=''; error.value=''; }
function closeSettings() { dialog.value?.close(); manage.value=false; clearForm(); }
function cancelSettings(event) { if (busy.value) event.preventDefault(); else closeSettings(); }
async function probeDevice() {
  try { available.value = await invoke('app_lock_device_available'); } catch { available.value=false; }
}
async function refresh() {
  const token = ++revision;
  try {
    const next = await invoke('app_lock_status');
    if (typeof next?.locked !== 'boolean') throw new Error();
    if (token !== revision || disposed) return;
    const entering = next.locked && !status.value?.locked;
    status.value = next;
    if (entering) {
      closeSettings();
      if (!props.launcher) {
        await nextTick();
        unlockInput.value?.focus();
        if (next.device) void probeDevice();
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
    try {
      const off = await listen('app-lock-changed', refresh);
      if (disposed) { off(); return; }
      stop = off;
    } catch { error.value='无法读取应用锁状态'; return; }
  }
  await refresh();
}
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
onMounted(()=>{ if (native) { void initialize(); window.addEventListener('focus', onFocus); } });
function onFocus() { if (stop) void refresh(); }
onUnmounted(()=>{ disposed=true; ++revision; stop?.(); window.removeEventListener('focus',onFocus); });
</script>

<style scoped>
.lock-page { min-height:100vh; display:grid; place-items:center; background:var(--surface); color:var(--text); }
.lock-page header { position:fixed; inset:0 0 auto; height:var(--titlebar-height); }
.lock-card,.lock-dialog { width:min(360px,calc(100vw - 32px)); }
.lock-card,.lock-card form,.lock-dialog form,.lock-dialog label,.lock-card label { display:grid; gap:10px; }
.lock-card h1,.lock-heading h2 { margin:0; color:var(--text); }
.lock-card h1 { margin:8px 0; }
.lock-dialog { padding:22px; border:1px solid var(--line); border-radius:16px; box-shadow:var(--shadow-menu); background:var(--surface); color:var(--text); }
.lock-dialog::backdrop { background:rgba(0,0,0,.3); }
.lock-heading { display:flex; justify-content:space-between; align-items:center; }
.lock-dialog p { color:var(--muted); line-height:1.5; }
.lock-card input,.lock-dialog :is(input,select) { width:100%; padding:9px; background:var(--surface); color:var(--text); border:1px solid var(--control-border); border-radius:var(--radius-control); font:inherit; }
.lock-dialog label.device-choice { display:flex; align-items:center; }
.lock-dialog .device-choice input { width:auto; }
.lock-page [role=alert],.lock-dialog [role=alert] { color:var(--danger); }
</style>
