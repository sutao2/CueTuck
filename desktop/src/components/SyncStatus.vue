<template>
  <component :is="expanded ? 'p' : 'button'" v-if="session.loggedIn" :type="expanded ? undefined : 'button'" class="sync-status" data-testid="sync-status" :title="details" @click="!expanded && $emit('open')">
    {{ error ? '同步状态读取失败' : pending ? `${pending} 项待发送` : result ? '同步记录' : '尚未手动同步' }}<span v-if="expanded"> · {{ details }}</span>
  </component>
</template>
<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { listSyncQueue } from '../platform/syncQueue.js';
import { readSyncResult } from '../platform/syncStatus.js';
const props = defineProps({ session: { type: Object, required: true }, expanded: Boolean });
defineEmits(['open']);
const pending = ref(0), result = ref(null), error = ref('');
let request = 0;
const details = computed(() => error.value || (result.value ? `${new Date(result.value.at).toLocaleString()}：${result.value.message}` : '个人库需在设置中手动同步；仅在本机保存不代表已上传。'));
async function refresh() {
  const token = ++request, email = props.session.email;
  pending.value = 0; result.value = null; error.value = '';
  if (!props.session.loggedIn || !email) return;
  try {
    const [queue, saved] = await Promise.all([listSyncQueue(), readSyncResult(email)]);
    if (token !== request) return;
    pending.value = queue.filter(job => job.email === email).length;
    result.value = saved;
  } catch { if (token === request) error.value = '同步状态读取失败，点击查看同步设置'; }
}
watch(() => [props.session.email, props.session.loggedIn], refresh, { immediate: true });
onMounted(() => window.addEventListener('library-sync-status', refresh));
onUnmounted(() => { ++request; window.removeEventListener('library-sync-status', refresh); });
</script>
<style scoped>
.sync-status { border: 0; background: transparent; color: var(--muted); font: inherit; text-align: left; cursor: pointer; padding: 4px; }
.sync-status:hover { color: var(--text); }
</style>
