<template>
  <section v-page-focus="() => $emit('cancel')" class="workspace-page" data-testid="local-detail" aria-labelledby="local-detail-title">
    <header class="modal-header">
      <div><p class="modal-kicker">{{ prompt.source === 'downloaded' ? '本地副本' : '本地提示词' }}</p><h2 id="local-detail-title">{{ prompt.title }}</h2></div>
      <button class="page-back" type="button" @click="$emit('cancel')">← 返回</button>
    </header>
    <div class="create-body">
      <p v-if="prompt.model || prompt.author" class="use-hint">{{ [prompt.model, prompt.author].filter(Boolean).join(' · ') }}</p>
      <pre class="reading-content">{{ prompt.content || '还没有正文' }}</pre>
      <template v-if="prompt.asset_count">
        <h3>参考资料</h3>
        <p v-if="loading" role="status">正在读取附件…</p>
        <p v-else-if="error" role="alert">{{ error }} <button type="button" class="button" @click="load">重试</button></p>
        <AttachmentPanel v-else :model-value="assets" :prompt-id="prompt.id" :saved-ids="assets.map(a => a.id)" readonly />
      </template>
    </div>
    <footer class="modal-footer">
      <button type="button" class="button ghost-button" data-testid="detail-edit" @click="$emit('edit')">编辑</button>
      <button type="button" class="button primary-button" @click="$emit('use')">使用提示词</button>
    </footer>
  </section>
</template>
<script setup>
import { onMounted, ref } from 'vue';
import { vPageFocus } from '../lib/pageFocus.js';
import { listPromptAssets } from '../platform/assets.js';
import AttachmentPanel from './AttachmentPanel.vue';
const props = defineProps({ prompt: { type: Object, required: true } });
defineEmits(['cancel', 'edit', 'use']);
const assets = ref([]), loading = ref(false), error = ref('');
async function load() {
  loading.value = true; error.value = '';
  try { assets.value = await listPromptAssets(props.prompt.id); }
  catch (err) { error.value = `读取附件失败：${err.message || err}`; }
  finally { loading.value = false; }
}
onMounted(() => { if (props.prompt.asset_count) load(); });
</script>
<style scoped>
.reading-content { white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; line-height: 1.9; margin: 0; }
</style>
