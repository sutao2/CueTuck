<template>
  <div class="content-workspace">
    <form class="content-filters" @submit.prevent="load(0, true)"><label>搜索内容<input v-model="query" placeholder="标题或内容 ID" maxlength="300" data-testid="content-query" :disabled="busy"></label><label>可见状态<select v-model="visibility" :disabled="busy" data-testid="content-filter"><option value="">全部状态</option><option v-for="(label, key) in states" :key="key" :value="key">{{ label }}</option></select></label><button :disabled="loading || busy">查询</button></form>
    <p v-if="error" class="error-banner" role="alert">{{ error }} <button :disabled="busy || loading" @click="load(offset)">刷新列表</button></p>
    <p v-if="message" role="status" class="notice">{{ message }}</p>
    <section class="panel"><div class="panel-heading"><h2>广场内容</h2><span class="badge">{{ total }} 条</span></div><p v-if="loading" role="status" class="empty-state">正在加载…</p><ul v-else class="content-list"><li v-for="item in items" :key="item.id"><div><strong>{{ item.title }}</strong><p class="muted">{{ item.kind === 'collection' ? '合集' : '提示词' }} · {{ item.model || '通用模型' }} · {{ item.download_count }} 次匿名下载</p></div><span class="badge">{{ states[item.visibility] }}</span><span v-if="item.recommended" class="badge enabled">推荐</span><button :disabled="busy" data-testid="content-edit" @click="open(item.id)">管理</button></li><li v-if="!items.length" class="empty-state">没有符合条件的内容</li></ul><footer class="content-pagination"><span>{{ total ? offset + 1 : 0 }}–{{ Math.min(offset + items.length, total) }} / {{ total }}</span><button :disabled="loading || busy || !offset" @click="load(offset - limit)">上一页</button><button data-testid="content-next" :disabled="loading || busy || offset + limit >= total" @click="load(offset + limit)">下一页</button></footer></section>
    <section v-if="selected" ref="editor" class="panel content-editor" aria-label="内容管理详情"><div class="panel-heading"><h2>{{ detail?.title || '内容详情' }}</h2><button :disabled="busy" @click="close">关闭</button></div><p v-if="detailLoading" class="empty-state">正在加载详情…</p><p v-if="detailError" class="error-banner" role="alert">{{ detailError }} <button :disabled="busy" @click="open(selected)">重新加载详情</button></p>
      <template v-if="detail && draft">
        <p class="panel-note">这里只编辑广场展示信息。标题、正文和合集成员为作者提交快照，不会改动作者的本地库。</p>
        <form data-testid="content-form" @submit.prevent="save"><fieldset :disabled="busy"><div class="content-fields">
          <label class="full">展示摘要<textarea v-model="draft.excerpt" maxlength="2000" rows="3" data-testid="content-excerpt" /></label>
          <label>广场分类<select v-model="draft.category_id"><option :value="null">未分类</option><option v-if="draft.category_id && !categories.some(item => item.id === draft.category_id)" :value="draft.category_id">{{ draft.category_id }}（已停用，保留原值）</option><option v-for="category in categories" :key="category.id" :value="category.id">{{ category.name }}</option></select></label>
          <label>模型<select v-model="draft.model"><option value="">通用模型</option><option v-if="draft.model && !models.some(item => item.id === draft.model)" :value="draft.model">{{ draft.model }}（原有值）</option><option v-for="model in models" :key="model.id" :value="model.id">{{ model.name }}</option></select></label>
          <label>排序权重<input v-model.number="draft.sort_index" type="number" min="-1000000" max="1000000" step="1"><small>数字越小越靠前，仅影响推荐排序。</small></label>
          <label class="check"><input v-model="draft.recommended" type="checkbox">优先推荐</label>
          <label>可见状态<select v-model="draft.visibility" data-testid="content-visibility"><option v-for="(label, key) in allowedStates" :key="key" :value="key">{{ label }}</option></select><small v-if="detail.visibility === 'trashed'">回收站先恢复为下架，再明确上架。</small></label>
          <label v-if="draft.visibility !== detail.visibility">变更原因<textarea v-model="reason" data-testid="content-reason" maxlength="1000" rows="2" required placeholder="说明上下架或回收的原因" /></label>
        </div><div class="content-save"><small>版本 {{ detail.revision }} · 下架/回收保留收藏关系及计数</small><button class="primary" data-testid="content-save" :disabled="!dirty || (draft.visibility !== detail.visibility && !reason.trim())">{{ busy ? '保存中…' : '保存变更' }}</button></div></fieldset></form>
        <details class="content-snapshot"><summary>查看只读正文快照</summary><template v-if="detail.kind === 'collection'"><article v-for="(member, index) in detail.members" :key="index"><h3>{{ member.title }}</h3><pre>{{ member.content }}</pre></article></template><pre v-else>{{ detail.content || '没有正文快照' }}</pre></details>
        <div class="content-history"><h3>运营记录</h3><ul v-if="detail.history?.length"><li v-for="event in detail.history" :key="event.id"><strong>{{ states[event.details.before?.visibility] }} → {{ states[event.details.after?.visibility] }} · 版本 {{ event.details.revision }}</strong><small>{{ new Date(event.created_at).toLocaleString('zh-CN') }} · {{ event.actor_email }}</small><p>{{ event.details.reason || '更新展示信息' }}</p></li></ul><p v-else class="muted">尚无运营修改记录</p></div>
      </template>
    </section>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { listState } from './listState.js';
