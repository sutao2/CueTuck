<template>
  <main
    class="launcher-canvas"
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
          <span class="brand-mark" aria-hidden="true"></span>
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
          <span class="brand-mark" aria-hidden="true"></span>
          <div class="result-copy">
            <span class="row-title">{{ active?.title }}</span>
            <span class="row-desc">{{ variableNames.length ? '填写变量后生成最终提示词' : '确认正文后复制或粘贴' }}</span>
          </div>
          <span class="pill">{{ variableNames.length ? '变量' : '预览' }}</span>
        </div>
        <div class="launcher-list launcher-fill-body">
          <div class="form-layout" :class="{ 'preview-only': !variableNames.length }">
            <form v-if="variableNames.length" ref="formEl" class="stack variable-form" @submit.prevent="copyRendered">
              <p class="fill-hint">Enter 下一项 / 最后一项复制 · Shift + Enter 换行<br>{{ copyChord }} 随时复制 · 留空保留占位符</p>
              <label v-for="(name, index) in variableNames" :key="name" class="field">
                <span>{{ name }}</span>
                <textarea v-model="values[name]" rows="2" :placeholder="`填写 ${name}`" :disabled="busy" @focus="focusedVariable = name" @keydown="onVariableKey($event, index)"></textarea>
              </label>
              <button
                v-if="canReadSelected"
                type="button"
                data-testid="read-selected"
                class="ghost"
                :disabled="busy"
                @click="readSelected"
              >
                读取原窗口选区 → {{ focusedVariable }}
              </button>
            </form>
            <section class="stack">
              <h3>预览最终提示词</h3>
              <pre class="preview code">{{ preview }}</pre>
              <p v-if="feedback" role="status" data-testid="launcher-feedback">{{ feedback }}</p>
            </section>
          </div>
        </div>
        <footer class="launcher-foot launcher-fill-foot">
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
import { extractVariables, renderPrompt } from "./lib/renderPrompt.js";
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

const props = defineProps({
  host: { type: String, default: () => detectHost() },
});

const LAUNCHER_RESULT_LIMIT = 20;
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
const values = reactive(Object.create(null));
const feedback = ref("");
const useBusy = ref(false);
const selectionBusy = ref(false);
const busy = computed(() => useBusy.value || selectionBusy.value);
let searchRequest = 0;
let disposed = false;
let unlisten = () => {};
let blurTimer;
const canReadSelected = !!window.__TAURI_INTERNALS__ && supportsSelectedText();

const variableNames = computed(() => extractVariables(active.value?.content ?? ""));
const preview = computed(() => renderPrompt(active.value?.content ?? "", values));
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
    if (request === searchRequest) results.value = rows.slice(0, LAUNCHER_RESULT_LIMIT);
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
  for (const key of Object.keys(values)) delete values[key];
  if (mode === "copy") {
    copyRendered(mode);
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
  if (disposed) return;
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
    resetState();
    await launcherCommand("hide_launcher");
  } else if (pasteRequested) {
    await launcherCommand("resume_launcher");
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
    if (selected?.trim()) { values[name] = selected; feedback.value = `已填入 ${name}`; }
    else feedback.value = "原窗口没有可读取的选区；已保留原内容。";
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    selectionBusy.value = false;
    await focusCurrent();
  }
}

async function onBlur() {
  if (!window.__TAURI_INTERNALS__ || busy.value || disposed) return;
  try {
    const hidden = await launcherCommand("hide_launcher_if_idle");
    if (hidden) resetState();
    else {
      clearTimeout(blurTimer);
      blurTimer = setTimeout(() => { if (!document.hasFocus()) onBlur(); }, 650);
    }
  } catch (error) { feedback.value = `隐藏窗口失败：${error}`; }
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
  for (const key of Object.keys(values)) delete values[key];
}

async function resetAndHide() {
  if (busy.value) return;
  try {
    await launcherCommand("hide_launcher");
    resetState();
    await focusCurrent();
  } catch (error) { feedback.value = `隐藏窗口失败：${error}`; }
}

async function refreshTheme() {
  const theme = await getLocalSetting("theme");
  if (!disposed) document.body.classList.toggle("theme-dark", theme === "dark");
}

async function onShown() {
  if (busy.value || disposed) return;
  resetState();
  await focusCurrent();
  try { await refreshTheme(); } catch (error) { feedback.value = `读取主题失败：${error}`; }
}

