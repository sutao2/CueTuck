<template>
  <section v-if="references.length" class="published-files" aria-label="公开附件">
    <h3>附件 <span>{{ references.length }}</span></h3>
    <p>查看时联网加载；下载到本地会一并保存全部附件。</p>
    <div v-for="file in references" :key="file.id" class="file-row"><div><strong>{{ file.name }}</strong><small>{{ formatBytes(file.size) }} · {{ file.mime }}</small></div><button type="button" class="button ghost-button" :disabled="Boolean(busy)" @click="preview(file)">{{ busy === file.id ? '正在加载…' : '查看' }}</button></div>
    <p v-if="error" role="alert">{{ error }}</p>
    <div v-if="asset" class="file-preview"><header><strong>{{ asset.name }}</strong><button class="button ghost-button" @click="asset = null">收起</button></header><img v-if="asset.mime.startsWith('image/')" :src="assetUrl(asset)" :alt="asset.name"><pre v-else-if="asset.mime === 'text/plain'">{{ textPreview(asset) }}</pre><p v-else>此文档不在应用内执行。请下载提示词到本地后，在附件区导出查看。</p></div>
  </section>
</template>
<script setup>
import { ref, watch, onUnmounted } from 'vue';
import { formatBytes, assetUrl, textPreview } from '../platform/assets.js';
import { downloadPublishedAsset } from '../platform/privateMedia.js';
import { getSession } from '../platform/session.js';
const props = defineProps({ itemId: { type: String, required: true }, references: { type: Array, default: () => [] } });
const asset = ref(null), busy = ref(''), error = ref('');
let version = 0;
function reset() { ++version; asset.value = null; busy.value = ''; error.value = ''; }
watch(() => [props.itemId, props.references], reset);
onUnmounted(reset);
async function preview(file) {
  if (busy.value) return;
  const current = ++version; busy.value = file.id; error.value = ''; asset.value = null;
  try { const value = await downloadPublishedAsset(props.itemId, file, getSession().accessToken); if (current === version) asset.value = value; }
  catch (caught) { if (current === version) error.value = caught.message; }
  finally { if (current === version) busy.value = ''; }
}
</script>
<style scoped>
.published-files { margin: 28px 0; } h3 { font-size: 14px; font-weight: 600; } h3 span, p, small { color: var(--muted); font-size: 12px; } .file-row { display: flex; justify-content: space-between; align-items: center; gap: 16px; padding: 12px 0; border-bottom: 1px solid var(--line); } .file-row > div { min-width: 0; overflow-wrap: anywhere; } strong { font-size: 13px; font-weight: 500; } small { display: block; margin-top: 5px; } .file-preview { margin-top: 16px; padding: 16px; background: var(--sidebar); border-radius: 12px; } header { display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-bottom: 12px; } img { display: block; max-width: 100%; max-height: 420px; margin: auto; border-radius: 6px; } pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 360px; overflow: auto; }
</style>
