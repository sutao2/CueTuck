<template>
  <div class="modal-layer" data-testid="prompt-editor">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section v-dialog-focus="() => $emit('cancel')" class="modal create-modal" role="dialog" aria-modal="true" aria-labelledby="editor-title">
      <header class="modal-header">
        <div>
          <h2 id="editor-title">{{ heading }}</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="create-body">
        <p v-if="error" role="alert" class="use-hint">{{ error }}</p>
        <div v-if="!prompt" class="create-type-grid">
          <button type="button" class="create-type" :class="{ active: kind === 'prompt' }" :aria-pressed="kind === 'prompt'" @click="kind = 'prompt'">
            <strong>单个提示词</strong>
            <small>一条可直接使用的提示词。</small>
          </button>
          <button type="button" class="create-type" :class="{ active: kind === 'collection' }" :aria-pressed="kind === 'collection'" @click="kind = 'collection'">
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
          <select data-testid="prompt-model" v-model="model">
            <option value="">未指定</option>
            <option v-for="name in modelOptions" :key="name" :value="name">{{ name }}</option>
          </select>
        </label>
        <label class="field">
          <span>分类</span>
          <select data-testid="prompt-category" v-model="categoryId">
            <option value="">未分类</option>
            <optgroup v-for="group in groups" :key="group.id" :label="group.name">
              <option :value="group.id">{{ group.name }}（大分类）</option>
              <option v-for="child in group.children" :key="child.id" :value="child.id">
                {{ child.name }}
              </option>
            </optgroup>
          </select>
        </label>
        </div>
        <label v-if="kind === 'prompt'" class="field">
          <span>提示词内容</span>
          <textarea v-model="content" rows="8" placeholder="输入 {} 创建独立占位符，或 {{变量名}} 创建同名共用的变量"></textarea>
        </label>
        <label v-else class="field">
          <span>封面</span>
          <select v-model="coverType">
            <option value="none">无封面</option>
            <option value="single">单图</option>
            <option value="grid">九宫格</option>
          </select>
        </label>
        <label v-if="kind === 'collection' && coverType !== 'none'" class="field">
          <span>{{ coverType === "single" ? "封面图" : "封面图（最多 9 张，缺图用占位）" }}</span>
          <input
            type="file"
            accept="image/*"
            data-testid="cover-files"
            :multiple="coverType === 'grid'"
            @change="onCoverFiles"
          >
        </label>
      </div>
      <footer class="modal-footer">
        <button v-if="prompt" type="button" class="button danger-button" @click="$emit('remove', prompt.id)">
          删除
        </button>
        <span v-else class="create-location">将创建在本地库</span>
        <div class="modal-actions">
          <button type="button" class="button ghost-button" @click="$emit('cancel')">取消</button>
          <button type="button" class="button primary-button" :disabled="busy || !title.trim()" @click="submit">
            {{ busy ? '正在保存…' : kind === "collection" && !prompt ? "创建合集" : "保存" }}
          </button>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup>
import { computed, ref } from "vue";
import { vDialogFocus } from "../lib/dialogFocus.js";
import { parseCoverUrls } from "../lib/cover.js";

const props = defineProps({
  prompt: { type: Object, default: null },
  groups: { type: Array, default: () => [] },
  modelOptions: { type: Array, default: () => [] },
  defaultModel: { type: String, default: "" },
  defaultCategoryId: { type: String, default: "" },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
});

const emit = defineEmits(["cancel", "save", "remove"]);
const kind = ref(props.prompt?.kind ?? "prompt");
const title = ref(props.prompt?.title ?? "");
const content = ref(props.prompt?.content ?? "");
const categoryId = ref(props.prompt ? (props.prompt.category_id ?? "") : props.defaultCategoryId);
const model = ref(props.prompt ? (props.prompt.model ?? "") : (props.defaultModel ?? ""));
const coverType = ref(props.prompt?.cover_type ?? "none");
const coverUrls = ref(parseCoverUrls(props.prompt?.cover_json));
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
  const limit = coverType.value === "single" ? 1 : 9;
  const files = [...(event.target.files || [])].slice(0, limit);
  coverUrls.value = (await Promise.all(files.map(readAsDataUrl))).filter(Boolean);
}

function submit() {
  if (props.busy || !title.value.trim()) return;
  emit("save", {
    id: props.prompt?.id,
    kind: kind.value,
    title: title.value.trim(),
    content: content.value,
    categoryId: categoryId.value || null,
    model: kind.value === "prompt" ? model.value || null : null,
    coverType: coverType.value,
    coverUrls: coverType.value === "none" ? [] : coverUrls.value,
  });
}
</script>