import { getAdminContent, listAdminContent, saveAdminContent } from './adminApi.js';
const emit = defineEmits(['busy-change']);
const states = { online: '已上架', offline: '已下架', trashed: '回收站' };
const query = ref(''), visibility = ref(''), items = ref([]), total = ref(0), offset = ref(0), categories = ref([]), loading = ref(false), busy = ref(false), error = ref(''), message = ref('');
const selected = ref(''), detail = ref(null), draft = ref(null), reason = ref(''), detailLoading = ref(false), detailError = ref(''), editor = ref(null);
const models = ref([]);
const limit = 25;
let listVersion = 0, detailVersion = 0, applied = { q: '', visibility: '' };
const memory = listState('content'), previous = memory.read();
if (previous.filters) { applied = previous.filters; query.value = applied.q; visibility.value = applied.visibility; }
offset.value = previous.offset || 0;
const saved = ref('');
const dirty = computed(() => Boolean(draft.value && (JSON.stringify(draft.value) !== saved.value || reason.value)));
const allowedStates = computed(() => detail.value?.visibility === 'trashed' ? { trashed: '回收站', offline: '恢复为下架' } : states);
defineExpose({ hasUnsavedChanges: dirty, isBusy: busy });
function canLeave() { return !busy.value && (!dirty.value || window.confirm('有未保存的内容修改，确定放弃吗？')); }
async function load(start = offset.value, apply = false) {
  if (busy.value) return;
  const current = ++listVersion, filters = apply ? { q: query.value, visibility: visibility.value } : applied;
  loading.value = true; error.value = '';
  try { const result = await listAdminContent({ ...filters, offset: start, limit }); if (current !== listVersion) return; items.value = result.items ?? []; total.value = result.total ?? items.value.length; categories.value = result.categories ?? []; models.value = result.models ?? []; offset.value = start; applied = filters; memory.save({ filters, offset: start }); }
  catch (caught) { if (current === listVersion) error.value = caught.message; }
  finally { if (current === listVersion) loading.value = false; }
}
async function open(id) {
  if (!canLeave()) return;
  const current = ++detailVersion; selected.value = id; detail.value = null; draft.value = null; reason.value = ''; detailError.value = ''; detailLoading.value = true;
  await nextTick(); editor.value?.scrollIntoView?.({ block: 'start' });
  try {
    const row = await getAdminContent(id); if (current !== detailVersion) return;
    if (!Number.isSafeInteger(row.revision) || !states[row.visibility]) throw new Error('内容响应无效，不能编辑');
    detail.value = row; draft.value = { excerpt: row.excerpt ?? '', category_id: row.category_id ?? null, model: row.model ?? '', sort_index: row.sort_index, recommended: row.recommended, visibility: row.visibility }; saved.value = JSON.stringify(draft.value);
    await nextTick(); editor.value?.querySelector('textarea')?.focus({ preventScroll: true });
  } catch (caught) { if (current === detailVersion) detailError.value = caught.message; }
  finally { if (current === detailVersion) detailLoading.value = false; }
}
function close() { if (!canLeave()) return; ++detailVersion; selected.value = ''; detail.value = null; draft.value = null; reason.value = ''; }
async function save() {
  if (busy.value || !dirty.value || !detail.value) return;
  const id = selected.value, version = detailVersion;
  if (!Number.isInteger(draft.value.sort_index) || Math.abs(draft.value.sort_index)>1000000) { detailError.value = '排序权重须为 -1000000 到 1000000 的整数'; return; }
  if (draft.value.visibility !== detail.value.visibility) {
    if (!reason.value.trim()) return;
    if (!window.confirm(`确认将「${detail.value.title}」设为${states[draft.value.visibility]}？`)) return;
  }
  busy.value = true; emit('busy-change', true); detailError.value = ''; message.value = '';
  try {
    const result = await saveAdminContent(selected.value, { ...draft.value, model: draft.value.model.trim() || null, revision: detail.value.revision, reason: reason.value.trim() || null });
    if (result.updated !== true || !Number.isSafeInteger(result.revision)) throw new Error('服务端未确认保存，请重新加载详情检查');
    detail.value = { ...detail.value, ...draft.value, revision: result.revision }; saved.value = JSON.stringify(draft.value); reason.value = ''; message.value = '变更已保存；公开广场立即生效。';
  } catch (caught) { detailError.value = caught.message; }
  finally { busy.value = false; emit('busy-change', false); }
  if (!detailError.value) { await load(offset.value); if (selected.value === id && detailVersion === version && !dirty.value) await open(id); }
}
onMounted(() => load(offset.value));
onUnmounted(() => { ++listVersion; ++detailVersion; });
</script>

