<template>
  <div class="review-workspace">
    <form class="review-filters" @submit.prevent="load(0, true)"><fieldset :disabled="busy || loading">
      <label>搜索投稿<input v-model="filters.q" placeholder="标题或来源标识" maxlength="300" data-testid="review-search"></label>
      <label>作者<input v-model="filters.author" placeholder="作者邮箱" maxlength="254"></label>
      <label>审核状态<select v-model="filters.status" data-testid="review-status"><option value="pending">待审核</option><option value="approved">已通过</option><option value="rejected">已驳回</option><option value="">全部状态</option></select></label>
      <label>提交起日（UTC）<input v-model="filters.from" type="date"></label><label>截止日（UTC）<input v-model="filters.to" type="date"></label>
      <button type="submit">查询</button>
    </fieldset></form>
    <p v-if="error" role="alert" data-testid="admin-error" class="error-banner">{{ error }} <button :disabled="busy || loading" @click="load(offset)">刷新列表</button></p>
    <p v-if="message" role="status" class="notice">{{ message }}</p>
    <section v-if="confirmation" ref="confirmPanel" class="panel review-confirm" aria-label="审核确认">
      <h2>{{ confirmation.status === 'rejected' ? '驳回投稿' : '批量通过' }} · {{ confirmation.ids.length }} 条</h2>
      <p class="muted">{{ confirmation.status === 'rejected' ? '原因会展示给投稿作者。不会删除或修改作者本地内容。' : '通过后会公开上架投稿快照。请确认已逐条检查内容。' }}</p>
      <label v-if="confirmation.status === 'rejected'">驳回原因<textarea v-model="reason" data-testid="review-reason" maxlength="1000" rows="3" :disabled="busy" placeholder="说明需要修改的具体问题" /></label>
      <div class="review-actions"><button class="primary" data-testid="review-confirm" :disabled="busy || (confirmation.status === 'rejected' && !reason.trim())" @click="confirm">{{ busy ? '正在审核…' : '确认审核' }}</button><button :disabled="busy" @click="cancel">取消</button></div>
    </section>
    <section v-if="results.length" class="panel review-results" aria-label="批量结果"><h2>本次批量结果</h2><ul><li v-for="result in results" :key="result.id"><strong>{{ result.id }}</strong><span :class="result.ok ? 'success-message' : 'error-message'">{{ result.ok ? '成功' : `${result.message}（${result.status}）` }}</span></li></ul></section>
    <section class="panel">
      <div class="panel-heading"><h2>{{ filtersApplied.status === 'pending' ? '待审核内容' : '审核记录' }}</h2><span class="badge">{{ total }} 条</span><button :disabled="busy || loading" @click="load(offset)">刷新</button></div>
      <div class="review-batch"><label><input type="checkbox" :checked="allSelected" :disabled="busy || loading || !pendingIds.length" @change="selected = $event.target.checked ? [...pendingIds] : []">选择本页待审</label><span class="muted">已选 {{ selected.length }} 条</span><button :disabled="busy || loading || !selected.length" data-testid="review-batch-approve" @click="prepare(selected, 'approved', true)">批量通过</button><button :disabled="busy || loading || !selected.length" @click="prepare(selected, 'rejected', true)">批量驳回</button></div>
      <p v-if="loading" class="empty-state" role="status">正在加载投稿…</p>
      <ul v-else data-testid="review-list" class="review-list"><li v-for="item in items" :key="item.id" class="review-row">
        <div class="review-summary"><label class="review-title"><input v-if="item.status === 'pending'" v-model="selected" type="checkbox" :value="item.id" :aria-label="`选择 ${item.title || item.source_id}`" :disabled="busy"><strong>{{ item.title || item.source_id }}</strong></label><div><span class="badge">{{ item.kind === 'collection' ? '合集' : '提示词' }}</span><span class="badge pending">{{ labels[item.status] || item.status }} · {{ item.status }}</span></div><small class="muted">{{ item.author_email || '作者未记录' }} · {{ date(item.created_at) }}</small></div>
        <div v-if="item.status === 'pending'" class="review-actions"><button data-testid="review-reject" :disabled="busy" @click="prepare([item.id], 'rejected')">驳回</button><button data-testid="review-approve" class="primary" :disabled="busy" @click="single(item.id, 'approved')">通过</button></div>
        <div v-if="item.moderation" class="review-content risk-rule-result"><strong>初筛 {{ item.moderation.score }} 分 · {{ item.moderation.decision==='approved'?'按策略自动通过':'转人工' }}</strong><small>策略版本 {{ item.moderation.policy_revision }} · 规则版本 {{ item.moderation.rules_revision }}</small><p>{{ item.moderation.notice }}</p><p v-for="(reason,index) in item.moderation.reasons" :key="index">{{ reason }}</p><p v-for="hit in item.moderation.rules?.hits" :key="hit.id">{{ hit.name }}：{{ hit.words.join('、') }}</p><details v-if="item.moderation.ai"><summary>AI 与 Skill 结果 · 配置版本 {{ item.moderation.ai.revision }}</summary><p>{{ item.moderation.ai.error }}</p><div v-for="run in item.moderation.ai.runs" :key="run.skill_id"><strong>{{ run.skill_id }}：{{ run.error || run.verdict?.decision }}</strong><p v-for="(model,index) in run.models" :key="index">{{ model.id }} · {{ model.model }}：{{ model.error || `${model.result?.decision} / ${model.result?.risk_score}` }}</p><p v-for="reason in run.verdict?.reasons || []" :key="reason">{{ reason }}</p></div></details></div>
        <details class="review-content" data-testid="review-content"><summary>查看投稿快照与记录</summary><p class="muted">来源 {{ item.source_id }} · {{ item.category_id || '未分类' }} · {{ item.model || '通用模型' }}</p><template v-if="item.kind === 'collection'"><article v-for="(member, index) in item.members" :key="index"><h3>{{ member.title }}</h3><p class="muted">{{ member.category_id }} · {{ member.model }}</p><pre>{{ member.content }}</pre><ReviewAttachments :publication-id="item.id" :references="(item.asset_refs || []).filter(file => member.asset_ids?.includes(file.id))" /></article></template><pre v-else>{{ item.content || '未提供正文快照' }}</pre><ReviewAttachments v-if="item.kind !== 'collection'" :publication-id="item.id" :references="item.asset_refs || []" /><h3>审核历史</h3><ul v-if="item.history?.length" class="review-history"><li v-for="event in item.history" :key="event.id"><strong>{{ labels[event.status] }}</strong><small>{{ date(event.created_at) }} · {{ event.actor_email }}</small><p v-if="event.reason">{{ event.reason }}</p></li></ul><p v-else class="muted">{{ item.status === 'pending' ? '尚未审核' : '历史审核详情未记录' }}</p></details>
      </li><li v-if="!items.length" class="empty-state"><span class="empty-icon">✓</span><h3>{{ error ? '列表暂不可用' : '没有符合条件的投稿' }}</h3><p>可调整筛选条件，或稍后刷新。</p></li></ul>
      <footer class="users-pagination"><span>{{ total ? offset + 1 : 0 }}–{{ Math.min(offset + items.length, total) }} / {{ total }}</span><button :disabled="busy || loading || offset === 0" @click="load(offset - limit)">上一页</button><button :disabled="busy || loading || offset + limit >= total" data-testid="reviews-next" @click="load(offset + limit)">下一页</button></footer>
    </section>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { listState } from './listState.js';
