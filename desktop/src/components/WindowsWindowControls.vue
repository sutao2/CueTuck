<template>
  <div class="window-controls" aria-label="窗口控制" @mousedown.stop @dblclick.stop>
    <button type="button" aria-label="最小化" title="最小化" @click="run('minimize')"><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M1 6.5h10" /></svg></button>
    <button type="button" :aria-label="maximized ? '还原窗口' : '最大化'" :title="maximized ? '还原窗口' : '最大化'" @click="run('toggleMaximize')"><svg viewBox="0 0 12 12" aria-hidden="true"><path v-if="maximized" d="M3.5 3.5v-2h7v7h-2M1.5 3.5h7v7h-7z" /><path v-else d="M1.5 1.5h9v9h-9z" /></svg></button>
    <button type="button" class="window-close" aria-label="关闭窗口" title="关闭窗口" @click="run('close')"><svg viewBox="0 0 12 12" aria-hidden="true"><path d="m1.5 1.5 9 9m0-9-9 9" /></svg></button>
    <p v-if="error" class="window-control-error" role="alert">窗口操作未完成，请重试。</p>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
const maximized = ref(false), error = ref(false);
let appWindow, unlisten, disposed = false, revision = 0;
async function refresh() {
  const current = ++revision;
  const value = await appWindow.isMaximized();
  if (!disposed && current === revision) maximized.value = value;
}
async function run(action) {
  if (!appWindow) return;
  error.value = false;
  try {
    await appWindow[action]();
    if (action === 'toggleMaximize') await refresh();
  } catch { if (!disposed) error.value = true; }
}
onMounted(async () => {
  if (!window.__TAURI_INTERNALS__) return;
  appWindow = getCurrentWindow();
  try {
    const stop = await appWindow.onResized(() => { refresh().catch(() => { if (!disposed) error.value = true; }); });
    if (disposed) { stop(); return; }
    unlisten = stop;
    await refresh();
  } catch { if (!disposed) error.value = true; }
});
onUnmounted(() => { disposed = true; unlisten?.(); });
</script>

<style scoped>
.window-controls { position: fixed; z-index: 3000; top: 0; right: 0; display: flex; height: var(--titlebar-height); background: var(--sidebar); color: var(--text); user-select: none; }
.window-controls button { display: grid; place-items: center; width: 46px; height: 100%; padding: 0; border: 0; border-radius: 0; background: transparent; color: inherit; }
.window-controls button:hover { background: var(--hover); }
.window-controls button:focus-visible { outline-offset: -3px; }
.window-controls .window-close:hover { background: #c42b1c; color: #fff; }
.window-controls svg { width: 12px; height: 12px; fill: none; stroke: currentColor; stroke-width: 1; }
.window-control-error { position: absolute; top: 100%; right: 8px; width: max-content; margin: 4px 0; padding: 8px 12px; border: 1px solid var(--line); border-radius: 6px; background: var(--surface); color: var(--danger); font-size: 12px; }
</style>