onMounted(async () => {
  document.body.classList.add("launcher-page");
  applyHostChrome(document.body, props.host);
  window.addEventListener("blur", onBlur);
  inputEl.value?.focus();
  try {
    unlisten = await listenLauncherLifecycle(onShown, () => { if (!busy.value) resetState(); }, (event) => { feedback.value = event.payload; });
    if (disposed) { unlisten(); return; }
    await refreshTheme();
  } catch (error) { feedback.value = `启动器初始化失败：${error}`; }
});

onUnmounted(() => {
  disposed = true;
  searchRequest += 1;
  clearTimeout(blurTimer);
  unlisten();
  window.removeEventListener("blur", onBlur);
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
  border-radius: 18px;
  overflow: hidden;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--bg) 94%, white), color-mix(in srgb, var(--surface) 88%, transparent)),
    color-mix(in srgb, var(--surface) 90%, transparent);
  box-shadow:
    inset 0 0 0 1px color-mix(in srgb, var(--text) 10%, transparent),
    inset 0 1px 0 color-mix(in srgb, white 70%, transparent);
  backdrop-filter: blur(28px) saturate(165%);
  -webkit-backdrop-filter: blur(28px) saturate(165%);
}
.launcher-stage.is-collapsed {
  grid-template-rows: 1fr;
}
.launcher-stage.is-fill {
  display: block;
  position: relative;
}
.launcher-search-wrap {
  min-height: 70px;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  border-bottom: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
}
.launcher-stage.is-collapsed .launcher-search-wrap {
  min-height: 80px;
  border-bottom: 0;
}
.brand-mark {
  width: 30px;
  height: 30px;
  border: 1px solid color-mix(in srgb, white 52%, transparent);
  border-radius: 10px;
  background: var(--accent);
  box-shadow: 0 8px 20px color-mix(in srgb, var(--accent) 35%, transparent);
}
.launcher-search {
  width: 100%;
  height: 48px;
  border: 0;
  background: transparent;
  appearance: none;
  -webkit-appearance: none;
  font-size: 18px;
  font-weight: 520;
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
  min-height: 22px;
  padding: 0 8px;
  border: 1px solid var(--line);
  border-radius: 999px;
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
  padding: 8px;
}
.group-title {
  margin: 4px 8px 8px;
  color: var(--muted);
  font-size: 11px;
}
.result-row {
  width: 100%;
  min-height: 58px;
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
.result-row.active,
.result-row:hover {
  background: color-mix(in srgb, var(--text) 6%, transparent);
}
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
  color: #fff;
}
.result-copy {
  display: grid;
  min-width: 0;
  gap: 2px;
}
.row-title {
  font-size: 14px;
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
  min-height: 46px;
  padding: 0 12px;
  border-top: 1px solid var(--line);
  color: var(--muted);
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
  height: 70px;
}
.launcher-fill-body {
  position: absolute;
  inset: 70px 0 52px;
  padding: 12px;
}
.launcher-fill-foot {
  position: absolute;
  inset: auto 0 0;
  height: 52px;
}
.form-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 16px;
  height: 100%;
}
.form-layout.preview-only { grid-template-columns: minmax(0, 1fr); }
.variable-form { overflow: auto; padding: 2px 4px 8px 2px; }
.fill-hint { margin: 0 0 4px; font-size: 11px; line-height: 1.6; color: var(--muted); }
.stack {
  display: grid;
  align-content: start;
  gap: 8px;
  min-width: 0;
}
.field {
  display: grid;
  gap: 4px;
  color: var(--muted);
  font-size: 11px;
}
.field textarea {
  width: 100%;
  min-height: 62px;
  resize: vertical;
  color: var(--text);
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 8px 10px;
  background: var(--bg);
}
.preview,
.code {
  margin: 0;
  max-height: 280px;
  overflow: auto;
  padding: 10px;
  border-radius: 8px;
  background: var(--bg);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-size: 12px;
}
h3 {
  margin: 0;
  font-size: 13px;
}
.primary,
.ghost {
  border: 0;
  border-radius: 6px;
  padding: 6px 10px;
}
button:focus-visible, textarea:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
button:disabled { opacity: .5; cursor: wait; }
.primary {
  background: var(--accent);
  color: #fff;
}
.ghost {
  background: transparent;
  color: var(--muted);
}
</style>
