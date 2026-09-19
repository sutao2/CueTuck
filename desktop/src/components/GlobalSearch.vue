<template>
  <div class="global-search-layer" @click.self="$emit('close')">
    <section v-dialog-focus="() => $emit('close')" class="global-search-panel" role="dialog" aria-modal="true" aria-labelledby="global-search-title">
      <header class="global-search-header">
        <h2 id="global-search-title">全局搜索</h2>
        <button type="button" class="global-search-close" aria-label="关闭全局搜索" @click="$emit('close')"><kbd>Esc</kbd></button>
      </header>
      <div class="global-search-scopes" role="group" aria-label="搜索范围">
        <button v-for="option in scopes" :key="option.id" type="button" :aria-pressed="scope === option.id" :data-search-scope="option.id" @click="scope = option.id">
          <AppIcon :name="option.id === 'local' ? 'library' : 'globe'" />{{ option.label }}
        </button>
      </div>
      <div class="global-search-field">
        <AppIcon name="search" />
        <input v-model="query" data-testid="global-search-input" data-dialog-autofocus type="search" role="combobox" aria-label="全局搜索关键词" aria-autocomplete="list" aria-controls="global-search-results" :aria-expanded="results.length > 0" :aria-activedescendant="results.length ? `global-result-${active}` : undefined" :placeholder="scope === 'local' ? '搜索全部本地提示词和合集…' : scope.includes('skills') ? '搜索 Skill 名称或描述…' : '搜索广场标题、标签或作者…'" @keydown="onKeydown" @compositionstart="composing = true; schedule()" @compositionend="composing = false; schedule()" />
      </div>
      <p class="global-search-summary" role="status" aria-live="polite">{{ statusText }}</p>
      <div v-if="error" class="global-search-empty" role="alert">
        <p>{{ error }}</p><button type="button" class="button ghost-button" @click="search">重试搜索</button>
      </div>
      <div v-else-if="!results.length" class="global-search-empty">
        <AppIcon :name="scope === 'local' ? 'library' : 'globe'" />
        <p>{{ blocked ? '广场访问已关闭' : !query.trim() ? '从这里查找，不必离开当前页面' : loading ? '正在查找…' : '没有找到匹配内容' }}</p>
        <small>{{ blocked ? '可在设置的网络页面开启广场访问。' : !query.trim() ? '不受当前分类、模型或页面筛选影响' : loading ? '稍等片刻' : '试试更短的关键词，或切换搜索范围' }}</small>
      </div>
      <div id="global-search-results" ref="resultList" class="global-search-results" role="listbox" aria-label="搜索结果" :aria-busy="loading">
        <button v-for="(row, index) in results" :id="`global-result-${index}`" :key="row.item.id" type="button" role="option" tabindex="-1" :aria-selected="active === index" class="global-search-result" @click="choose(row)">
          <span class="global-search-kind"><AppIcon :name="row.item.kind === 'collection' ? 'folder' : 'file'" /></span>
          <span class="global-search-result-copy"><strong><SearchHighlight :text="row.item.title" :query="query" /></strong><small><SearchHighlight :text="row.excerpt || '暂无摘要'" :query="query" /></small></span>
          <span class="global-search-type">{{ row.item.kind === 'skill' ? 'Skill' : row.item.kind === 'collection' ? '合集' : '提示词' }}</span>
        </button>
      </div>
      <footer class="global-search-footer"><span><kbd>↑</kbd><kbd>↓</kbd> 选择 <kbd>Enter</kbd> 打开</span><span>{{ scopes.find(option => option.id === scope)?.label + (scope === 'local' || scope === 'skills-local' ? ' · 无需联网' : ' · 联网搜索') }}</span></footer>
    </section>
  </div>
</template>

<script setup>
import { computed, nextTick, onUnmounted, ref, watch } from 'vue';
import SearchHighlight from './SearchHighlight.vue';
import AppIcon from './AppIcon.vue';
import { skillsSupported, skillsRequest, groupSkills } from '../platform/skills.js';
import { skillMarket } from '../platform/skillMarket.js';
import { vDialogFocus } from '../lib/dialogFocus.js';
import { getLocalSetting, listLocalCollections, listLocalPrompts } from '../platform/library.js';
import { listSquarePage } from '../platform/square.js';
import { applyQueuedFavorites } from '../platform/syncQueue.js';

