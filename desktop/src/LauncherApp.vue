<template>
  <main
    class="launcher-canvas"
    :class="{ 'host-mac': host === 'macos' }"
    aria-label="快捷搜索"
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
            @keydown="onSearchKey"
          />
          <span class="launcher-window-tools">
            <span class="pill">本地</span>
            <button class="launcher-close-btn" type="button" title="关闭 (Esc)" @click="resetAndHide">
              Esc
            </button>
          </span>
        </div>

        <div v-if="!isCollapsed" class="launcher-list">
          <div v-if="results.length" id="launcher-results" role="listbox" aria-label="搜索结果">
            <p class="group-title">本地提示词</p>
            <button
              v-for="(row, index) in results"
              :key="row.id"
              class="result-row"
              :class="{ active: index === selectedIndex }"
              type="button"
              role="option"
              :aria-selected="index === selectedIndex"
              tabindex="-1"
              @mousedown.prevent="activate(row, 'default')"
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
          <p v-else class="launcher-empty">没有找到相关提示词</p>
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
            <span class="row-desc">填写变量后生成最终提示词</span>
          </div>
          <span class="pill">变量</span>
        </div>
        <div class="launcher-list launcher-fill-body">
          <div class="form-layout">
            <form class="stack" @submit.prevent="copyRendered('default')">
              <label v-for="name in variableNames" :key="name" class="field">
                <span>{{ name }}</span>
                <input v-model="values[name]" :placeholder="`填写 ${name}`">
              </label>
              <button
                v-if="canReadSelected"
                type="button"
                data-testid="read-selected"
                class="ghost"
                @click="readSelected"
              >
                读取选中文本
              </button>
            </form>
            <section class="stack">
              <h3>预览最终提示词</h3>
              <pre class="preview code">{{ preview }}</pre>
            </section>
          </div>
        </div>
        <footer class="launcher-foot launcher-fill-foot">
          <div class="launcher-actions">
            <button type="button" class="ghost" @click="backToSearch">返回</button>
            <button type="button" class="ghost" @click="copyRendered('copy')">复制</button>
            <button type="button" class="primary" @click="pasteRendered">粘贴到当前窗口</button>
          </div>
        </footer>
      </template>
    </section>
  </main>
</template>

<script setup>
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { extractVariables, renderPrompt } from "./lib/renderPrompt.js";
import { handleLauncherSearchKey } from "./platform/launcherKeyboard.js";
import {
  resizeLauncherWindow,
  startDraggingLauncher,
} from "./platform/launcherWindow.js";
import { getLocalSetting, listLocalPrompts, recordLocalPromptUse } from "./platform/library.js";
import { copyThenPaste } from "./platform/paste.js";
import { supportsSelectedText } from "./platform/selectedText.js";
import { applyHostChrome, detectHost, formatShortcutLabel } from "./platform/windowChrome.js";

const props = defineProps({
  host: { type: String, default: () => detectHost() },
});

const LAUNCHER_RESULT_LIMIT = 20;
const query = ref("");
const inputEl = ref(null);
const databaseStatus = ref("pending");
const results = ref([]);
const selectedIndex = ref(0);
const step = ref("search");
const active = ref(null);
const values = reactive({});
const feedback = ref("");
const canReadSelected = supportsSelectedText();

const variableNames = computed(() => extractVariables(active.value?.content ?? ""));
const preview = computed(() => renderPrompt(active.value?.content ?? "", values));
const isCollapsed = computed(() => step.value === "search" && !query.value.trim());
const launcherLayout = computed(() =>
  step.value === "fill" ? "fill" : isCollapsed.value ? "collapsed" : "expanded",
);
const copyChord = computed(() => formatShortcutLabel("Control+Enter", props.host));

watch(query, async (value) => {
  const needle = value.trim();
  selectedIndex.value = 0;
  if (!needle) {
    results.value = [];
    return;
  }
  results.value = (await listLocalPrompts({ query: needle })).slice(0, LAUNCHER_RESULT_LIMIT);
});

watch(launcherLayout, (layout) => resizeLauncherWindow(layout), { immediate: true });

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
  handleLauncherSearchKey(event, {
    move: (delta) => {
      const count = results.value.length;
      if (!count) return;
      selectedIndex.value = (selectedIndex.value + delta + count) % count;
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

function activate(row, mode) {
  if (!row) return;
  active.value = row;
  for (const key of Object.keys(values)) delete values[key];
  const names = extractVariables(row.content);
  if (mode === "copy" || !names.length) {
    copyRendered(mode);
    return;
  }
  step.value = "fill";
}

function backToSearch() {
  step.value = "search";
  active.value = null;
  inputEl.value?.focus();
}

function startDragFromChrome(event) {
  const interactive = event.target.closest(
    "input, textarea, select, button, a, .launcher-list, .launcher-foot, .result-row",
  );
  if (!interactive) startDraggingLauncher();
}

async function finishUse(text, pasted) {
  if (active.value?.id) {
    try {
      await recordLocalPromptUse(active.value.id);
    } catch {
      /* ignore missing row in tests */
    }
  }
  feedback.value = pasted === false ? "已复制，未能粘贴" : "已复制";
  const closeAfter = await getLocalSetting("close_launcher_after_use");
  if (pasted !== false && closeAfter !== "0") {
    await resetAndHide();
  }
  return text;
}

async function copyRendered() {
  const text = renderPrompt(active.value?.content ?? "", values);
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    /* tests and some browsers deny clipboard */
  }
  await finishUse(text, true);
}

async function pasteRendered() {
  const text = renderPrompt(active.value?.content ?? "", values);
  const result = await copyThenPaste(text);
  await finishUse(text, result.ok);
}

async function readSelected() {
  if (!window.__TAURI_INTERNALS__) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const selected = await invoke("capture_selected_text");
    const name = variableNames.value[0];
    if (name) values[name] = selected;
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  }
}

async function onBlur() {
  if (!window.__TAURI_INTERNALS__) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("hide_launcher_if_idle");
  } catch {
    /* ignore */
  }
}

async function resetAndHide() {
  query.value = "";
  results.value = [];
  selectedIndex.value = 0;
  step.value = "search";
  active.value = null;
  await resizeLauncherWindow("collapsed");
  if (!window.__TAURI_INTERNALS__) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("hide_launcher");
  } catch {
    /* window may already be hidden */
  }
}

onMounted(async () => {
  document.body.classList.add("launcher-page");
  applyHostChrome(document.body, props.host);
  window.addEventListener("blur", onBlur);
  inputEl.value?.focus();
  const theme = await getLocalSetting("theme");
  document.body.classList.toggle("theme-dark", theme === "dark");
  if (!window.__TAURI_INTERNALS__) {
    return;
  }
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    databaseStatus.value = await invoke("get_local_database_status");
  } catch {
    databaseStatus.value = "failed";
  }
});

onUnmounted(() => {
  window.removeEventListener("blur", onBlur);
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
.field input {
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
.primary {
  background: var(--accent);
  color: #fff;
}
.ghost {
  background: transparent;
  color: var(--muted);
}
</style>
