<template>
  <input
    class="shortcut-input"
    readonly
    :value="formatShortcutLabel(modelValue, host)"
    :class="{ recording }"
    placeholder="点击后按组合键"
    :title="recording ? '请按组合键；Esc 取消，Tab 下一项' : '点击后直接按组合键'"
    @focus="start"
    @blur="stop"
    @keydown="record"
  >
</template>

<script setup>
import { onUnmounted, ref } from "vue";
import { formatShortcutLabel } from "../platform/windowChrome.js";
import { setShortcutRecording } from "../platform/shortcut.js";

const props = defineProps({ modelValue: String, host: String });
const emit = defineEmits(["update:modelValue"]);
const recording = ref(false);
let original;

function start() {
  original = props.modelValue;
  recording.value = true;
  setShortcutRecording(true);
}
function stop() {
  if (!recording.value) return;
  recording.value = false;
  setShortcutRecording(false);
}
function record(event) {
  if (event.key === "Tab" && !event.ctrlKey && !event.altKey && !event.metaKey) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.isComposing || event.keyCode === 229 || event.repeat) return;
  if (event.key === "Escape") {
    emit("update:modelValue", original ?? props.modelValue);
    stop();
    event.target.blur();
    return;
  }
  const code = event.code;
  let key;
  if (/^Key[A-Z]$/.test(code)) key = code.slice(3);
  else if (/^Digit[0-9]$/.test(code)) key = code.slice(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) key = code;
  else if (/^(Space|Enter|Tab|Backspace|Delete|Insert|Home|End|PageUp|PageDown|Arrow(Up|Down|Left|Right)|Backquote|Backslash|BracketLeft|BracketRight|Comma|Period|Slash|Semicolon|Quote|Minus|Equal|Numpad([0-9]|Add|Subtract|Multiply|Divide|Decimal|Enter|Equal))$/.test(code)) key = code;
  if (!key || (!event.ctrlKey && !event.altKey && !event.metaKey && !/^F\d+$/.test(key))) return;
  const modifiers = [event.ctrlKey && "Control", event.altKey && "Alt", event.shiftKey && "Shift", event.metaKey && "Super"].filter(Boolean);
  emit("update:modelValue", [...modifiers, key].join("+"));
}
onUnmounted(stop);
</script>

<style scoped>
.shortcut-input { cursor: pointer; font-variant-numeric: tabular-nums; }
.shortcut-input.recording { border-color: var(--accent); outline: 2px solid var(--accent-soft); }
</style>