const emit = defineEmits(['close', 'select']);
const scopes = [{ id: 'local', label: '本地库' }, { id: 'square', label: '提示词广场' }, ...(skillsSupported() ? [{ id: 'skills-local', label: '本机 Skills' }, { id: 'skills-square', label: '社区 Skills' }] : [])];
const scope = ref('local'), query = ref(''), composing = ref(false);
const results = ref([]), active = ref(0), total = ref(0), loading = ref(false), error = ref(''), blocked = ref(false);
const resultList = ref(null);
let timer, controller, request = 0;
const statusText = computed(() => {
  if (!query.value.trim()) return '输入关键词开始搜索';
  if (loading.value) return '正在搜索…';
  if (error.value || blocked.value) return '搜索未完成';
  return total.value > results.value.length ? `找到 ${total.value} 条 · 显示前 ${results.value.length} 条，请细化关键词` : `找到 ${total.value} 条结果`;
});

function invalidate() {
  clearTimeout(timer);
  controller?.abort();
  return ++request;
}
function schedule() {
  invalidate();
  results.value = []; total.value = 0; active.value = 0; error.value = ''; blocked.value = false;
  loading.value = Boolean(query.value.trim());
  if (query.value.trim() && !composing.value) timer = setTimeout(search, 250);
}
async function search() {
  const token = invalidate(), text = query.value.trim(), source = scope.value;
  if (!text || composing.value) return;
  loading.value = true; error.value = ''; blocked.value = false;
  controller = new AbortController();
  const signal = controller.signal;
  try {
    let items, count;
    if (source === 'local') {
      const [prompts, collections] = await Promise.all([listLocalPrompts({ query: text, categoryId: null }), listLocalCollections({ query: text, categoryId: null })]);
      items = [...prompts.map(item => ({ ...item, kind: 'prompt' })), ...collections.map(item => ({ ...item, kind: 'collection' }))]; count = items.length;
    } else if (source === 'skills-local') {
      const snapshot = await skillsRequest({ action: 'snapshot' });
      items = groupSkills(snapshot.skills).filter(item => `${item.name} ${item.description || ''}`.toLowerCase().includes(text.toLowerCase())).map(item => ({ ...item, id: item.key, title: item.name, kind: 'skill' }));
      count = items.length;
    } else {
      const access = await getLocalSetting('square_access');
      if (token !== request) return;
      if (access === '0') { blocked.value = true; return; }
      if (source === 'skills-square') {
        const page = await skillMarket('browse', { query: { q: text, offset: 0 } });
        items = page.items.map(item => ({ ...item, kind: 'skill' })); count = page.total;
      } else {
        const page = await listSquarePage({ query: text, categoryId: null, model: '', sort: '推荐', offset: 0, signal });
        const savedFavorites = page.items.filter(item => item.is_favorite).map(item => item.id);
        const favorites = await applyQueuedFavorites(savedFavorites).catch(() => savedFavorites);
        items = page.items.map(item => ({ ...item, is_favorite: favorites.includes(item.id) })); count = page.total;
      }
    }
    if (token !== request) return;
    results.value = items.slice(0, 48).map(item => ({ item, excerpt: (item.excerpt || item.content || item.description || '').slice(0, 240).replace(/\s+/g, ' ') }));
    total.value = count; active.value = 0;
    if (resultList.value) resultList.value.scrollTop = 0;
  } catch (cause) {
    if (token === request) error.value = `搜索失败：${cause.message || cause}`;
  } finally {
    if (token === request) loading.value = false;
  }
}
function choose(row) {
  emit('select', { item: row.item, scope: scope.value });
}
async function onKeydown(event) {
  if (event.isComposing || event.keyCode === 229 || composing.value || !results.value.length) return;
  if (event.key === 'Enter') { event.preventDefault(); choose(results.value[active.value]); }
  else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    active.value = (active.value + (event.key === 'ArrowDown' ? 1 : -1) + results.value.length) % results.value.length;
    await nextTick();
    resultList.value?.children[active.value]?.scrollIntoView?.({ block: 'nearest' });
  }
}
watch([query, scope], schedule, { flush: 'sync' });
onUnmounted(invalidate);
</script>

