<template>
  <div class="modal-layer" data-testid="prompt-editor">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section class="modal create-modal" role="dialog" aria-modal="true">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">LOCAL PROMPT</p>
          <h2>{{ heading }}</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="create-body">
        <div v-if="!prompt" class="create-type-grid">
          <button type="button" class="create-type" :class="{ active: kind === 'prompt' }" @click="kind = 'prompt'">
            <strong>单个提示词</strong>
            <small>一条可直接使用的提示词。</small>
          </button>
          <button type="button" class="create-type" :class="{ active: kind === 'collection' }" @click="kind = 'collection'">
            <strong>提示词合集</strong>
            <small>同一主题下的一组提示词。</small>
          </button>
        </div>
        <label class="field">
          <span>{{ kind === "collection" ? "合集名称" : "标题" }}</span>
          <input v-model="title" placeholder="例如：SaaS 官网生成器">
        </label>
        <label class="field">
          <span>小分类</span>
          <select v-model="categoryId">
            <option value="">未分类</option>
            <optgroup v-for="group in groups" :key="group.id" :label="group.name">
              <option v-for="child in group.children" :key="child.id" :value="child.id">
                {{ child.name }}
              </option>
            </optgroup>
          </select>
        </label>
        <label v-if="kind === 'prompt'" class="field">
          <span>提示词内容</span>
          <textarea v-model="content" rows="8" placeholder="在正文中输入 {{变量名}} 即可创建变量"></textarea>
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
          <button type="button" class="button primary-button" :disabled="!title.trim()" @click="submit">
            {{ kind === "collection" ? "创建合集" : "保存" }}
          </button>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup>
import { computed, ref } from "vue";

const props = defineProps({
  prompt: { type: Object, default: null },
  groups: { type: Array, default: () => [] },
});

const emit = defineEmits(["cancel", "save", "remove"]);
const kind = ref("prompt");
const title = ref(props.prompt?.title ?? "");
const content = ref(props.prompt?.content ?? "");
const categoryId = ref(props.prompt?.category_id ?? "");
const coverType = ref("none");
const coverUrls = ref([]);
const heading = computed(() => {
  if (props.prompt) return "编辑提示词";
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
  if (!title.value.trim()) return;
  emit("save", {
    id: props.prompt?.id,
    kind: kind.value,
    title: title.value.trim(),
    content: content.value,
    categoryId: categoryId.value || null,
    coverType: coverType.value,
    coverUrls: coverType.value === "none" ? [] : coverUrls.value,
  });
}
</script>
