<template>
  <div class="modal-layer" data-testid="collection-detail">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section class="modal create-modal" role="dialog" aria-modal="true">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">LOCAL COLLECTION</p>
          <h2>{{ collection.title }}</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="create-body">
        <div v-if="collection.cover_type === 'single' && singleCover" class="cover-single">
          <img :src="singleCover" alt="">
        </div>
        <div v-else-if="collection.cover_type === 'grid'" class="cover-grid" data-testid="cover-grid">
          <i v-for="(src, index) in coverCells" :key="index" :class="{ filled: Boolean(src) }">
            <img v-if="src" :src="src" alt="">
          </i>
        </div>
        <p class="use-hint">{{ members.length }} 个提示词。缺图不会阻止打开详情。</p>
        <ul class="member-list">
          <li v-for="member in members" :key="member.id">{{ member.title }}</li>
        </ul>
        <label class="field">
          <span>加入已有提示词</span>
          <select v-model="selectedPromptId">
            <option value="">选择一条</option>
            <option v-for="prompt in available" :key="prompt.id" :value="prompt.id">
              {{ prompt.title }}
            </option>
          </select>
        </label>
      </div>
      <footer class="modal-footer">
        <span class="create-location">合集不出现在侧栏树中</span>
        <div class="modal-actions">
          <button type="button" class="button ghost-button" @click="$emit('cancel')">关闭</button>
          <button type="button" class="button primary-button" :disabled="!selectedPromptId" @click="add">
            加入合集
          </button>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup>
import { computed, ref } from "vue";
import { coverSlots, parseCoverUrls } from "../lib/cover.js";

const props = defineProps({
  collection: { type: Object, required: true },
  members: { type: Array, default: () => [] },
  prompts: { type: Array, default: () => [] },
});
const emit = defineEmits(["cancel", "add"]);
const selectedPromptId = ref("");
const available = computed(() =>
  props.prompts.filter((prompt) => prompt.collection_id !== props.collection.id),
);
const coverCells = computed(() => coverSlots(props.collection.cover_json, 9));
const singleCover = computed(() => parseCoverUrls(props.collection.cover_json)[0] || "");

function add() {
  if (!selectedPromptId.value) return;
  emit("add", selectedPromptId.value);
  selectedPromptId.value = "";
}
</script>
