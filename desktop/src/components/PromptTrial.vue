<template>
  <aside class="prompt-trial" aria-label="试填预览" data-testid="prompt-trial">
    <h3>试填预览</h3><p class="use-hint">仅预览，不保存填写内容。</p>
    <label v-for="name in names" :key="name" class="field"><span>{{ name }}</span>
      <input :value="values[name]" :placeholder="name" @input="values = { ...values, [name]: $event.target.value }">
    </label>
    <p v-if="!names.length" class="use-hint">没有变量，以下是原始正文。</p>
    <pre data-testid="trial-result">{{ result || '输入正文后在这里查看效果' }}</pre>
  </aside>
</template>
<script setup>
import { computed, ref, watch } from 'vue';
import { extractVariables, renderPrompt, variableDefaults } from '../lib/renderPrompt.js';
const props = defineProps({ content: { type: String, default: '' } });
const names = computed(() => extractVariables(props.content));
const values = ref({});
watch(() => props.content, content => {
  const defaults = variableDefaults(content);
  values.value = Object.fromEntries(names.value.map(name => [name, Object.hasOwn(values.value, name) ? values.value[name] : Object.hasOwn(defaults, name) ? defaults[name] : '']));
}, { immediate: true });
const result = computed(() => renderPrompt(props.content, values.value));
</script>
<style scoped>
.prompt-trial { border: 1px solid var(--line); padding: 18px; border-radius: 10px; background: var(--sidebar); }
h3 { margin: 0; font-size: 14px; }
pre { white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; font-size: 13px; line-height: 1.8; margin-bottom: 0; }
</style>
