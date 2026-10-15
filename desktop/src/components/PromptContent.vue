<template>
  <section class="prompt-content">
    <div class="reading-mode" role="group" :aria-label="tr('正文显示方式')">
      <button type="button" :aria-pressed="!source" @click="source = false">{{ tr('预览') }}</button>
      <button type="button" :aria-pressed="source" @click="source = true">{{ tr('原文') }}</button>
    </div>
    <pre v-if="source" class="prompt-source">{{ text }}</pre>
    <div v-else class="markdown-body" v-html="html" />
  </section>
</template>
<script setup>
import { computed, ref } from 'vue';
import { Marked } from 'marked';
import DOMPurify from 'dompurify';
import { tr } from '../platform/interfaceLanguage.js';
const props = defineProps({ text: { type: String, default: '' } });
const source = ref(false);
const escape = text => String(text).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const markdown = new Marked({ gfm: true, breaks: true, renderer: {
  html: ({ text }) => escape(text),
  image: ({ text }) => `<span>${escape(text)}</span>`,
} });
const html = computed(() => {
  const fragment = DOMPurify.sanitize(markdown.parse(props.text), {
    ALLOWED_TAGS: ['p','br','hr','h1','h2','h3','h4','h5','h6','strong','em','del','blockquote','pre','code','ul','ol','li','table','thead','tbody','tr','th','td','a','span','input'],
    ALLOWED_ATTR: ['href','title','class','align','type','checked','disabled','start'],
    RETURN_DOM_FRAGMENT: true,
  });
  for (const link of fragment.querySelectorAll('a')) {
    if (!/^(https?:\/\/|mailto:)/i.test(link.getAttribute('href') || '')) link.removeAttribute('href');
    else { link.setAttribute('target', '_blank'); link.setAttribute('rel', 'noopener noreferrer'); }
  }
  for (const input of fragment.querySelectorAll('input')) { input.type = 'checkbox'; input.disabled = true; }
  const container = document.createElement('div'); container.append(fragment); return container.innerHTML;
});
</script>
<style scoped>
.prompt-content { min-width: 0; }
.reading-mode { display: flex; gap: 4px; margin-bottom: 16px; }
.reading-mode button { border: 0; border-radius: var(--radius-pill, 999px); padding: 6px 12px; background: transparent; color: var(--muted); font: inherit; font-size: 12px; cursor: pointer; }
.reading-mode button[aria-pressed=true] { color: var(--text); background: var(--surface-secondary); }
.prompt-source { margin: 0; white-space: pre-wrap; overflow-wrap: anywhere; font: 13px/1.8 var(--font-code); }
.markdown-body { overflow-wrap: anywhere; font-size: 14px; line-height: 1.85; }
.markdown-body :deep(> :first-child) { margin-top: 0; }
.markdown-body :deep(> :last-child) { margin-bottom: 0; }
.markdown-body :deep(h1), .markdown-body :deep(h2), .markdown-body :deep(h3) { margin: 1.5em 0 .65em; line-height: 1.4; font-weight: 600; }
.markdown-body :deep(h1) { font-size: 24px; }.markdown-body :deep(h2) { font-size: 20px; }.markdown-body :deep(h3) { font-size: 17px; }
.markdown-body :deep(p), .markdown-body :deep(ul), .markdown-body :deep(ol) { margin: .8em 0; }
.markdown-body :deep(ul), .markdown-body :deep(ol) { padding-left: 1.6em; }
.markdown-body :deep(blockquote) { margin: 16px 0; padding: 4px 16px; border-left: 3px solid var(--line-strong); color: var(--muted); }
.markdown-body :deep(code) { font: .9em var(--font-code); background: var(--surface-secondary); border-radius: 4px; padding: 2px 5px; }
.markdown-body :deep(pre) { max-width: 100%; overflow: auto; padding: 16px; background: var(--surface-secondary); border: 1px solid var(--line); border-radius: var(--radius-control, 12px); line-height: 1.65; }
.markdown-body :deep(pre code) { padding: 0; background: none; white-space: pre; }
.markdown-body :deep(table) { display: block; max-width: 100%; overflow-x: auto; border-collapse: collapse; margin: 16px 0; }
.markdown-body :deep(th), .markdown-body :deep(td) { border: 1px solid var(--line); padding: 8px 12px; min-width: 90px; text-align: left; }
.markdown-body :deep(th) { background: var(--surface-secondary); font-weight: 600; }
.markdown-body :deep(hr) { border: 0; border-top: 1px solid var(--line); margin: 24px 0; }
.markdown-body :deep(a) { color: var(--blue); text-decoration: underline; }
</style>
