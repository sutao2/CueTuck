<template>
  <main
    class="launcher-canvas"
    :style="{ '--launcher-content-size': `${launcherPreferences.fontSize}px` }"
    :class="{ 'host-mac': host === 'macos' }"
    aria-label="快捷搜索"
    @keydown="onCanvasKey"
  >
    <section
      class="launcher-stage"
      :class="{
        'host-mac': host === 'macos',
        'is-collapsed': isCollapsed,
        'is-fill': step === 'fill',
      }"
      data-testid="launcher-chrome"
      @mousedown="startDragFromChrome"
    >
      <template v-if="step === 'search'">
        <div class="launcher-search-wrap">
          <img class="brand-mark" :src="appIcon" alt="" aria-hidden="true" draggable="false" />
          <input
            ref="inputEl"
            v-model="query"
            class="launcher-search"
            type="text"
            placeholder="搜索提示词，或输入一个任务…"
            role="combobox"
            aria-autocomplete="list"
            :aria-expanded="!isCollapsed"
            aria-controls="launcher-results"
            autocomplete="off"
            :disabled="busy"
            :aria-activedescendant="searchRows.length ? `launcher-result-${selectedIndex}` : undefined"
            @keydown="onSearchKey"
          />
          <span class="launcher-window-tools">
            <button v-if="scope === 'square'" class="pill" type="button" @click="scope = 'local'">返回本地</button><span v-else class="pill">本地</span>
            <button class="launcher-close-btn" type="button" title="关闭 (Esc)" :disabled="busy" @click="resetAndHide">
              Esc
            </button>
          </span>
        </div>

        <div v-if="!isCollapsed" id="launcher-results" class="launcher-list" role="listbox" aria-label="搜索与快捷操作">
          <p v-if="feedback" role="status" data-testid="launcher-feedback" class="launcher-empty">{{ feedback }}</p>
          <p v-if="searching" role="status" class="launcher-empty">正在搜索…</p>
          <div v-if="results.length">
            <p class="group-title">{{ scope === 'square' ? '广场搜索 · 显示前 20 条' : '本地提示词' }}</p>
            <button
              v-for="(row, index) in results"
              :key="row.id"
              :id="`launcher-result-${index}`"
              class="result-row"
              :class="{ active: index === selectedIndex }"
              type="button"
              role="option"
              :aria-selected="index === selectedIndex"
              tabindex="-1"
              :disabled="busy"
              @mousedown.prevent
              @click="activate(row, 'default')"
              @mousemove="selectWithPointer($event, index)"
            >
              <span class="result-icon">{{ rowIcon(row) }}</span>
              <span class="result-copy">
                <span class="row-title">{{ row.title }}</span>
                <span class="row-desc">{{ rowDesc(row) }}</span>
              </span>
              <span class="pill">{{ scope === 'square' ? (row.kind === 'collection' ? '选择提示词' : '使用') : rowIcon(row) === 'VAR' ? '变量' : '提示词' }}</span>
            </button>
          </div>
          <p v-else-if="!searching && !feedback" class="launcher-empty">没有找到相关提示词</p>
          <div v-if="query.trim()" class="quick-actions" aria-label="输入快捷操作">
            <p class="group-title">使用当前输入</p>
            <button v-for="(action, index) in quickActions" :key="action.action" :aria-selected="selectedIndex === results.length + index" :id="`launcher-result-${results.length + index}`" type="button" role="option" tabindex="-1" class="result-row" :class="{active: selectedIndex === results.length + index}" :disabled="busy" @mousedown.prevent @click="runQuickAction(action.action)" @mousemove="selectWithPointer($event, results.length + index)">
              <span class="result-icon">{{ action.icon }}</span><span class="row-title">{{ action.title }}</span>
            </button>
          </div>
        </div>

        <footer v-if="!isCollapsed" class="launcher-foot">
          <div class="launcher-keys">
            <span><kbd>↑↓</kbd> 选择</span>
            <span><kbd>Enter</kbd> 使用</span>
            <span><kbd>{{ copyChord }}</kbd> 复制</span>
            <span><kbd>Esc</kbd> 关闭</span>
          </div>
          <button v-if="scope === 'square' && searchRows[selectedIndex]?.remote" type="button" class="ghost" :disabled="busy" @click="viewSquareDetail(searchRows[selectedIndex])">查看详情</button>
        </footer>
      </template>

      <template v-else-if="step === 'draft'">
        <header class="launcher-search-wrap"><img class="brand-mark" :src="appIcon" alt=""><div class="result-copy"><span class="row-title">{{ optimizedDraft ? '检查 AI 优化结果' : '快捷创建提示词' }}</span><span class="row-desc">检查内容后使用或保存</span></div></header>
        <div class="launcher-list quick-draft">
          <label class="field"><span>标题</span><input ref="draftTitleEl" v-model="draftTitle" maxlength="160" :disabled="busy" data-testid="quick-title"></label>
          <label class="field quick-content"><span>正文 · 可继续修改</span><textarea v-model="draftContent" :disabled="busy" data-testid="quick-content" /></label>
          <details v-if="optimizedDraft"><summary>查看原始输入</summary><pre class="preview code">{{ query }}</pre></details>
          <p v-if="feedback" role="status">{{ feedback }}</p>
        </div>
        <footer class="launcher-foot launcher-actions"><button type="button" class="ghost" :disabled="busy" @click="backToSearch">返回输入</button><button type="button" class="ghost" :disabled="busy || !draftContent.trim()" @click="activate({title:draftTitle,content:draftContent}, 'default')">{{ optimizedDraft ? '采用并使用' : '直接使用' }}</button><button type="button" class="primary" :disabled="busy || !draftTitle.trim() || !draftContent.trim()" @click="saveDraft">保存到本地</button></footer>
      </template>
      <template v-else>
        <div class="launcher-search-wrap launcher-fill-head">
          <img class="brand-mark" :src="appIcon" alt="" aria-hidden="true" draggable="false" />
          <div class="result-copy">
            <span class="row-title">{{ active?.title }}</span>
            <span class="row-desc">{{ variableNames.length ? '填写内容，预览后复制' : '确认正文后复制' }}</span>
          </div>
          <span class="pill">{{ variableNames.length ? `${variableNames.length} 个变量` : '预览' }}</span>
        </div>
        <div class="launcher-list launcher-fill-body">
          <div class="form-layout" :class="{ 'preview-only': !variableNames.length }">
            <form v-if="variableNames.length" ref="formEl" class="stack variable-form" @submit.prevent="copyRendered">
              <div class="form-heading"><h3>填写变量</h3><span>留空使用默认值，无默认值则保留占位符</span></div>
              <label v-for="(name, index) in variableNames" :key="name" class="field">
                <span>{{ name }}</span>
                <textarea :value="values.get(name) ?? ''" rows="1" :placeholder="`填写 ${name}`" :disabled="busy" @input="values.set(name, $event.target.value)" @focus="focusedVariable = name" @keydown="onVariableKey($event, index)"></textarea>
              </label>
              <button
                v-if="canReadSelected"
                type="button"
                data-testid="read-selected"
                class="ghost selection-action"
                :title="`填入 ${focusedVariable}`"
                :disabled="busy"
                @click="readSelected"
              >
                填入原窗口选中文字
              </button>
            </form>
            <section class="stack preview-pane">
              <h3>实时预览</h3>
              <pre class="preview code">{{ preview }}</pre>
              <p v-if="feedback" role="status" data-testid="launcher-feedback">{{ feedback }}</p>
            </section>
          </div>
        </div>
        <footer class="launcher-foot launcher-fill-foot">
          <div class="fill-keys" :title="`${copyChord} 随时复制 · Esc 关闭`"><span><kbd>↵</kbd> {{ variableNames.length ? '下一项 / 末项复制' : '复制' }}</span><span v-if="variableNames.length"><kbd>⇧↵</kbd> 换行</span></div>
          <div class="launcher-actions">
            <button type="button" class="ghost" :disabled="busy" @click="backToSearch">返回</button>
            <button v-if="active?.remote" type="button" class="ghost" :disabled="busy" @click="viewSquareDetail(active)">查看详情</button>
            <button ref="copyButton" type="button" class="primary" :disabled="busy" @click="copyRendered">复制</button>
            </div>
        </footer>
      </template>
    </section>
  </main>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { squarePromptForUse } from './lib/squarePromptUse.js';
