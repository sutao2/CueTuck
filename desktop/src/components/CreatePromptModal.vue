<template>
    <section v-page-focus="requestClose" class="workspace-page editor-page" data-testid="prompt-editor" role="region" aria-labelledby="editor-title" :aria-busy="busy || assetBusy" @keydown="onSaveKeydown" @paste="assetPanel?.paste($event)" @dragover="assetPanel?.dragover($event)" @drop="assetPanel?.drop($event)">
      <header class="modal-header">
        <div>
          <h2 id="editor-title">{{ heading }}</h2>
        </div>
        <button type="button" class="page-back" aria-label="返回" :disabled="busy || assetBusy" @click="requestClose">← 返回</button>
      </header>
      <div v-if="confirmDiscard" class="create-body" data-testid="discard-editor">
        <h3>放弃未保存的修改？</h3>
        <p>关闭后本次修改不会保存。</p>
        <div class="modal-actions">
          <button ref="keepEditingButton" type="button" class="button primary-button" @click="keepEditing">继续编辑</button>
          <button type="button" class="button danger-button" @click="$emit('cancel')">放弃修改</button>
        </div>
      </div>
      <div v-else class="create-body" :inert="busy ? '' : undefined">
        <p v-if="example" class="use-hint">这是可编辑的示例，尚未保存。点击「试填预览」体验变量，保存后可用底栏启动器再次找到它。</p>
        <p v-if="collectionTitle" class="use-hint" data-testid="collection-only-note">保存到「{{ collectionTitle }}」，仅在此合集中显示。移出合集后会保留为独立提示词。</p>
        <p v-if="error" role="alert" class="use-hint">{{ error }}</p>
        <div v-if="!prompt && !collectionTitle" class="create-type-grid">
          <button type="button" class="create-type" :disabled="assetBusy || assetLoading" :class="{ active: kind === 'prompt' }" :aria-pressed="kind === 'prompt'" @click="kind = 'prompt'">
            <strong>单个提示词</strong>
            <small>一条可直接使用的提示词。</small>
          </button>
          <button type="button" class="create-type" :disabled="assetBusy || assetLoading" :class="{ active: kind === 'collection' }" :aria-pressed="kind === 'collection'" @click="kind = 'collection'">
            <strong>提示词合集</strong>
            <small>同一主题下的一组提示词。</small>
          </button>
        </div>
        <label class="field">
          <span>{{ kind === "collection" ? "合集名称" : "标题" }}</span>
          <input v-model="title" data-dialog-autofocus placeholder="例如：SaaS 官网生成器">
        </label>
        <div class="editor-metadata" :class="{ 'single-field': kind === 'collection' }">
        <label v-if="kind === 'prompt'" class="field">
          <span>模型</span>
          <SearchableSelect data-testid="prompt-model" v-model="model" :options="[{value:'',label:'未指定'}, ...modelOptions.map(name => ({value:name,label:name}))]" />
        </label>
        <label class="field">
          <span>分类</span>
          <SearchableSelect data-testid="prompt-category" v-model="categoryId" :options="[{value:'',label:'未分类'}, ...groups.flatMap(group => [{value:group.id,label:group.name}, ...group.children.map(child => ({value:child.id,label:group.name+' / '+child.name}))])]" />
        </label>
        </div>
        <div v-if="kind === 'prompt'" class="editor-writing">
          <div class="editor-tools">
            <button type="button" class="button ghost-button" data-testid="insert-variable" @click="insertVariable">＋ 插入变量</button>
            <button type="button" class="button ghost-button" :aria-expanded="trialOpen" data-testid="toggle-trial" @click="trialOpen = !trialOpen">{{ trialOpen ? '收起试填' : '试填预览' }}</button>
          </div>
          <div class="editor-writing-grid" :class="{ 'with-trial': trialOpen }">
            <label class="field"><span>提示词内容</span>
              <textarea ref="contentInput" v-model="content" rows="12" placeholder="输入 {} 创建独立占位符，或 {{变量名}} 创建同名共用的变量"></textarea>
            </label>
            <PromptTrial v-if="trialOpen" :content="content" />
          </div>
        </div>
        <label v-else class="field">
          <span>封面</span>
          <select v-model="coverType" :disabled="assetBusy">
            <option value="none">无封面</option>
            <option value="single">单图</option>
            <option value="grid">九宫格</option>
          </select>
        </label>
        <label v-if="kind === 'collection' && coverType !== 'none'" class="field">
          <span>{{ coverType === "single" ? "封面图" : "封面图（最多 9 张，缺图用占位）" }}</span>
          <input
            type="file"
            accept="image/png,image/jpeg,image/gif,image/webp"
            data-testid="cover-files"
            :disabled="busy || assetBusy"
            :multiple="coverType === 'grid'"
            @change="onCoverFiles"
          >
        </label>
        <p v-if="coverError" role="alert" class="use-hint">{{ coverError }}</p>
        <template v-if="kind === 'prompt'">
          <p v-if="assetLoading" role="status">正在读取附件…</p>
          <p v-else-if="assetError" role="alert">{{ assetError }} <button type="button" class="button" @click="loadAssets">重试</button></p>
          <AttachmentPanel v-else ref="assetPanel" v-model="assets" :prompt-id="prompt?.id" :saved-ids="savedAssetIds" :disabled="busy" @busy="assetBusy = $event" />
        </template>
      </div>
      <footer v-if="!confirmDiscard" class="modal-footer">
        <button v-if="prompt" type="button" class="button danger-button" :disabled="busy || assetBusy" @click="$emit('remove', prompt.id)">
          删除
        </button>
        <span v-else class="create-location">将创建在本地库</span>
        <div class="modal-actions">
          <button type="button" class="button ghost-button" :disabled="busy || assetBusy" @click="requestClose">取消</button>
          <button type="button" class="button primary-button" :disabled="busy || assetBusy || assetLoading || Boolean(assetError) || !title.trim()" @click="submit">
            {{ busy ? '正在保存…' : kind === "collection" && !prompt ? "创建合集" : "保存" }}
          </button>
        </div>
      </footer>
    </section>
