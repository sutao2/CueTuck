<template>
  <div class="modal-layer" data-testid="use-modal">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section v-dialog-focus="() => $emit('cancel')" class="modal create-modal" role="dialog" aria-modal="true" aria-labelledby="use-title">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">{{ prompt.title }}</p>
          <h2 id="use-title">{{ heading }}</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="create-body">
        <p v-if="error" role="alert" class="use-hint">{{ error }}</p>
        <template v-if="step === 'variable'">
          <p class="use-hint">填写后进入下一步。未填会在最终文本里保留原占位符。</p>
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
          <pre class="preview-box" data-testid="use-preview">{{ preview }}</pre>
        </template>
      </div>
      <footer class="modal-footer">
        <span class="create-location">{{ stepLabel }}</span>
        <div class="modal-actions">
          <button v-if="step !== 'preview' || names.length" type="button" class="button ghost-button" @click="back">
            上一步
          </button>
          <button ref="nextButton" type="button" class="button primary-button" data-testid="use-next" :disabled="busy" @click="next">
            {{ busy ? '正在复制…' : step === "preview" ? "复制并完成" : "下一步" }}
          </button>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup>
import { computed, ref, watch } from "vue";
import { vDialogFocus } from "../lib/dialogFocus.js";
import { extractVariables, renderPrompt } from "../lib/renderPrompt.js";
import { hintForVariable } from "../platform/variableHints.js";

const props = defineProps({
  prompt: { type: Object, required: true },
  hintsEnabled: { type: Boolean, default: false },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
});
const emit = defineEmits(["cancel", "copied"]);

const names = extractVariables(props.prompt.content);
const values = ref({});
const index = ref(0);
const currentValue = ref("");
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
      currentValue.value = values.value[names[index.value]] ?? "";
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
  if (step.value === "preview" && names.length) {
    step.value = "variable";
    index.value = names.length - 1;
    currentValue.value = values.value[currentName.value] ?? "";
    return;
  }
  if (index.value > 0) {
    values.value = { ...values.value, [currentName.value]: currentValue.value };
    index.value -= 1;
    currentValue.value = values.value[currentName.value] ?? "";
    return;
  }
  emit("cancel");
}
</script>