import { fetchSquareContent, listSquarePage } from './platform/square.js';
import { getLauncherAiConfig, optimizeLauncherPrompt } from './platform/launcherAi.js';
import { invokeCommand } from './platform/tauri.js';
import appIcon from "./assets/app-icon.png";
import { extractVariables, renderPrompt, variableDefaults } from "./lib/renderPrompt.js";
import { handleLauncherSearchKey } from "./platform/launcherKeyboard.js";
import {
  resizeLauncherWindow,
  startDraggingLauncher,
  launcherCommand,
  listenLauncherLifecycle,
} from "./platform/launcherWindow.js";
import { createLocalPrompt, getLocalSetting, listLocalPrompts, recordLocalPromptUse, setLocalSetting } from "./platform/library.js";
import { copyLauncherText } from "./platform/paste.js";
import { supportsSelectedText } from "./platform/selectedText.js";
import { applyHostChrome, detectHost, formatShortcutLabel } from "./platform/windowChrome.js";
import { DEFAULT_LAUNCHER_PREFERENCES, readLauncherPreferences } from './platform/launcherPreferences.js';

const props = defineProps({
  host: { type: String, default: () => detectHost() },
});

const launcherPreferences = ref({ ...DEFAULT_LAUNCHER_PREFERENCES });
const query = ref("");
const inputEl = ref(null);
const formEl = ref(null);
const copyButton = ref(null);
const focusedVariable = ref("");
const searching = ref(false);
const results = ref([]);
const selectedIndex = ref(0);
const step = ref("search");
const active = ref(null);
const values = reactive(new Map());
const feedback = ref("");
const useBusy = ref(false);
const selectionBusy = ref(false);
const actionBusy = ref(false);
const busy = computed(() => useBusy.value || selectionBusy.value || actionBusy.value);
const scope = ref('local'), draftTitle = ref(''), draftContent = ref(''), optimizedDraft = ref(false), draftTitleEl = ref(null);
const quickActions = computed(() => query.value.trim() ? [
  {action:'create',title:'创建提示词',icon:'＋'},
  {action:'optimize',title:'AI 优化提示词',icon:'AI'},
  {action:'square',title:'搜索提示词广场',icon:'⌕'},
] : []);
const searchRows = computed(() => [...results.value, ...quickActions.value]);
let searchTimer, squareController;
let searchRequest = 0;
let disposed = false;
let unlisten = () => {};
let blurTimer;
let hidden = !!window.__TAURI_INTERNALS__;
let lifecycleVersion = 0;
const canReadSelected = !!window.__TAURI_INTERNALS__ && supportsSelectedText();

