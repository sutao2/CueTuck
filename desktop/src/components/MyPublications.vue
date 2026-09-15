<template>
  <section v-page-focus="close" class="workspace-page" data-testid="publications-page" aria-labelledby="publications-title">
    <header class="modal-header"><div><p class="modal-kicker">账号</p><h2 id="publications-title">我的发布</h2></div><button type="button" class="page-back"  :disabled="busy" @click="close">← 返回</button></header>
    <div class="create-body" data-testid="my-publications">
      <nav class="skills-market-tabs" aria-label="发布内容类型"><button :disabled="busy" :aria-current="kind==='prompts'?'page':undefined" @click="kind='prompts'">提示词</button><button :disabled="busy" :aria-current="kind==='skills'?'page':undefined" @click="kind='skills'">Skills</button></nav>
      <p class="use-hint">查看已提交到广场的内容、审核结果与驳回说明。</p>
      <p v-if="!session.loggedIn">登录后查看自己的投稿。<button type="button" class="button" @click="$emit('login')">登录</button></p>
      <SkillCommunity v-else-if="kind==='skills'" :key="session.email" mine @busy="skillBusy=$event"/>
      <p v-else-if="loading" role="status">正在读取投稿…</p>
      <p v-else-if="error" role="alert">{{ error }} <button type="button" class="button" @click="load">重试</button></p>
      <template v-else>
      <div class="publication-metrics" data-testid="publication-metrics">
        <div><strong>{{ formatMetric(rows.length) }}</strong><span>累计投稿</span></div>
        <div><strong>{{ formatMetric(totalMetric('download_count')) }}</strong><span>已记录下载次数</span></div>
        <div><strong>{{ formatMetric(totalMetric('favorite_count')) }}</strong><span>当前收藏总次数</span></div>
      </div>
      <div class="publication-toolbar">
        <p class="use-hint">下载仅统计开启匿名上报后的成功下载；收藏为各作品当前收藏人数之和，非去重粉丝数。</p>
        <label>排序 <select v-model="sort" aria-label="作品排序"><option value="latest">最新提交</option><option value="download_count">下载最多</option><option value="favorite_count">收藏最多</option></select></label>
      </div>
      <p v-if="!rows.length">暂无投稿。可以从广场的发布入口提交本地提示词。</p>
      <ul v-else class="publication-list">
        <li v-for="row in sortedRows" :key="row.id">
          <div class="publication-heading"><strong>{{ row.title || row.source_id }}</strong><span class="publication-status">{{ statusLabel(row.status) }}</span></div>
          <p class="publication-counters"><span>记录下载 {{ formatMetric(row.download_count) }}</span><span>当前收藏 {{ formatMetric(row.favorite_count) }}</span></p>
          <p v-if="row.visibility">广场：{{ { online: '已上架', offline: '已下架', trashed: '已移入回收站' }[row.visibility] || row.visibility }}</p>
          <p v-for="event in row.history || []" :key="event.id">{{ event.status === 'rejected' ? '驳回原因：' : '审核通过' }}{{ event.reason || '' }}<small v-if="event.created_at"> · {{ new Date(event.created_at).toLocaleString() }}</small></p>
        </li>
      </ul>
      </template>
    </div>
    <footer class="modal-footer"><span class="use-hint">{{ session.email }}</span><button v-if="kind==='prompts'" type="button" class="button" :disabled="loading || !session.loggedIn" @click="load">刷新</button></footer>
  </section>
</template>
<script setup>
import { computed, ref, watch, onUnmounted } from 'vue';
import SkillCommunity from './SkillCommunity.vue';
import { formatMetric } from '../platform/contentMetrics.js';
import { listMyPublications } from '../platform/square.js';
import { vPageFocus } from '../lib/pageFocus.js';
const props = defineProps({ session: { type: Object, required: true }, initialKind:{type:String,default:'prompts'} });
const emit=defineEmits(['cancel', 'login','busy']);
const kind=ref(props.initialKind),skillBusy=ref(false);
const busy=computed(()=>loading.value||skillBusy.value);
function close(){if(!busy.value)emit('cancel');}
const rows = ref([]), loading = ref(false), error = ref('');
const sort = ref('latest');
function totalMetric(key) {
  return rows.value.every(row => Number.isSafeInteger(row[key]) && row[key] >= 0)
    ? rows.value.reduce((total, row) => total + row[key], 0) : undefined;
}
const sortedRows = computed(() => sort.value === 'latest' ? rows.value : [...rows.value].sort((a, b) => {
  const value = row => Number.isSafeInteger(row[sort.value]) && row[sort.value] >= 0 ? row[sort.value] : -1;
  return value(b) - value(a);
}));
let request = 0;
const statusLabel = value => ({ pending: '待审核', approved: '已通过', rejected: '需修改' }[value] || value);
async function load() {
  const token = ++request;
  rows.value = []; error.value = ''; loading.value = false;
  if (!props.session.loggedIn || kind.value!=='prompts') return;
  loading.value = true;
  try { const result = await listMyPublications(); if (token === request) rows.value = result; }
  catch (err) { if (token === request) error.value = `读取失败：${err.message || err}`; }
  finally { if (token === request) loading.value = false; }
}
watch(busy,value=>emit('busy',value),{flush:'sync'});
watch(kind,()=>{if(kind.value==='prompts')load();});
watch(() => [props.session.email, props.session.loggedIn], load, { immediate: true });
onUnmounted(() => { ++request;emit('busy',false); });
</script>
<style scoped src="./skills.css"></style>
<style scoped>
.publication-metrics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border: 1px solid var(--line); border-radius: 14px; margin: 24px 0 18px; }
.publication-metrics > div { display: grid; gap: 8px; padding: 22px; }
.publication-metrics > div + div { border-left: 1px solid var(--line); }
.publication-metrics strong { font-size: 26px; font-weight: 550; font-variant-numeric: tabular-nums; }
.publication-metrics span, .publication-counters { color: var(--muted); font-size: 12px; }
.publication-toolbar { display: flex; align-items: center; gap: 20px; justify-content: space-between; }
.publication-toolbar p { max-width: 560px; }
.publication-toolbar label { white-space: nowrap; font-size: 12px; }
.publication-toolbar select { border: 1px solid var(--line); border-radius: 7px; background: var(--surface); color: var(--text); padding: 7px; }
.publication-counters { display: flex; gap: 24px; font-variant-numeric: tabular-nums; }
@media (max-width: 700px) { .publication-toolbar { align-items: start; flex-direction: column; gap: 8px; } .publication-metrics > div { padding: 14px; } .publication-metrics strong { font-size: 22px; } }

.publication-list { list-style: none; padding: 0; margin: 20px 0; }
.publication-list li { border-bottom: 1px solid var(--line); padding: 20px 0; overflow-wrap: anywhere; }
.publication-heading { display: flex; justify-content: space-between; align-items: start; gap: 16px; }
.publication-status { flex-shrink: 0; font-size: 12px; padding: 4px 8px; background: var(--sidebar); border-radius: 6px; }
p { font-size: 13px; line-height: 1.7; } small { color: var(--muted); }
</style>