<style scoped>
.content-workspace { display: grid; gap: 20px; }
.content-filters { display: flex; flex-wrap: wrap; align-items: end; gap: 12px; }
.content-filters label:first-child { flex: 1; min-width: 180px; }
select, textarea { font: inherit; color: #242b30; border: 1px solid #dce1e4; border-radius: 7px; padding: 10px 12px; background: #fff; width: 100%; margin-top: 8px; }
textarea { resize: vertical; }
.content-list, .content-history ul { list-style: none; padding: 0; margin: 0; }
.content-list li { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; padding: 20px 24px; border-top: 1px solid #edf0f2; }
.content-list li > div { flex: 1; min-width: 160px; overflow-wrap: anywhere; }
.content-pagination { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; padding: 16px 24px; border-top: 1px solid #edf0f2; }
.content-pagination span { margin-right: auto; color: #7c858d; }
.content-editor { scroll-margin-top: 16px; }
.content-editor .panel-note, .content-editor form, .content-snapshot, .content-history { padding: 20px 24px; }
fieldset { border: 0; padding: 0; margin: 0; }
.content-fields { display: grid; grid-template-columns: repeat(2,minmax(0,1fr)); gap: 18px; }
.content-fields .full { grid-column: 1 / -1; }
small { color: #7c858d; display: block; font-weight: 400; margin-top: 4px; }
.check { display: flex; align-items: center; gap: 8px; }
.check input { margin: 0; }
.content-save { display: flex; align-items: center; flex-wrap: wrap; gap: 14px; margin-top: 24px; }
.content-save small { margin-right: auto; }
.content-snapshot { border-top: 1px solid #edf0f2; }
.content-snapshot summary { cursor: pointer; }
pre { white-space: pre-wrap; overflow-wrap: anywhere; background: #f7f8fa; padding: 16px; border-radius: 8px; max-height: 300px; overflow: auto; }
.content-history li { display: grid; gap: 6px; border-bottom: 1px solid #edf0f2; padding: 14px 0; overflow-wrap: anywhere; }
@media(max-width: 640px) { .content-fields { grid-template-columns: minmax(0,1fr); } .content-list li, .content-editor form, .content-editor .panel-note, .content-snapshot, .content-history { padding: 16px; } }
</style>
