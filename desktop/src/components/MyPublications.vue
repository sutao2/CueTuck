<template>
  <section v-page-focus="() => $emit('cancel')" class="workspace-page" data-testid="publications-page" aria-labelledby="publications-title">
    <header class="modal-header"><div><p class="modal-kicker">账号</p><h2 id="publications-title">我的发布</h2></div><button type="button" class="page-back" @click="$emit('cancel')">← 返回</button></header>
    <div class="create-body" data-testid="my-publications">
      <p class="use-hint">查看已提交到广场的内容、审核结果与驳回说明。</p>
      <p v-if="!session.loggedIn">登录后查看自己的投稿。<button type="button" class="button" @click="$emit('login')">登录</button></p>
      <p v-else-if="loading" role="status">正在读取投稿…</p>
      <p v-else-if="error" role="alert">{{ error }} <button type="button" class="button" @click="load">重试</button></p>
      <p v-else-if="!rows.length">暂无投稿。可以从广场的发布入口提交本地提示词。</p>
      <ul v-else class="publication-list">
        <li v-for="row in rows" :key="row.id">
          <div class="publication-heading"><strong>{{ row.title || row.source_id }}</strong><span class="publication-status">{{ statusLabel(row.status) }}</span></div>
          <p v-if="row.visibility">广场：{{ { online: '已上架', offline: '已下架', trashed: '已移入回收站' }[row.visibility] || row.visibility }}</p>
          <p v-for="event in row.history || []" :key="event.id">{{ event.status === 'rejected' ? '驳回原因：' : '审核通过' }}{{ event.reason || '' }}<small v-if="event.created_at"> · {{ new Date(event.created_at).toLocaleString() }}</small></p>
        </li>
      </ul>
    </div>
    <footer class="modal-footer"><span class="use-hint">{{ session.email }}</span><button type="button" class="button" :disabled="loading || !session.loggedIn" @click="load">刷新</button></footer>
  </section>
</template>
<script setup>
import { ref, watch, onUnmounted } from 'vue';
import { listMyPublications } from '../platform/square.js';
import { vPageFocus } from '../lib/pageFocus.js';
const props = defineProps({ session: { type: Object, required: true } });
defineEmits(['cancel', 'login']);
const rows = ref([]), loading = ref(false), error = ref('');
let request = 0;
const statusLabel = value => ({ pending: '待审核', approved: '已通过', rejected: '需修改' }[value] || value);
async function load() {
  const token = ++request;
  rows.value = []; error.value = ''; loading.value = false;
  if (!props.session.loggedIn) return;
  loading.value = true;
  try { const result = await listMyPublications(); if (token === request) rows.value = result; }
  catch (err) { if (token === request) error.value = `读取失败：${err.message || err}`; }
  finally { if (token === request) loading.value = false; }
}
watch(() => [props.session.email, props.session.loggedIn], load, { immediate: true });
onUnmounted(() => { ++request; });
</script>
<style scoped>
.publication-list { list-style: none; padding: 0; margin: 20px 0; }
.publication-list li { border-bottom: 1px solid var(--line); padding: 20px 0; overflow-wrap: anywhere; }
.publication-heading { display: flex; justify-content: space-between; align-items: start; gap: 16px; }
.publication-status { flex-shrink: 0; font-size: 12px; padding: 4px 8px; background: var(--sidebar); border-radius: 6px; }
p { font-size: 13px; line-height: 1.7; } small { color: var(--muted); }
</style>
