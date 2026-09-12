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
            :aria-activedescendant="results.length ? `launcher-result-${selectedIndex}` : undefined"
            @keydown="onSearchKey"
          />
          <span class="launcher-window-tools">
            <span class="pill">本地</span>
            <button class="launcher-close-btn" type="button" title="关闭 (Esc)" :disabled="busy" @click="resetAndHide">
              Esc
            </button>
          </span>
        </div>

        <div v-if="!isCollapsed" class="launcher-list">
          <p v-if="feedback" role="status" data-testid="launcher-feedback" class="launcher-empty">{{ feedback }}</p>
          <p v-if="searching" role="status" class="launcher-empty">正在搜索…</p>
          <div v-if="results.length" id="launcher-results" role="listbox" aria-label="搜索结果">
            <p class="group-title">本地提示词</p>
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
              @mouseenter="selectedIndex = index"
            >
              <span class="result-icon">{{ rowIcon(row) }}</span>
              <span class="result-copy">
                <span class="row-title">{{ row.title }}</span>
                <span class="row-desc">{{ rowDesc(row) }}</span>
              </span>
              <span class="pill">{{ rowIcon(row) === "VAR" ? "变量" : "提示词" }}</span>
            </button>
          </div>
          <p v-else-if="!searching && !feedback" class="launcher-empty">没有找到相关提示词</p>
        </div>

        <footer v-if="!isCollapsed" class="launcher-foot">
          <div class="launcher-keys">
            <span><kbd>↑↓</kbd> 选择</span>
            <span><kbd>Enter</kbd> 填写/预览</span>
            <span><kbd>{{ copyChord }}</kbd> 复制</span>
            <span><kbd>Esc</kbd> 关闭</span>
          </div>
        </footer>
      </template>

      <template v-else>
        <div class="launcher-search-wrap launcher-fill-head">
          <img class="brand-mark" :src="appIcon" alt="" aria-hidden="true" draggable="false" />
          <div class="result-copy">
            <span class="row-title">{{ active?.title }}</span>
            <span class="row-desc">{{ variableNames.length ? '填写内容，预览后复制' : '确认正文后复制或粘贴' }}</span>
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
            <button ref="copyButton" type="button" class="primary" :disabled="busy" @click="copyRendered">复制</button>
            <button type="button" class="ghost" :disabled="busy" @click="pasteRendered">粘贴到原窗口</button>
          </div>
        </footer>
      </template>
    </section>
  </main>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import appIcon from "./assets/app-icon.png";
import { extractVariables, renderPrompt, variableDefaults } from "./lib/renderPrompt.js";
import { handleLauncherSearchKey } from "./platform/launcherKeyboard.js";
import {
  resizeLauncherWindow,
  startDraggingLauncher,
  launcherCommand,
  listenLauncherLifecycle,
} from "./platform/launcherWindow.js";
import { getLocalSetting, listLocalPrompts, recordLocalPromptUse, setLocalSetting } from "./platform/library.js";
import { copyLauncherText, copyThenPaste } from "./platform/paste.js";
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
const busy = computed(() => useBusy.value || selectionBusy.value);
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

watch(query, async (value) => {
  const request = ++searchRequest;
  const needle = value.trim();
  feedback.value = "";
  selectedIndex.value = 0;
  results.value = [];
  searching.value = !!needle;
  if (!needle) {
    results.value = [];
    return;
  }
  try {
    const rows = await listLocalPrompts({ query: needle });
    if (request === searchRequest) results.value = rows.slice(0, launcherPreferences.value.resultLimit);
  } catch (error) {
    if (request === searchRequest) { results.value = []; feedback.value = `搜索失败：${error.message || error}`; }
  } finally { if (request === searchRequest) searching.value = false; }
});

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
      const count = results.value.length;
      if (!count) return;
      selectedIndex.value = ((selectedIndex.value + delta) % count + count) % count;
    },
    moveTo: (index) => {
      selectedIndex.value = Math.max(0, Math.min(index, results.value.length - 1));
    },
    rowCount: results.value.length,
    current: () => results.value[selectedIndex.value] ?? null,
    activate,
    close: resetAndHide,
  });
}

async function activate(row, mode) {
  if (!row || busy.value) return;
  feedback.value = "";
  active.value = row;
  values.clear();
  for (const [name,value] of Object.entries(variableDefaults(row.content))) values.set(name,value);
  if (mode === "copy") {
    copyRendered();
    return;
  }
  step.value = "fill";
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

async function finishUse(text, id, result, pasteRequested) {
  feedback.value = result.message;
  let closeAfter = false;
  try {
    if (id) await recordLocalPromptUse(id);
    await setLocalSetting("last_rendered_prompt", text);
    closeAfter = await getLocalSetting("close_launcher_after_use") !== "0";
  } catch (error) { feedback.value += `；保存使用记录失败：${error.message || error}`; }
  if (result.ok && closeAfter) {
    await launcherCommand("hide_launcher");
    hidden = true;
    resetState();
  } else if (pasteRequested) {
    await launcherCommand("resume_launcher");
    hidden = false;
  }
  return text;
}

async function useRendered(pasteRequested = false) {
  if (busy.value || !active.value) return;
  const text = preview.value;
  const id = active.value.id;
  if (!text.trim()) { feedback.value = "提示词内容为空，未修改剪贴板。"; return; }
  useBusy.value = true;
  feedback.value = pasteRequested ? "正在复制并粘贴…" : "正在复制…";
  let copied = false;
  try {
    const result = pasteRequested
      ? await copyThenPaste(text)
      : (await copyLauncherText(text), { ok: true, message: "已复制" });
    copied = true;
    if (pasteRequested && result.ok) result.message = "已复制并发送粘贴指令";
    await finishUse(text, id, result, pasteRequested);
  } catch (error) {
    feedback.value = copied ? `已复制；窗口恢复失败：${error?.message || error}` : `复制失败，请检查剪贴板权限后重试；内容已保留。${error?.message || error}`;
  } finally {
    useBusy.value = false;
    await focusCurrent();
  }
}

const copyRendered = () => useRendered(false);
const pasteRendered = () => useRendered(true);

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
    if (didHide) { hidden = true; if (step.value !== 'fill') resetState(); }
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
  searchRequest += 1;
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
  if (step.value !== 'fill') resetState();
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
      if (!busy.value && (event?.payload !== 'blur' || step.value !== 'fill')) resetState();
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
  disposed = true;
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
