<template>
    <section v-page-focus="() => !busy && $emit('cancel')" class="workspace-page" data-testid="collection-detail" role="region" aria-labelledby="collection-title" :aria-busy="busy || loading">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">提示词合集</p>
          <h2 id="collection-title">{{ collection.title }}</h2>
        </div>
        <button type="button" class="page-back" aria-label="返回" :disabled="busy" @click="$emit('cancel')">← 返回</button>
      </header>
      <div class="create-body" :inert="busy ? '' : undefined">
        <p v-if="error" role="alert" class="use-hint">{{ error }}</p>
        <p v-if="loading" role="status" class="use-hint">正在读取合集…</p>
        <button v-else-if="!ready" type="button" class="button ghost-button" data-testid="retry-collection-load" :disabled="busy" @click="$emit('retry')">重新读取</button>
        <template v-if="ready">
        <div v-if="collection.cover_type === 'single' && singleCover" class="cover-single">
          <img :src="singleCover" alt="">
        </div>
        <div v-else-if="collection.cover_type === 'grid'" class="cover-grid" data-testid="cover-grid">
          <i v-for="(src, index) in coverCells" :key="index" :class="{ filled: Boolean(src) }">
            <img v-if="src" :src="src" alt="">
          </i>
        </div>
        <p class="use-hint">{{ members.length }} 个提示词</p>
        <div v-if="!members.length" class="collection-empty"><strong>把相关提示词放在一起</strong><p>从下方选择本地提示词，开始整理这个合集。</p></div>
        <ul class="member-list">
          <li v-for="member in members" :key="member.id">
            <button type="button" class="member-title" :title="member.title" @click="$emit('open', member)">{{ member.title }}</button>
            <span class="member-actions">
            <button type="button" class="card-action" @click="$emit('use', member)">使用</button>
            <button type="button" class="card-action" data-testid="remove-member" @click="$emit('remove-member', member.id)">移出合集</button>
            </span>
          </li>
        </ul>
        <label class="field">
          <span>加入已有提示词</span>
          <select v-model="selectedPromptId">
            <option value="">{{ available.length ? '选择一条本地提示词' : '暂无可加入的提示词' }}</option>
            <option v-for="prompt in available" :key="prompt.id" :value="prompt.id">
              {{ prompt.title }}
            </option>
          </select>
        </label>
        </template>
      </div>
      <footer class="modal-footer">
        <button type="button" class="button ghost-button" data-testid="edit-collection" :disabled="busy || !ready" @click="$emit('edit')">编辑合集</button>
        <div class="modal-actions">
          <button type="button" class="button ghost-button" :disabled="busy" @click="$emit('cancel')">返回</button>
          <button type="button" class="button primary-button" :disabled="busy || !ready || !selectedPromptId" @click="add">
            加入合集
          </button>
        </div>
      </footer>
    </section>
</template>

<script setup>
import { computed, ref, watch } from "vue";
import { vPageFocus } from "../lib/pageFocus.js";
import { coverSlots, parseCoverUrls } from "../lib/cover.js";

const props = defineProps({
  collection: { type: Object, required: true },
  members: { type: Array, default: () => [] },
  prompts: { type: Array, default: () => [] },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },
  ready: { type: Boolean, default: true },
});
const emit = defineEmits(["cancel", "add", "open", "use", "remove-member", "edit", "retry"]);
const selectedPromptId = ref("");
const available = computed(() =>
  props.prompts.filter((prompt) => prompt.collection_id !== props.collection.id),
);
const coverCells = computed(() => coverSlots(props.collection.cover_json, 9));
const singleCover = computed(() => parseCoverUrls(props.collection.cover_json)[0] || "");
watch([() => props.ready, available], () => {
  if (props.ready && !available.value.some(prompt => prompt.id === selectedPromptId.value)) selectedPromptId.value = "";
});

function add() {
  if (props.busy || !props.ready || !selectedPromptId.value) return;
  emit("add", selectedPromptId.value);
}
</script>
