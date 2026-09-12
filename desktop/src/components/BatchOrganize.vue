<template>
  <section class="batch-organize" aria-label="批量整理" :aria-busy="busy">
    <div class="batch-controls">
      <strong role="status">已选 {{ prompts.length }} 条</strong>
      <button type="button" class="button" data-testid="select-page" :disabled="busy || !canSelectPage" @click="$emit('select-page')">选择本页</button>
      <button type="button" class="button ghost-button" data-testid="clear-selection" :disabled="busy || !prompts.length" @click="$emit('clear-selection')">清空选择</button>
      <label>操作<select v-model="mode" :disabled="busy"><option value="category">移动分类</option><option value="collection">加入合集</option></select></label>
      <label v-if="mode === 'category'">目标分类<select v-model="categoryId" :disabled="busy" data-testid="batch-category">
        <option value="">未分类</option><optgroup v-for="group in groups" :key="group.id" :label="group.name"><option :value="group.id">{{ group.name }}</option><option v-for="child in group.children" :key="child.id" :value="child.id">{{ child.name }}</option></optgroup>
      </select></label>
      <label v-else>目标合集<select v-model="collectionId" :disabled="busy" data-testid="batch-collection"><option value="">选择合集</option><option v-for="collection in collections" :key="collection.id" :value="collection.id">{{ collection.title }}</option></select></label>
      <button type="button" class="button primary-button" data-testid="apply-batch" :disabled="busy || !prompts.length || (mode === 'collection' && !collectionId)" @click="apply">{{ busy ? '正在整理…' : '应用到所选' }}</button>
    </div>
    <p class="use-hint">翻页保留选择，切换筛选会清空。{{ mode === 'collection' ? '一条提示词只能属于一个合集，已归属其他合集的条目会移入目标合集。' : '只移动所选提示词的分类，正文与附件保持不变。' }}</p>
    <p v-if="result" role="status" data-testid="batch-result">{{ result }}</p>
    <ul v-if="errors.length" role="alert"><li v-for="error in errors" :key="error">{{ error }}</li></ul>
  </section>
</template>
<script setup>
import { ref } from 'vue';
import { addPromptToCollection, moveLocalPromptCategory } from '../platform/library.js';
const props = defineProps({ prompts: { type: Array, required: true }, groups: Array, collections: Array, canSelectPage: Boolean, complete: { type: Function, required: true } });
const emit = defineEmits(['busy', 'select-page', 'clear-selection']);
const mode = ref('category'), categoryId = ref(''), collectionId = ref(''), busy = ref(false), errors = ref([]), result = ref('');
async function apply() {
  if (busy.value || !props.prompts.length || (mode.value === 'collection' && !collectionId.value)) return;
  const rows = [...props.prompts], completed = [];
  busy.value = true; emit('busy', true); errors.value = []; result.value = '';
  for (const row of rows) {
    try {
      if (mode.value === 'category') await moveLocalPromptCategory(row.id, categoryId.value || null);
      else await addPromptToCollection(row.id, collectionId.value);
      completed.push(row.id);
    } catch (error) { errors.value.push(`${row.title}：${error.message || error}`); }
  }
  result.value = `已完成 ${completed.length} 条${errors.value.length ? `，失败 ${errors.value.length} 条；失败项仍保留选择，可重试。` : '。'}`;
  try { await props.complete(completed); }
  finally { busy.value = false; emit('busy', false); }
}
</script>
<style scoped>
.batch-organize { margin: 0 28px 16px; padding: 16px; border: 1px solid var(--line); border-radius: 10px; background: var(--sidebar); }
.batch-controls { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; }
.batch-controls label { display: flex; align-items: center; gap: 8px; font-size: 12px; }
select { max-width: 220px; padding: 7px; border: 1px solid var(--line); border-radius: 6px; color: var(--text); background: var(--surface); }
strong { font-size: 13px; } p, li { font-size: 12px; }
</style>
