<template>
    <section v-page-focus="() => !busy && $emit('cancel')" class="workspace-page" data-testid="collection-detail" role="region" aria-labelledby="collection-title" :aria-busy="busy || loading">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">提示词合集</p>
          <h2 id="collection-title">{{ collection.title }}</h2>
        </div>
        <button type="button" class="page-back" aria-label="返回" :disabled="busy" @click="$emit('cancel')">← 返回</button>
      </header>
      <div class="create-body collection-body" :inert="busy ? '' : undefined">
        <p v-if="error" role="alert" class="use-hint">{{ error }}</p>
        <p v-if="loading" role="status" class="use-hint">正在读取合集…</p>
        <button v-else-if="!ready" type="button" class="button ghost-button" data-testid="retry-collection-load" :disabled="busy" @click="$emit('retry')">重新读取</button>
        <template v-if="ready">
        <CollectionCover :type="collection.cover_type" :json="collection.cover_json" />
        <div class="collection-members-heading"><p class="use-hint">{{ members.length }} 个提示词</p><button type="button" class="button primary-button" data-testid="create-collection-prompt" :disabled="busy || loading" @click="$emit('create')">＋ 新建提示词</button></div>
        <div v-if="!members.length" class="collection-empty"><strong>把相关提示词放在一起</strong><p>从下方选择本地提示词，或新建仅属于这个合集的提示词。</p></div>
        <ul class="member-list">
          <li v-for="member in members" :key="member.id">
            <button type="button" class="member-title" :title="member.title" @click="$emit('open', member)">{{ member.title }}</button>
            <span class="member-actions">
            <button type="button" class="card-action" @click="$emit('use', member)">使用</button>
            <button type="button" class="card-action" data-testid="remove-member" @click="$emit('remove-member', member.id)">移出合集</button>
            </span>
          </li>
        </ul>
        <div class="member-picker">
          <label class="field"><span>搜索并加入提示词</span><input v-model="memberQuery" type="search" placeholder="搜索本地提示词标题" /></label>
          <p class="use-hint">可多选；已有合集的提示词会移动到本合集，正文保持不变。移出合集会保留为独立提示词。</p>
          <div class="member-choices">
            <label v-for="prompt in filteredAvailable" :key="prompt.id">
              <input v-model="selectedPromptIds" type="checkbox" :value="prompt.id" :data-member-choice="prompt.id" />
              <span>{{ prompt.title }}<small v-if="prompt.collection_id">将从原合集移入</small></span>
            </label>
            <p v-if="!filteredAvailable.length" role="status">{{ memberQuery ? '没有匹配的提示词' : '暂无可加入的提示词' }}</p>
          </div>
        </div>
        </template>
      </div>
      <footer class="modal-footer">
        <button type="button" class="button ghost-button" data-testid="edit-collection" :disabled="busy || !ready" @click="$emit('edit')">编辑合集</button>
        <div class="modal-actions">
          <button type="button" class="button ghost-button" :disabled="busy" @click="$emit('cancel')">返回</button>
          <button type="button" class="button primary-button" data-testid="add-collection-members" :disabled="busy || !ready || !selectedPromptIds.length" @click="add">
            加入合集
          </button>
        </div>
      </footer>
    </section>
</template>

<script setup>
import { computed, ref, watch } from "vue";
import { vPageFocus } from "../lib/pageFocus.js";
import CollectionCover from "./CollectionCover.vue";

const props = defineProps({
  collection: { type: Object, required: true },
  members: { type: Array, default: () => [] },
  prompts: { type: Array, default: () => [] },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },
  ready: { type: Boolean, default: true },
});
const emit = defineEmits(["cancel", "add", "open", "use", "remove-member", "edit", "retry", "create"]);
const selectedPromptIds = ref([]), memberQuery = ref('');
const available = computed(() =>
  props.prompts.filter((prompt) => prompt.collection_id !== props.collection.id),
);
const filteredAvailable = computed(() => available.value.filter(p => p.title.toLowerCase().includes(memberQuery.value.trim().toLowerCase())));
watch([() => props.ready, available], () => {
  if (props.ready) selectedPromptIds.value = selectedPromptIds.value.filter(id => available.value.some(p => p.id === id));
});
function add() {
  if (props.busy || !props.ready || !selectedPromptIds.value.length) return;
  emit('add', [...selectedPromptIds.value]);
}
</script>

<style scoped>
.collection-body { display: block; }
.collection-body > * + * { margin-top: 18px; }
.collection-members-heading { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin: 20px 0 8px; }
.member-picker { margin-top: 24px; }
.member-choices { max-height: 240px; overflow: auto; border: 1px solid var(--line); border-radius: 8px; padding: 8px; }
.member-choices label { display: flex; align-items: center; gap: 10px; padding: 9px; font-size: 13px; }
.member-choices input { width: 16px; height: 16px; flex-shrink: 0; }
.member-choices span { overflow-wrap: anywhere; min-width: 0; }
.member-choices small { display: block; color: var(--muted); font-size: 11px; margin-top: 3px; }
</style>