const variableNames = computed(() => extractVariables(active.value?.content ?? ""));
const preview = computed(() => renderPrompt(active.value?.content ?? "", Object.fromEntries(values)));
const isCollapsed = computed(() => step.value === "search" && !query.value.trim() && !feedback.value);
const launcherLayout = computed(() =>
  step.value === "fill" ? "fill" : isCollapsed.value ? "collapsed" : "expanded",
);
const copyChord = computed(() => formatShortcutLabel("Control+Enter", props.host));

async function searchCurrent(request) {
  const needle = query.value.trim();
  if (!needle) { searching.value = false; return; }
  try {
    let rows;
    if (scope.value === 'square') {
      if (await getLocalSetting('square_access') === '0') throw new Error('广场访问已关闭，请在设置中开启');
      if (request !== searchRequest) return;
      squareController = new AbortController();
      const page = await listSquarePage({query:needle,signal:squareController.signal});
      rows = page.items.slice(0,20).map(row=>({...row,remote:true}));
    } else rows = await listLocalPrompts({ query: needle });
    if (request === searchRequest) results.value = rows.slice(0, launcherPreferences.value.resultLimit);
  } catch (error) {
    if (request === searchRequest) { results.value = []; feedback.value = `搜索失败：${error.message || error}`; }
  } finally { if (request === searchRequest) searching.value = false; }
}
watch([query,scope], () => {
  const request = ++searchRequest;
  clearTimeout(searchTimer);squareController?.abort();
  feedback.value = '';selectedIndex.value = 0;results.value = [];searching.value = !!query.value.trim();
  if (scope.value === 'square') searchTimer = setTimeout(()=>searchCurrent(request),250);
  else searchCurrent(request);
});
async function openDestination(destination, id) {
  if (!window.__TAURI_INTERNALS__) throw new Error('请在桌面客户端打开此页面');
  await invokeCommand('open_launcher_destination',{destination,id:id||null});
}
async function runQuickAction(action) {
  if (busy.value || !query.value.trim()) return;
  if (action === 'create') { draftTitle.value=query.value.trim().split('\n')[0].slice(0,80);draftContent.value=query.value;optimizedDraft.value=false;step.value='draft';await nextTick();draftTitleEl.value?.focus();return; }
  if (action === 'square') { if(scope.value==='square') {clearTimeout(searchTimer);squareController?.abort();searching.value=true;searchCurrent(++searchRequest);} else scope.value='square';return; }
  actionBusy.value=true;
  try {
    const config=await getLauncherAiConfig();
    if (!config.endpoint || !config.model) { await openDestination('ai-settings'); feedback.value='请先在「AI 与模型」设置自己的接口和模型。';return; }
    feedback.value='正在优化，仅发送当前输入…';
    const version=lifecycleVersion, source=query.value;
    const result=await optimizeLauncherPrompt(source);
    if(disposed || version!==lifecycleVersion || source!==query.value)return;
    draftTitle.value=source.trim().split('\n')[0].slice(0,80);draftContent.value=result;optimizedDraft.value=true;step.value='draft';feedback.value='原文已保留；检查结果后再采用或保存。';
  } catch(error) { feedback.value=String(error.message||error); }
  finally { actionBusy.value=false; }
}
async function saveDraft() {
  if(busy.value || !draftTitle.value.trim() || !draftContent.value.trim())return;
  actionBusy.value=true;
  try {
    const row=await createLocalPrompt({title:draftTitle.value.trim(),content:draftContent.value,model:(await getLocalSetting('default_model'))||null});
    actionBusy.value=false;await activate(row,'default');feedback.value='已创建并保存到本地';
    if(window.__TAURI_INTERNALS__) { try {const {emit}=await import('@tauri-apps/api/event'); await emit('local-library-changed');}catch {feedback.value='已保存到本地；主窗口请刷新查看。';} }
  } catch(error){feedback.value=`保存失败：${error.message||error}`;}
  finally{actionBusy.value=false;}
}