import ReviewAttachments from './ReviewAttachments.vue';
import { approvePublication, batchReviewPublications, listPendingPublications, rejectPublication } from './adminApi.js';
const emit = defineEmits(['busy-change']);
const labels = { pending: '待审核', approved: '已通过', rejected: '已驳回' };
const filters = ref({ q: '', author: '', status: 'pending', from: '', to: '' }), filtersApplied = ref({ ...filters.value });
const items = ref([]), total = ref(0), offset = ref(0), loading = ref(false), busy = ref(false), error = ref(''), message = ref('');
const selected = ref([]), confirmation = ref(null), reason = ref(''), results = ref([]);
const confirmPanel = ref(null);
const limit = 25;
const memory = listState('review'), previous = memory.read();
if (previous.filters) filters.value = { ...previous.filters };
filtersApplied.value = { ...filters.value }; offset.value = previous.offset || 0;
let version = 0;
const pendingIds = computed(() => items.value.filter(item => item.status === 'pending').map(item => item.id));
const allSelected = computed(() => pendingIds.value.length > 0 && pendingIds.value.every(id => selected.value.includes(id)));
defineExpose({ hasUnsavedChanges: computed(() => Boolean(confirmation.value)), isBusy: busy });
function date(value) { return value ? new Date(value).toLocaleString('zh-CN') : '历史时间未记录'; }
function canDiscard() { return !confirmation.value || window.confirm('放弃当前审核确认和未保存原因？'); }
async function load(start = offset.value, apply = false) {
  if (busy.value || !canDiscard()) return;
  const query = apply ? { ...filters.value } : { ...filtersApplied.value };
  if (query.from && query.to && query.from > query.to) { error.value = '起日不能晚于截止日'; return; }
  cancel(); selected.value = []; const current = ++version; loading.value = true; error.value = '';
  try { const result = await listPendingPublications({ ...query, offset: start, limit }); if (current !== version) return; items.value = result.items ?? []; total.value = result.total ?? items.value.length; offset.value = start; filtersApplied.value = query; memory.save({ filters: query, offset: start }); }
  catch (caught) { if (current === version) error.value = caught.message; }
  finally { if (current === version) loading.value = false; }
}
async function prepare(ids, status, batch = false) {
  if (busy.value || !canDiscard()) return;
  confirmation.value = { ids: [...ids], status, batch }; reason.value = ''; error.value = ''; message.value = '';
  await nextTick(); confirmPanel.value?.scrollIntoView?.({ block: 'nearest' }); confirmPanel.value?.querySelector('textarea, button')?.focus();
}
function cancel() { if (busy.value) return; confirmation.value = null; reason.value = ''; }
function setBusy(value) { busy.value = value; emit('busy-change', value); }
function removeResolved(ids, status) {
  selected.value = selected.value.filter(id => !ids.includes(id));
  if (filtersApplied.value.status && filtersApplied.value.status !== status) { const count = items.value.filter(item => ids.includes(item.id)).length; items.value = items.value.filter(item => !ids.includes(item.id)); total.value = Math.max(0, total.value - count); }
  else items.value = items.value.map(item => ids.includes(item.id) ? { ...item, status } : item);
}
async function single(id, status) {
  if (status === 'approved' && items.value.find(item => item.id === id)?.asset_refs?.length && !confirmation.value) return prepare([id], status);
  if (busy.value || (status === 'approved' && confirmation.value?.ids[0] !== id && !canDiscard())) return;
  setBusy(true); error.value = ''; message.value = '';
  try { const result = status === 'approved' ? await approvePublication(id) : await rejectPublication(id, reason.value.trim()); if (result.status !== status) throw new Error('服务端未确认审核结果，请刷新检查'); removeResolved([id], status); confirmation.value = null; reason.value = ''; message.value = '审核已保存，可切换状态并刷新查看历史。'; }
  catch (caught) { error.value = caught.message; }
  finally { setBusy(false); }
}
async function confirm() {
  if (busy.value || !confirmation.value) return;
  const input = confirmation.value;
  if (input.status === 'rejected' && !reason.value.trim()) return;
  if (!input.batch) return single(input.ids[0], input.status);
  setBusy(true); error.value = ''; message.value = ''; results.value = [];
  try {
    const result = await batchReviewPublications({ ids: input.ids, status: input.status, ...(input.status === 'rejected' ? { reason: reason.value.trim() } : {}) });
    if (!Array.isArray(result.results) || result.results.length !== input.ids.length || new Set(result.results.map(row => row.id)).size !== input.ids.length || result.results.some(row => !input.ids.includes(row.id) || typeof row.ok !== 'boolean')) throw new Error('批量响应不完整，请刷新确认后重试');
    results.value = result.results; const succeeded = result.results.filter(row => row.ok).map(row => row.id); removeResolved(succeeded, input.status);
    message.value = `成功 ${succeeded.length} 条，失败 ${input.ids.length - succeeded.length} 条。`; confirmation.value = null; reason.value = '';
  } catch (caught) { error.value = caught.message; }
  finally { setBusy(false); }
}
onMounted(() => load(offset.value));
onUnmounted(() => { ++version; });
</script>

