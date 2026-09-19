<template>
  <details data-testid="local-versions" @toggle="event => event.target.open && load()">
    <summary>查看同步前版本</summary>
    <p>保留本机最近 50 次被同步覆盖的标题、正文和库内附件，仅存于本机。恢复会新建独立副本，当前版本不被替换。</p>
    <p v-if="error" role="alert">{{ error }} <button class="button" :disabled="busy || disabled" @click="load">重新读取</button></p>
    <p v-if="note" role="status">{{ note }}</p>
    <p v-if="!busy && !error && !items.length">暂无被同步覆盖的本机版本</p>
    <ul><li v-for="item in items" :key="item.id"><button type="button" class="button" :disabled="busy || disabled" @click="preview(item)">{{ item.title }} · {{ new Date(Number(item.saved_at)).toLocaleString() }}</button></li></ul>
    <section v-if="selected && payload">
      <h4>{{ payload.title }}</h4><pre>{{ payload.content }}</pre><p>{{ payload.assets?.length || 0 }} 个库内附件</p>
      <button type="button" class="button" :disabled="busy || disabled || restored" @click="restore">{{ restored ? '已恢复为本地副本' : '恢复为本地副本' }}</button>
    </section>
  </details>
</template>
<script setup>
import { onUnmounted, ref } from 'vue';
import { getLocalPromptVersion, listLocalPromptVersions, restoreLocalPromptVersion } from '../platform/library.js';
const props = defineProps({ disabled: Boolean });
const emit = defineEmits(['restored', 'busy']);
const items = ref([]), selected = ref(null), payload = ref(null), busy = ref(false), error = ref(''), note = ref(''), restored = ref(false);
let disposed = false;
onUnmounted(() => { disposed = true; });
async function load() {
  if (busy.value || props.disabled) return;
  busy.value = true; error.value = ''; selected.value = null; payload.value = null;
  try { const rows = await listLocalPromptVersions(); if (!disposed) items.value = rows; }
  catch (e) { if (!disposed) error.value = `读取失败：${e.message || e}`; }
  finally { busy.value = false; }
}
async function preview(item) {
  if (busy.value || props.disabled) return;
  busy.value = true; error.value = ''; selected.value = null; payload.value = null; restored.value = false; note.value = '';
  try { const data = await getLocalPromptVersion(item.id); if (!disposed) { payload.value = data; selected.value = item; } }
  catch (e) { if (!disposed) error.value = `读取失败：${e.message || e}`; }
  finally { busy.value = false; }
}
async function restore() {
  if (busy.value || props.disabled || restored.value || !selected.value) return;
  busy.value = true; emit('busy', true); error.value = '';
  try {
    await restoreLocalPromptVersion(selected.value.id);
    restored.value = true; note.value = '已恢复为本地独立提示词，当前同步版本保持不变。'; emit('restored');
  } catch (e) { error.value = `恢复失败：${e.message || e}`; }
  finally { busy.value = false; emit('busy', false); }
}
</script>
<style scoped>
details { margin: 20px 0; } summary { cursor: pointer; font-weight: 600; }
ul { list-style: none; padding: 0; max-height: 240px; overflow: auto; } li { margin: 6px 0; }
pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 320px; overflow: auto; font: inherit; }
</style>