<style scoped>
.global-search-layer { position: fixed; inset: 0; z-index: 900; display: flex; align-items: flex-start; justify-content: center; padding: min(12vh, 96px) 24px 24px; background: #0005; }
.global-search-panel { width: 640px; max-width: 100%; max-height: 100%; display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--line-strong); border-radius: 14px; background: var(--surface); color: var(--text); box-shadow: 0 16px 60px #0003; }
.global-search-header { display: flex; align-items: center; justify-content: space-between; padding: 18px 22px 12px; }
.global-search-header h2 { margin: 0; font-size: 16px; font-weight: 650; }
.global-search-close { border: 0; background: transparent; color: var(--muted); cursor: pointer; padding: 4px; }
.global-search-scopes { display: flex; flex-wrap: wrap; gap: 6px; padding: 0 22px 16px; }
.global-search-scopes button { display: flex; align-items: center; gap: 7px; padding: 7px 12px; border: 1px solid transparent; border-radius: 7px; background: transparent; color: var(--muted); font: inherit; font-size: 13px; cursor: pointer; }
.global-search-scopes button[aria-pressed="true"] { background: var(--accent-soft); border-color: var(--line); color: var(--text); }
.global-search-scopes .app-icon { width: 16px; height: 16px; }
.global-search-field { display: flex; align-items: center; gap: 12px; margin: 0 22px; padding: 11px 12px; border: 1px solid var(--line-strong); border-radius: 8px; background: var(--bg); }
.global-search-field:focus-within { outline: 2px solid var(--focus); outline-offset: 2px; }
.global-search-field > .app-icon { width: 19px; height: 19px; color: var(--muted); flex-shrink: 0; }
.global-search-field input { min-width: 0; width: 100%; border: 0; outline: 0; background: transparent; color: var(--text); font: inherit; font-size: 15px; }
.global-search-summary { margin: 16px 22px 8px; font-size: 12px; color: var(--muted); }
.global-search-empty { padding: 30px 22px 42px; text-align: center; color: var(--muted); }
.global-search-empty > .app-icon { width: 30px; height: 30px; opacity: .6; }
.global-search-empty p { font-size: 14px; }
.global-search-empty small { font-size: 12px; }
.global-search-results { overflow-y: auto; min-height: 0; padding: 0 10px 10px; overscroll-behavior: contain; }
.global-search-result { display: flex; align-items: center; gap: 12px; width: 100%; padding: 12px; border: 0; border-radius: 8px; background: transparent; color: var(--text); text-align: left; cursor: pointer; font: inherit; }
.global-search-result:hover { background: var(--hover); }
.global-search-result[aria-selected="true"] { background: var(--accent-soft); }
.global-search-kind { display: grid; place-items: center; flex-shrink: 0; width: 32px; height: 36px; color: var(--muted); }
.global-search-kind .app-icon { width: 22px; height: 22px; }
.global-search-result-copy { min-width: 0; flex: 1; display: grid; gap: 5px; }
.global-search-result-copy strong, .global-search-result-copy small { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.global-search-result-copy strong { font-size: 14px; font-weight: 600; }
.global-search-result-copy small, .global-search-type { color: var(--muted); font-size: 12px; }
.global-search-type { flex-shrink: 0; }
.global-search-footer { display: flex; justify-content: space-between; gap: 12px; border-top: 1px solid var(--line); padding: 12px 22px; color: var(--muted); font-size: 11px; }
.global-search-footer kbd { margin-right: 3px; }
@media (max-width: 520px) { .global-search-layer { padding: 24px 12px; } .global-search-footer { flex-wrap: wrap; } }
</style>