<style scoped>
.review-workspace { display: grid; gap: 18px; }
.review-filters fieldset { border: 0; padding: 0; margin: 0; display: flex; flex-wrap: wrap; align-items: end; gap: 12px; }
.review-filters label { flex: 1 1 160px; min-width: 0; }
.review-filters input, .review-filters select { width: 100%; min-width: 0; }
.review-batch { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; padding: 16px 24px; border-bottom: 1px solid var(--line); }
.review-batch label, .review-title { display: flex; flex-direction: row; align-items: center; gap: 10px; margin: 0; }
.review-title { font-size: 15px; color: inherit; }
input[type=checkbox] { width: 16px; height: 16px; flex: none; margin: 0; }
.review-filters select, .review-confirm textarea { font: inherit; border: 1px solid var(--line-strong); border-radius: 7px; padding: 10px 12px; background: #fff; color: var(--text); margin-top: 8px; }
.review-confirm, .review-results { padding: 24px; }
.review-confirm textarea { width: 100%; resize: vertical; }
.review-confirm .review-actions { margin-top: 16px; }
.review-history { list-style: none; padding: 0; }
.review-history li { padding: 12px 0; display: grid; gap: 6px; }
.review-history p { white-space: pre-wrap; overflow-wrap: anywhere; }
.review-history small { color: var(--muted); }
.review-results ul { list-style: none; padding: 0; }
.review-results li { display: flex; flex-wrap: wrap; gap: 16px; padding: 8px 0; overflow-wrap: anywhere; }
.review-summary { min-width: 0; overflow-wrap: anywhere; }
.panel-heading { gap: 12px; }
.panel-heading .badge { margin-right: auto; }
.users-pagination { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 10px; padding: 16px 24px; border-top: 1px solid var(--line); }
.users-pagination > span { margin-right: auto; color: var(--muted); }
@media(max-width: 640px) { .review-batch, .review-confirm, .review-results { padding: 16px; } .review-row { display: flex; flex-direction: column; align-items: stretch; } }
</style>