</template>

<script setup>
import SearchableSelect from "./SearchableSelect.vue";
import { computed, nextTick, onMounted, ref } from "vue";
import PromptTrial from './PromptTrial.vue';
import AttachmentPanel from './AttachmentPanel.vue';
import { listPromptAssets } from '../platform/assets.js';
import { vPageFocus } from "../lib/pageFocus.js";
import { parseCoverUrls, normalizeCoverUrls } from "../lib/cover.js";

const props = defineProps({
  prompt: { type: Object, default: null },
  example: Boolean,
  collectionTitle: { type: String, default: "" },
  groups: { type: Array, default: () => [] },
  modelOptions: { type: Array, default: () => [] },
  defaultModel: { type: String, default: "" },
  defaultCategoryId: { type: String, default: "" },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
});

const emit = defineEmits(["cancel", "save", "remove", "stay"]);
const kind = ref(props.prompt?.kind ?? "prompt");
const title = ref(props.prompt?.title ?? (props.example ? "示例：写一封简洁的邮件" : ""));
const content = ref(props.prompt?.content ?? (props.example ? "请帮我写一封邮件，收件人是{{收件人}}，主题是{{主题}}。语气友好，表达简洁，结尾明确下一步。" : ""));
const contentInput = ref(null), trialOpen = ref(false);
async function insertVariable() {
  if (props.busy || assetBusy.value) return;
  const input = contentInput.value;
  const start = input.selectionStart, end = input.selectionEnd;
  const selected = content.value.slice(start, end).trim();
  const name = selected && !/[{}\n]/.test(selected) ? selected : '变量名';
  content.value = content.value.slice(0, start) + '{{' + name + '}}' + content.value.slice(end);
  await nextTick(); input.focus(); input.setSelectionRange(start + 2, start + 2 + name.length);
}
const categoryId = ref(props.prompt ? (props.prompt.category_id ?? "") : props.defaultCategoryId);
const model = ref(props.prompt ? (props.prompt.model ?? "") : (props.defaultModel ?? ""));
const coverType = ref(props.prompt?.cover_type ?? "none");
const coverUrls = ref(parseCoverUrls(props.prompt?.cover_json));
const coverError = ref('');
const assets = ref([]), savedAssetIds = ref([]), assetPanel = ref(null), assetBusy = ref(false), assetError = ref('');
const assetLoading = ref(Boolean(props.prompt?.asset_count));
let initialAssets = '[]';
async function loadAssets() {
  assetLoading.value = true; assetError.value = '';
  try {
    assets.value = await listPromptAssets(props.prompt.id);
    savedAssetIds.value = assets.value.map(a => a.id);
    initialAssets = JSON.stringify(assets.value);
  } catch (err) { assetError.value = `读取附件失败，暂不能保存：${err.message || err}`; }
  finally { assetLoading.value = false; }
}
onMounted(() => { if (props.prompt?.asset_count) loadAssets(); });
const confirmDiscard = ref(false), keepEditingButton = ref(null);
const snapshot = () => JSON.stringify([kind.value, title.value, content.value, categoryId.value, model.value, coverType.value, coverUrls.value]);
const initialSnapshot = snapshot();
async function requestClose() {
  if (props.busy || assetBusy.value) return;
  if (confirmDiscard.value) { keepEditing(); return; }
  if (snapshot() === initialSnapshot && JSON.stringify(assets.value) === initialAssets) { emit('cancel'); return; }
  confirmDiscard.value = true;
  await nextTick();
  keepEditingButton.value?.focus();
}
async function keepEditing() {
  confirmDiscard.value = false;
  emit('stay');
  await nextTick();
  document.querySelector('[data-testid="prompt-editor"] [data-dialog-autofocus]')?.focus();
}
defineExpose({ requestClose });
const heading = computed(() => {
  if (props.prompt) return kind.value === "collection" ? "编辑合集" : "编辑提示词";
  return kind.value === "collection" ? "新建合集" : "新建提示词";
});

function readAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

async function onCoverFiles(event) {
  if (props.busy || assetBusy.value) return;
  const limit = coverType.value === "single" ? 1 : 9;
  const files = [...(event.target.files || [])].slice(0, limit);
  if (!files.length) return;
  assetBusy.value = true; coverError.value = '';
  try {
    if (files.some(file => file.size > 5 * 1024 * 1024) || files.reduce((size, file) => size + file.size, 0) > 20 * 1024 * 1024) throw new Error('单文件上限 5 MiB，封面总量上限 20 MiB');
    coverUrls.value = normalizeCoverUrls(await Promise.all(files.map(readAsDataUrl)));
  } catch (err) { coverError.value = `封面未更改：${err?.message || '图片读取失败，请重新选择'}`; }
  finally { assetBusy.value = false; event.target.value = ''; }
}

function submit() {
  if (confirmDiscard.value || props.busy || assetBusy.value || assetLoading.value || assetError.value || !title.value.trim()) return;
  emit("save", {
    id: props.prompt?.id,
    kind: kind.value,
    title: title.value.trim(),
    content: content.value,
    assets: assets.value,
    categoryId: categoryId.value || null,
    model: kind.value === "prompt" ? model.value || null : null,
    coverType: coverType.value,
    coverUrls: coverType.value === "none" ? [] : coverUrls.value,
  });
}

function onSaveKeydown(event) {
  if (event.isComposing || event.key.toLowerCase() !== 's' || !(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) return;
  event.preventDefault();
  event.stopPropagation();
  if (!event.repeat) submit();
}
</script>

<style scoped>
.editor-page > .modal-header { padding-top: 16px; gap: 10px; }
.editor-page > .create-body { padding-top: 20px; gap: 18px; }
.editor-page .editor-metadata { display: flex; flex-wrap: wrap; gap: 10px 16px; padding: 12px 14px; border: 1px solid var(--line); border-radius: 8px; background: var(--sidebar); }
.editor-metadata .field { flex: 1 1 180px; display: flex; align-items: center; gap: 8px; min-width: 0; }
.editor-metadata .field > span { flex-shrink: 0; color: var(--muted); }
.editor-metadata select, .editor-metadata :deep(.searchable-select) { flex: 1; width: 0; min-width: 0; padding: 6px 8px; font-size: 12px; }
.editor-tools { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 10px; }
.editor-tools .button { border-color: transparent; background: transparent; color: var(--muted); }
.editor-tools .button:hover, .editor-tools .button[aria-expanded="true"] { background: var(--sidebar); color: var(--text); }
.editor-page .create-type { padding: 10px 12px; }
.editor-writing-grid { display: grid; gap: 20px; }
.editor-writing-grid > * { min-width: 0; }
.editor-page .editor-writing-grid textarea { min-height: min(420px, 42vh); }
@media (min-width: 1100px) { .editor-writing-grid.with-trial { grid-template-columns: 1fr 1fr; align-items: start; } }
</style>
