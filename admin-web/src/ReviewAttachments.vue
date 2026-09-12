<template>
  <section v-if="references.length" class="review-files" aria-label="投稿附件">
    <h3>待公开附件 · {{ references.length }} 个</h3>
    <p class="muted">请逐一检查隐私与内容。文本自动审核不验证文件安全；通过后这些文件将可在广场下载。</p>
    <div v-for="file in references" :key="file.id" class="review-file"><div><strong>{{ file.name }}</strong><small>{{ file.mime }} · {{ Math.ceil(file.size / 1024) }} KB</small></div><button :disabled="Boolean(busy)" @click="load(file)">{{ busy === file.id ? '读取中…' : '查看文件' }}</button></div>
    <p v-if="error" role="alert">{{ error }}</p>
    <div v-if="selected" class="review-file-preview"><header><strong>{{ selected.name }}</strong><a :href="url" :download="selected.name">下载文件</a><button @click="clear">收起</button></header><img v-if="image" :src="url" :alt="selected.name"><pre v-else-if="selected.mime === 'text/plain'">{{ text }}</pre><p v-else class="muted">此格式不在网页内执行。请下载后使用可信工具检查，勿启用文档宏。</p></div>
  </section>
</template>
<script setup>
import { computed, onUnmounted, ref, watch } from 'vue';
import { fetchReviewAsset } from './reviewAssets.js';
const props = defineProps({ publicationId: { type: String, required: true }, references: { type: Array, default: () => [] } });
const selected = ref(null), url = ref(''), text = ref(''), busy = ref(''), error = ref('');
const image = computed(() => ['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(selected.value?.mime));
let version = 0;
function clear() { ++version; if (url.value) URL.revokeObjectURL(url.value); url.value = ''; text.value = ''; selected.value = null; busy.value = ''; error.value = ''; }
watch(() => [props.publicationId, props.references], clear);
onUnmounted(clear);
async function load(file) {
  if (busy.value) return;
  clear(); const current = version; busy.value = file.id;
  try {
    const bytes = await fetchReviewAsset(props.publicationId, file);
    if (current !== version) return;
    selected.value = file;
    const inline = ['image/png', 'image/jpeg', 'image/gif', 'image/webp', 'text/plain'].includes(file.mime);
    url.value = URL.createObjectURL(new Blob([bytes], { type: inline ? file.mime : 'application/octet-stream' }));
    if (file.mime === 'text/plain') text.value = new TextDecoder().decode(bytes.slice(0, 12000)) + (bytes.length > 12000 ? '\n…仅预览前 12 KB，请下载完整检查。' : '');
  } catch (caught) { if (current === version) error.value = caught.message; }
  finally { if (current === version) busy.value = ''; }
}
</script>
<style scoped>
.review-files { padding: 16px 0; border-top: 1px solid var(--line); margin-top: 16px; } .review-file, header { display: flex; gap: 16px; align-items: center; justify-content: space-between; } .review-file { padding: 12px 0; border-bottom: 1px solid var(--line); } small { display: block; margin-top: 4px; color: var(--muted); } .review-file-preview { padding: 16px; background: var(--bg); border-radius: 10px; margin-top: 16px; } header { margin-bottom: 16px; } header strong { margin-right: auto; overflow-wrap: anywhere; } img { display: block; max-width: 100%; max-height: 420px; margin: auto; } pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 360px; overflow: auto; }
</style>