watch(launcherLayout, (layout) => resizeLauncherWindow(layout).catch((error) => { feedback.value = `窗口调整失败：${error}`; }), { immediate: true });
watch(selectedIndex, async () => {
  await nextTick();
  document.getElementById(`launcher-result-${selectedIndex.value}`)?.scrollIntoView?.({ block: "nearest" });
});

function rowIcon(row) {
  return extractVariables(row.content ?? "").length ? "VAR" : "TXT";
}

function rowDesc(row) {
  return String(row.summary || row.content || "")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, 72);
}

function onSearchKey(event) {
  if (busy.value) return;
  handleLauncherSearchKey(event, {
    move: (delta) => {
      const count = searchRows.value.length;
      if (!count) return;
      selectedIndex.value = ((selectedIndex.value + delta) % count + count) % count;
    },
    moveTo: (index) => {
      selectedIndex.value = Math.max(0, Math.min(index, searchRows.value.length - 1));
    },
    rowCount: searchRows.value.length,
    current: () => searchRows.value[selectedIndex.value] ?? null,
    activate,
    close: resetAndHide,
  });
}

function selectWithPointer(event, index) {
  if (event.movementX || event.movementY) selectedIndex.value = index;
}

async function viewSquareDetail(row) {
  try { await openDestination('square-detail', row.id); }
  catch (error) { feedback.value = String(error.message || error); }
}

