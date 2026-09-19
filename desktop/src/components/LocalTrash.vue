<template>
  <details data-testid="local-trash" @toggle="opened">
    <summary>本地回收站</summary>
    <p>恢复删除的提示词及库内附件。合集恢复为空合集；原成员保留在当前所在位置。最多显示最近 200 个匹配项。</p>
    <form @submit.prevent="load"><input v-model="query" placeholder="搜索已删除的标题" aria-label="搜索回收站" :disabled="busy"><button type="submit" class="button" :disabled="busy">查找</button></form>
    <p v-if="note" role="status">{{ note }}</p><p v-if="error" role="alert">{{ error }} <button class="button" :disabled="busy" @click="load">重新读取</button></p>
    <p v-if="!busy && !error && !items.length">没有已删除的匹配内容</p>
    <ul><li v-for="item in items" :key="item.kind + item.id"><span>{{ item.title }} · {{ item.kind === 'collection' ? '合集' : '提示词' }}</span><button class="button" :disabled="busy" @click="restore(item)">恢复</button></li></ul>
  </details>
</template>
<script setup>
import { onUnmounted, ref } from 'vue';
import { listDeletedLocalItems, restoreDeletedLocalItem } from '../platform/library.js';
const emit = defineEmits(['restored', 'busy']);
const props = defineProps({ disabled: Boolean });
const items = ref([]), query = ref(''), busy = ref(false), error = ref(''), note = ref('');
let disposed = false;
onUnmounted(() => { disposed = true; });
function opened(event) { if (event.target.open) load(); }
async function load() {
  if (busy.value || props.disabled) return;
  busy.value = true; error.value = '';
  try { const rows = await listDeletedLocalItems(query.value); if (!disposed) items.value = rows; }
  catch (e) { if (!disposed) error.value = `读取失败：${e.message || e}`; }
  finally { busy.value = false; }
}
async function restore(item) {
  if (busy.value || props.disabled) return;
  busy.value = true; emit('busy', true); error.value = ''; note.value = '';
  try {
    await restoreDeletedLocalItem(item.id, item.kind);
    items.value = items.value.filter(row => row !== item);
    note.value = `已恢复「${item.title}」`; emit('restored');
  } catch (e) { error.value = `恢复失败：${e.message || e}`; }
  finally { busy.value = false; emit('busy', false); }
}
</script>
<style scoped>
details { margin: 20px 0; } summary { cursor: pointer; font-weight: 600; }
form, li { display: flex; gap: 12px; align-items: center; } input, li span { min-width: 0; flex: 1; overflow-wrap: anywhere; }
ul { padding: 0; max-height: 320px; overflow: auto; } li { padding: 8px 0; border-bottom: 1px solid var(--line); }
</style>
