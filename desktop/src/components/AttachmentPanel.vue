<template>
  <section class="attachments" data-testid="attachments" :aria-busy="working" @dragover="dragover" @drop="drop" @paste="paste">
    <header class="attachments-heading">
      <div><h3>图片与附件 <span>{{ modelValue.length }}</span></h3><p>仅保存在本机，随本地备份保留；不会自动同步或发布。</p></div>
      <button v-if="!readonly" class="button" type="button" :disabled="disabled || working" @click="picker.click()">＋ 添加文件</button>
      <input v-if="!readonly" ref="picker" hidden type="file" multiple :accept="ASSET_ACCEPT" data-testid="asset-picker" @change="selectFiles">
    </header>
    <p v-if="error" role="alert">{{ error }}</p><p v-if="note" role="status" class="field-help">{{ note }}</p>
    <button v-if="!modelValue.length && !readonly" type="button" class="attachment-drop" :disabled="disabled || working" @click="picker.click()">
      <AppIcon name="image" /><strong>{{ working ? '正在读取文件…' : '选择文件，或拖到这里' }}</strong><span>也可在编辑页粘贴图片 · 单文件 5 MiB · 共 20 MiB / 12 个</span>
      <small>图片、PDF、文本、Markdown、CSV、JSON、DOCX、XLSX、PPTX</small>
    </button>
    <div v-if="modelValue.length" class="attachment-grid">
      <article v-for="asset in modelValue" :key="asset.id" class="attachment-item" :class="{ selected: selectedId === asset.id }">
        <button type="button" class="attachment-open" :aria-label="`查看 ${asset.name}`" @click="selectedId = selectedId === asset.id ? '' : asset.id">
          <img v-if="asset.mime.startsWith('image/')" :src="assetUrl(asset)" alt="" loading="lazy">
          <span v-else class="attachment-file-icon"><AppIcon name="file" /></span>
          <span class="attachment-label"><strong :title="asset.name">{{ asset.name }}</strong><small>{{ formatBytes(assetSize(asset)) }}</small></span>
        </button>
        <button v-if="!readonly" type="button" class="attachment-remove" :aria-label="`移除 ${asset.name}`" :disabled="disabled || working" @click="remove(asset.id)">×</button>
      </article>
    </div>
    <section v-if="selected" class="attachment-preview" :aria-label="selected.name">
      <header><strong>{{ selected.name }}</strong><div><button type="button" class="button" :disabled="working || !savedIds.includes(selected.id)" @click="exportFile">{{ savedIds.includes(selected.id) ? '导出副本' : '保存后可导出' }}</button><button type="button" class="button ghost-button" aria-label="收起附件预览" @click="selectedId = ''">收起</button></div></header>
      <img v-if="selected.mime.startsWith('image/')" :src="assetUrl(selected)" :alt="selected.name">
      <pre v-else-if="selected.mime === 'text/plain'">{{ textPreview(selected) }}</pre>
      <p v-else>此文档不在应用内执行或解析。导出副本后，可使用系统应用打开。</p>
    </section>
  </section>
</template>

<script setup>
import { computed, ref } from 'vue';
import AppIcon from './AppIcon.vue';
import { ASSET_ACCEPT, readAssetFiles, assetUrl, assetSize, formatBytes, exportPromptAsset, textPreview } from '../platform/assets.js';
const props = defineProps({ modelValue: { type: Array, default: () => [] }, promptId: String, savedIds: { type: Array, default: () => [] }, readonly: Boolean, disabled: Boolean });
const emit = defineEmits(['update:modelValue', 'busy']);
const picker = ref(null), working = ref(false), error = ref(''), note = ref(''), selectedId = ref('');
const selected = computed(() => props.modelValue.find(a => a.id === selectedId.value));
async function add(files) {
  if (props.readonly || props.disabled || working.value || !files.length) return;
  working.value = true; emit('busy', true); error.value = ''; note.value = '';
  try { emit('update:modelValue', await readAssetFiles(files, props.modelValue)); }
  catch (err) { error.value = err.message || '文件读取失败，请重试'; }
  finally { working.value = false; emit('busy', false); }
}
function selectFiles(event) { add([...event.target.files]); event.target.value = ''; }
function dragover(event) { if (event.dataTransfer?.types.includes('Files') && !props.readonly && !props.disabled) event.preventDefault(); }
function drop(event) {
  const files = [...(event.dataTransfer?.files ?? [])];
  if (files.length && !props.readonly && !props.disabled) { event.preventDefault(); event.stopPropagation(); add(files); }
}
function paste(event) {
  const files = [...(event.clipboardData?.files ?? [])].filter(file => file.type.startsWith('image/'));
  if (files.length && !props.readonly && !props.disabled) { event.preventDefault(); event.stopPropagation(); add(files); }
}
function remove(id) { emit('update:modelValue', props.modelValue.filter(a => a.id !== id)); if (selectedId.value === id) selectedId.value = ''; }
async function exportFile() {
  if (!selected.value || working.value) return;
  working.value = true;
  try { note.value = `已导出：${await exportPromptAsset(props.promptId, selected.value)}`; }
  catch (err) { error.value = err.message || String(err); }
  finally { working.value = false; }
}
defineExpose({ paste, drop, dragover });
</script>