async function activate(row, mode) {
  if (!row || busy.value) return;
  if (row.action) { await runQuickAction(row.action); return; }
  if (row.remote) {
    if (row.kind === 'collection') { await viewSquareDetail(row); return; }
    const version = lifecycleVersion, request = searchRequest;
    actionBusy.value = true;
    feedback.value = '正在读取完整提示词…';
    try {
      const content = await fetchSquareContent(row.id);
      if (disposed || version !== lifecycleVersion || request !== searchRequest) return;
      if (content.kind === 'collection') { await viewSquareDetail(row); return; }
      row = squarePromptForUse({ ...row, ...content });
    } catch (error) { feedback.value = `读取失败：${error.message || error}，请重试。`; return; }
    finally { actionBusy.value = false; }
  }
  if (!row.content?.trim()) { feedback.value = '提示词内容为空，未修改剪贴板。'; return; }
  feedback.value = "";
  active.value = row;
  values.clear();
  for (const [name,value] of Object.entries(variableDefaults(row.content))) values.set(name,value);
  step.value = "fill";
  if (mode === "copy" || !variableNames.value.length) {
    await copyRendered();
    return;
  }
  focusedVariable.value = variableNames.value[0] || "";
  await focusCurrent();
}

async function backToSearch() {
  if (busy.value) return;
  step.value = "search";
  active.value = null;
  feedback.value = "";
  await focusCurrent();
}

async function focusCurrent() {
  await nextTick();
  if (disposed || hidden) return;
  if (step.value === "search") inputEl.value?.focus();
  else if (step.value === 'draft') draftTitleEl.value?.focus();
  else if (variableNames.value.length) {
    const index = Math.max(0, variableNames.value.indexOf(focusedVariable.value));
    formEl.value?.querySelectorAll("textarea")[index]?.focus();
  } else copyButton.value?.focus();
}

function onCanvasKey(event) {
  if (event.isComposing || event.keyCode === 229) return;
  if (event.key === "Escape" || (step.value === "fill" && event.key === "Enter" && (event.metaKey || event.ctrlKey))) {
    event.preventDefault();
    event.stopPropagation();
    if (event.repeat || busy.value) return;
    if (event.key === "Escape") resetAndHide();
    else copyRendered();
  } else if (event.key === "Enter" && event.repeat) event.preventDefault();
}

function onVariableKey(event, index) {
  if (event.isComposing || event.keyCode === 229 || event.key !== "Enter") return;
  if (event.shiftKey && !event.metaKey && !event.ctrlKey) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.repeat || busy.value) return;
  if (event.metaKey || event.ctrlKey || index === variableNames.value.length - 1) copyRendered();
  else formEl.value?.querySelectorAll("textarea")[index + 1]?.focus();
}

function startDragFromChrome(event) {
  if (event.button !== 0) return;
  const interactive = event.target.closest(
    "input, textarea, select, button, a, .launcher-list, .launcher-foot, .result-row",
  );
  if (!interactive) startDraggingLauncher();
}

async function finishUse(text, id) {
  feedback.value = "已复制";
  let closeAfter = false;
  try {
    if (id) await recordLocalPromptUse(id);
    await setLocalSetting("last_rendered_prompt", text);
    closeAfter = await getLocalSetting("close_launcher_after_use") !== "0";
  } catch (error) { feedback.value += `；保存使用记录失败：${error.message || error}`; }
  if (closeAfter) {
    await launcherCommand("hide_launcher");
    hidden = true;
    resetState();
  }
  return text;
}

