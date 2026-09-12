<template>
    <section v-page-focus="() => !busy && $emit('cancel')" class="workspace-page" data-testid="use-modal" role="region" aria-labelledby="use-title">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">{{ prompt.title }}</p>
          <h2 id="use-title">{{ heading }}</h2>
        </div>
        <button type="button" class="page-back" aria-label="返回" :disabled="busy" @click="$emit('cancel')">← 返回</button>
      </header>
      <div class="create-body">
        <nav v-if="names.length" class="variable-steps" aria-label="填写步骤">
          <button v-for="(name, position) in names" :key="name" type="button" :disabled="busy" :aria-current="step === 'variable' && index === position ? 'step' : undefined" :data-variable-step="position" @click="jump(position)">
            <span>{{ position + 1 }}</span> {{ name }} <small>{{ resolved(name) ? '已填' : '待填' }}</small>
          </button>
          <button type="button" :disabled="busy" :aria-current="step === 'preview' ? 'step' : undefined" data-testid="jump-preview" @click="showPreview">预览</button>
        </nav>
        <p v-if="error" role="alert" class="use-hint">{{ error }}</p>
        <template v-if="step === 'variable'">
          <p class="use-hint">填写后进入下一步。留空时使用默认值，没有默认值则保留原占位符。</p>
          <label class="field">
            <span data-testid="use-variable">{{ currentName }}</span>
            <textarea
              ref="variableInput"
              v-model="currentValue"
              rows="4"
              data-testid="use-value"
              :placeholder="'请输入' + currentName"
              @keydown="onValueKeydown"
            ></textarea>
            <small v-if="currentHint" data-testid="variable-hint">{{ currentHint }}</small>
            <small class="field-help">Enter 下一步 · Shift+Enter 换行</small>
          </label>
        </template>
        <template v-else>
          <p class="use-hint">确认后复制到剪贴板，并记一次使用。</p>
          <p v-if="missing.length" class="use-hint" data-testid="missing-variables">还有 {{ missing.length }} 项未填写，复制时将保留占位符。点击上方参数可补填。</p>
          <pre class="preview-box" data-testid="use-preview">{{ preview }}</pre>
        </template>
        <details v-if="prompt.asset_count" class="use-assets" @toggle="loadAssets">
          <summary>参考资料 · {{ prompt.asset_count }} 个附件 <span>复制仅包含正文</span></summary>
          <p v-if="assetError" role="alert">{{ assetError }} <button type="button" @click="loadAssets">重试</button></p>
          <p v-if="assetLoading" role="status">正在读取附件…</p>
          <AttachmentPanel v-else :model-value="assets" :prompt-id="prompt.id" :saved-ids="assets.map(a => a.id)" readonly />
        </details>
      </div>
      <footer class="modal-footer">
        <span class="create-location">{{ stepLabel }}</span>
        <div class="modal-actions">
          <button v-if="step !== 'preview' || names.length" type="button" class="button ghost-button" :disabled="busy" @click="back">
            上一步
          </button>
          <button ref="nextButton" type="button" class="button primary-button" data-testid="use-next" :disabled="busy" @click="next">
            {{ busy ? '正在复制…' : step === "preview" ? "复制并完成" : "下一步" }}
          </button>
        </div>
      </footer>
    </section>
</template>

<script setup>
import { computed, ref, watch } from "vue";
import { vPageFocus } from "../lib/pageFocus.js";
import { extractVariables, renderPrompt, variableDefaults } from "../lib/renderPrompt.js";
import { hintForVariable } from "../platform/variableHints.js";
import AttachmentPanel from './AttachmentPanel.vue';
import { listPromptAssets } from '../platform/assets.js';

const assets = ref([]), assetLoading = ref(false), assetError = ref('');
let assetsLoaded = false;
async function loadAssets(event) {
  if (assetsLoaded || assetLoading.value || (event?.target?.tagName === 'DETAILS' && !event.target.open)) return;
  assetLoading.value = true; assetError.value = '';
  try { assets.value = await listPromptAssets(props.prompt.id); assetsLoaded = true; }
  catch (err) { assetError.value = `附件读取失败：${err.message || err}`; }
  finally { assetLoading.value = false; }
}

const props = defineProps({
  prompt: { type: Object, required: true },
  hintsEnabled: { type: Boolean, default: false },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
});
const emit = defineEmits(["cancel", "copied"]);

const names = extractVariables(props.prompt.content);
const values = ref(variableDefaults(props.prompt.content));
const index = ref(0);
const currentValue = ref(Object.hasOwn(values.value,names[0]) ? values.value[names[0]] : "");
const step = ref(names.length ? "variable" : "preview");
const variableInput = ref(null);
const nextButton = ref(null);
watch([step, index], () => {
  (step.value === 'variable' ? variableInput.value : nextButton.value)?.focus();
}, { flush: 'post' });

const currentName = computed(() => names[index.value] ?? "");
const currentHint = computed(() =>
  props.hintsEnabled ? hintForVariable(currentName.value) : "",
);
const defaults = variableDefaults(props.prompt.content);
function resolved(name) {
  const value = step.value === 'variable' && currentName.value === name ? currentValue.value : values.value[name];
  return (Object.hasOwn(values.value, name) || name === currentName.value) && value !== '' && value != null || Object.hasOwn(defaults, name);
}
const missing = computed(() => names.filter(name => !resolved(name)));
function preserveValue() {
  if (step.value === 'variable') values.value = { ...values.value, [currentName.value]: currentValue.value };
}
function jump(position) {
  if (props.busy) return;
  preserveValue(); index.value = position; step.value = 'variable';
  currentValue.value = Object.hasOwn(values.value, names[position]) ? values.value[names[position]] : '';
}
function showPreview() {
  if (props.busy) return;
  preserveValue(); step.value = 'preview';
}
const preview = computed(() => renderPrompt(props.prompt.content, values.value));
const heading = computed(() => (step.value === "preview" ? "确认并使用提示词" : currentName.value));
const stepLabel = computed(() =>
  step.value === "preview" ? "预览" : `变量 ${index.value + 1} / ${names.length}`,
);

function next() {
  if (props.busy) return;
  if (step.value === "variable") {
    values.value = { ...values.value, [currentName.value]: currentValue.value };
    if (index.value < names.length - 1) {
      index.value += 1;
      currentValue.value = Object.hasOwn(values.value,names[index.value]) ? values.value[names[index.value]] : "";
      return;
    }
    step.value = "preview";
    return;
  }
  emit("copied", preview.value);
}

function onValueKeydown(event) {
  if (event.key !== "Enter" || event.shiftKey || event.isComposing) return;
  event.preventDefault();
  next();
}

function back() {
  if (props.busy) return;
  if (step.value === "preview" && names.length) {
    step.value = "variable";
    index.value = names.length - 1;
    currentValue.value = Object.hasOwn(values.value,currentName.value) ? values.value[currentName.value] : "";
    return;
  }
  if (index.value > 0) {
    values.value = { ...values.value, [currentName.value]: currentValue.value };
    index.value -= 1;
    currentValue.value = Object.hasOwn(values.value,currentName.value) ? values.value[currentName.value] : "";
    return;
  }
  emit("cancel");
}
</script>

<style scoped>
.variable-steps { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 24px; }
.variable-steps button { display: flex; align-items: center; gap: 7px; padding: 8px 12px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); color: var(--muted); font: inherit; font-size: 12px; max-width: 100%; overflow-wrap: anywhere; }
.variable-steps button[aria-current] { border-color: var(--text); color: var(--text); background: var(--sidebar); }
.variable-steps small { font-size: 10px; flex-shrink: 0; }
</style>