async function copyRendered() {
  if (busy.value || !active.value) return;
  const text = preview.value;
  const id = active.value.remote ? null : active.value.id;
  if (!text.trim()) { feedback.value = "提示词内容为空，未修改剪贴板。"; return; }
  useBusy.value = true;
  feedback.value = "正在复制…";
  let copied = false;
  try {
    await copyLauncherText(text);
    copied = true;
    await finishUse(text, id);
  } catch (error) {
    feedback.value = copied ? `已复制；窗口关闭失败：${error?.message || error}` : `复制失败，请检查剪贴板权限后重试；内容已保留。${error?.message || error}`;
  } finally {
    useBusy.value = false;
    await focusCurrent();
  }
}

async function readSelected() {
  if (!canReadSelected || busy.value) return;
  const name = focusedVariable.value;
  if (!name) return;
  selectionBusy.value = true;
  try {
    const selected = await launcherCommand("capture_selected_text");
    if (selected?.trim()) { values.set(name, selected); feedback.value = `已填入 ${name}`; }
    else feedback.value = "原窗口没有可读取的选区；已保留原内容。";
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    selectionBusy.value = false;
    await focusCurrent();
  }
}

async function onBlur() {
  if (!window.__TAURI_INTERNALS__ || busy.value || disposed || hidden) return;
  const version = lifecycleVersion;
  try {
    const didHide = await launcherCommand("hide_launcher_if_idle");
    if (disposed || version !== lifecycleVersion) return;
    if (didHide) { hidden = true; if (step.value === 'search') resetState(); }
    else {
      clearTimeout(blurTimer);
      blurTimer = setTimeout(() => { if (!document.hasFocus()) onBlur(); }, 650);
    }
  } catch (error) { feedback.value = `隐藏窗口失败：${error}`; }
}

function onFocus() {
  clearTimeout(blurTimer);
  if (!busy.value) focusCurrent();
}

function resetState() {
  clearTimeout(blurTimer);
  searchRequest += 1; clearTimeout(searchTimer); squareController?.abort(); scope.value='local'; draftContent.value='';draftTitle.value='';
  searching.value = false;
  feedback.value = "";
  query.value = "";
  results.value = [];
  selectedIndex.value = 0;
  step.value = "search";
  active.value = null;
  values.clear();
}

async function resetAndHide() {
  if (busy.value) return;
  try {
    await launcherCommand("hide_launcher");
    hidden = true;
    resetState();
  } catch (error) { feedback.value = `隐藏窗口失败：${error}`; }
}

let themePreference = "light";
let systemTheme;
function applyLauncherTheme() {
  if (!disposed) document.body.classList.toggle("theme-dark", themePreference === "dark" ||
    (themePreference === "system" && Boolean(systemTheme?.matches)));
}
async function refreshTheme() {
  themePreference = await getLocalSetting("theme");
  applyLauncherTheme();
}

async function onShown() {
  if (busy.value || disposed) return;
  lifecycleVersion++;
  clearTimeout(blurTimer);
  hidden = false;
  if (step.value === 'search') resetState();
  try { launcherPreferences.value = await readLauncherPreferences(); }
  catch (error) { feedback.value = `读取启动器设置失败：${error}`; }
  try { await resizeLauncherWindow(launcherLayout.value); }
  catch (error) { feedback.value = `窗口调整失败：${error}`; }
  await focusCurrent();
  try { await refreshTheme(); } catch (error) { feedback.value = `读取主题失败：${error}`; }
}

onMounted(async () => {
  document.body.classList.add("launcher-page");
  applyHostChrome(document.body, props.host);
  systemTheme = window.matchMedia?.("(prefers-color-scheme: dark)");
  systemTheme?.addEventListener("change", applyLauncherTheme);
  window.addEventListener("blur", onBlur);
  window.addEventListener("focus", onFocus);
  focusCurrent();
  try {
    unlisten = await listenLauncherLifecycle(onShown, (event) => {
      hidden = true;
      if (!busy.value && (event?.payload !== 'blur' || step.value === 'search')) resetState();
    }, (event) => {
      hidden = false;
      feedback.value = event.payload;
      focusCurrent();
    });
    if (disposed) { unlisten(); return; }
    launcherPreferences.value = await readLauncherPreferences();
    await refreshTheme();
  } catch (error) { feedback.value = `启动器初始化失败：${error}`; }
});

onUnmounted(() => {
  disposed = true; clearTimeout(searchTimer); squareController?.abort();
  searchRequest += 1;
  clearTimeout(blurTimer);
  unlisten();
  systemTheme?.removeEventListener("change", applyLauncherTheme);
  window.removeEventListener("blur", onBlur);
  window.removeEventListener("focus", onFocus);
  document.body.classList.remove("launcher-page");
});
</script>

<style scoped>
.launcher-canvas {
  width: 100%;
  height: 100%;
  display: grid;
  place-items: stretch;
  background: transparent;
}
.launcher-stage {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  width: 100%;
  height: 100%;
  min-height: 0;
  border-radius: 14px;
  overflow: hidden;
  background: var(--surface);
  box-shadow:
    inset 0 0 0 1px color-mix(in srgb, var(--text) 10%, transparent),
    inset 0 1px 0 color-mix(in srgb, var(--surface) 70%, transparent);
}
.launcher-stage.is-collapsed {
  grid-template-rows: 1fr;
}
.launcher-stage.is-fill {
  display: block;
  position: relative;
}
.launcher-search-wrap {
  min-height: 60px;
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  padding: 0 14px;
  border-bottom: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
}
.launcher-stage.is-collapsed .launcher-search-wrap {
  min-height: 64px;
  border-bottom: 0;
}
.brand-mark {
  width: 28px;
  height: 28px;
  display: block;
  object-fit: contain;
  user-select: none;
}
.launcher-search {
  width: 100%;
  height: 42px;
  border: 0;
  background: transparent;
  appearance: none;
  -webkit-appearance: none;
  font-size: 16px;
  font-weight: 500;
  outline: none;
}
.launcher-search::-webkit-search-decoration,
.launcher-search::-webkit-search-cancel-button {
  appearance: none;
}
.launcher-window-tools {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.pill {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 0 8px;
  border: 1px solid transparent;
  border-radius: 5px;
  background: var(--bg);
  color: var(--muted);
  font-size: 11px;
}
.launcher-close-btn {
  min-width: 34px;
  height: 24px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: transparent;
  color: var(--muted);
  font-size: 11px;
}
.launcher-list {
  min-height: 0;
  overflow: auto;
  scrollbar-gutter: stable;
  padding: 8px;
}
.group-title {
  margin: 4px 8px 8px;
  color: var(--muted);
  font-size: 11px;
}
.result-row {
  width: 100%;
  min-height: 52px;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  margin: 0 0 4px;
  padding: 8px 12px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  text-align: left;
}
.result-row:hover { background: var(--hover); }
.result-row.active { background: var(--accent-soft); }
.result-icon {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border-radius: 8px;
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-size: 10px;
  font-weight: 700;
}
.result-row.active .result-icon {
  background: var(--accent);
  color: var(--surface);
}
.result-copy {
  display: grid;
  min-width: 0;
  gap: 2px;
}
.row-title {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.row-desc {
  overflow: hidden;
  color: var(--muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.launcher-empty {
  margin: 24px;
  color: var(--muted);
  text-align: center;
}
.launcher-foot {
  display: flex;
  align-items: center;
  min-height: 42px;
  padding: 0 12px;
  border-top: 1px solid var(--line);
  color: var(--muted);
  background: var(--bg);
  font-size: 12px;
}
.launcher-keys,
.launcher-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}
.launcher-actions {
  width: 100%;
  justify-content: flex-end;
}
.launcher-keys span {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.launcher-fill-head {
  position: absolute;
  inset: 0 0 auto;
  height: 54px;
  min-height: 54px;
}
.launcher-fill-body {
  position: absolute;
  inset: 54px 0 48px;
  padding: 14px;
  overflow: hidden;
}
.launcher-fill-foot {
  position: absolute;
  inset: auto 0 0;
  height: 48px;
  justify-content: space-between;
  gap: 12px;
}
.launcher-fill-foot .launcher-actions { width: auto; flex-wrap: nowrap; gap: 6px; }
.fill-keys { display: flex; flex-wrap: wrap; gap: 4px 10px; font-size: 11px; }
.fill-keys span { white-space: nowrap; }
.form-layout {
  display: grid;
  grid-template-columns: minmax(0, .9fr) minmax(0, 1.1fr);
  gap: 14px;
  height: 100%;
}
.form-layout.preview-only { grid-template-columns: minmax(0, 1fr); }
.variable-form { overflow: auto; scrollbar-gutter: stable; padding: 2px 12px 8px 2px; }
.form-heading { display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 4px; }
.form-heading > span { font-size: 11px; color: var(--muted); }
.preview-pane { min-height: 0; grid-template-rows: auto minmax(0, 1fr) auto; border-left: 1px solid var(--line); padding-left: 14px; }
.preview-only .preview-pane { border-left: 0; padding-left: 0; }
.selection-action { justify-self: start; font-size: 11px; padding-left: 0; }
.stack {
  display: grid;
  align-content: start;
  gap: 12px;
  min-width: 0;
}
.field {
  display: grid;
  gap: 6px;
  color: var(--text);
  font-size: 12px;
}
.field > span { overflow-wrap: anywhere; }
.field textarea {
  font-size: var(--launcher-content-size, 12px);
  width: 100%;
  height: 44px;
  min-height: 44px;
  resize: none;
  line-height: 24px;
  color: var(--text);
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  padding: 9px 10px;
  background: var(--surface);
}
.field textarea::placeholder { color: var(--faint); }
.field textarea:hover { border-color: var(--muted); }
.preview,
.code {
  margin: 0;
  min-height: 0;
  overflow: auto;
  scrollbar-gutter: stable;
  padding: 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--bg);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font: var(--launcher-content-size, 12px)/1.8 var(--font-code);
}
h3 {
  margin: 0;
  font-size: 13px;
}
.primary,
.ghost {
  border: 0;
  min-height: 32px;
  border-radius: 7px;
  padding: 6px 10px;
  cursor: pointer;
  transition: background-color 120ms, color 120ms;
}
button:focus-visible, textarea:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
.field textarea:focus-visible { outline: none; border-color: var(--focus); box-shadow: 0 0 0 3px color-mix(in srgb, var(--focus) 14%, transparent); }
button:disabled { opacity: .5; cursor: wait; }
.primary {
  background: var(--accent);
  color: var(--surface);
}
.primary:hover:not(:disabled) { background: var(--accent-strong); }
.ghost:hover:not(:disabled), .launcher-close-btn:hover:not(:disabled) { background: var(--hover); color: var(--text); }
.ghost {
  background: transparent;
  color: var(--muted);
}
@media (prefers-reduced-motion: reduce) {
  .primary, .ghost { transition: none; }
}
</style>

<style scoped>
.quick-actions { border-top:1px solid var(--line,#ddd);margin-top:8px;padding-top:4px; }.quick-actions .result-row{min-height:38px;padding-top:8px;padding-bottom:8px}.quick-draft{display:flex;flex-direction:column;gap:12px;padding:18px}.quick-content{flex:1;min-height:150px;display:flex;flex-direction:column;gap:6px}.quick-content textarea{flex:1;resize:vertical;min-height:130px}.quick-draft input,.quick-draft textarea{width:100%;box-sizing:border-box;background:var(--surface,#fff);color:inherit;border:1px solid var(--line,#ccc);border-radius:8px;padding:10px;font:inherit}.quick-draft .field{display:flex;flex-direction:column;gap:6px}.quick-draft p{font-size:12px}.quick-draft .preview{max-height:130px;overflow:auto}
</style>
