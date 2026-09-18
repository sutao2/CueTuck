<template>
  <div ref="grid" class="prompt-grid" :class="{ 'list-view': list, 'windowed-grid': enabled, 'content-grid': !enabled && !list }">
    <div v-if="enabled && range.top" aria-hidden="true" class="grid-spacer" :style="{ height: `${Math.max(0, range.top - gap)}px` }" />
    <template v-for="(item, index) in visible" :key="item.kind + item.id">
      <slot :item="item" :card-height="enabled ? rows[Math.floor((range.start + index) / columns)].height : undefined" />
    </template>
    <div v-if="enabled && range.bottom" aria-hidden="true" class="grid-spacer" :style="{ height: `${Math.max(0, range.bottom - gap)}px` }" />
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, onUpdated, ref, watch } from 'vue';
import { buildRows, visibleRange } from '../platform/squareWindow.js';
const props = defineProps({ items: { type: Array, required: true }, enabled: Boolean, list: Boolean, scrollRoot: Object });
const emit = defineEmits(['near-end']);
const grid = ref(null), columns = ref(3), gap = ref(14), scroll = ref(0), viewport = ref(720);
const rows = computed(() => buildRows(props.items, columns.value, props.list, gap.value));
const range = computed(previous => {
  const next = visibleRange(rows.value, scroll.value, viewport.value, columns.value, props.items.length);
  // Keep slot rendering stable while scrolling inside the same buffered rows.
  return previous && next.start === previous.start && next.end === previous.end && next.top === previous.top && next.bottom === previous.bottom ? previous : next;
});
const visible = computed(() => props.enabled ? props.items.slice(range.value.start, range.value.end) : props.items);
let observer, frame, root, measureStyles = true;
let cardObserver;
const observedCards = new Set();
function observeCards() {
  if (props.enabled || props.list || !grid.value || typeof ResizeObserver === 'undefined') return;
  if (!cardObserver) cardObserver = new ResizeObserver(entries => {
    // Read only resized cards; scrolling never measures the local grid.
    for (const entry of entries) {
      const height = entry.borderBoxSize?.[0]?.blockSize ?? entry.target.getBoundingClientRect().height;
      if (height > 0) entry.target.style.gridRowEnd = `span ${Math.ceil(height + 14)}`;
    }
  });
  const cards = new Set(grid.value.querySelectorAll(':scope > .prompt-card'));
  for (const card of observedCards) if (!cards.has(card)) { cardObserver.unobserve(card); observedCards.delete(card); }
  for (const card of cards) if (!observedCards.has(card)) { observedCards.add(card); cardObserver.observe(card); }
}
function clearCards() {
  cardObserver?.disconnect(); cardObserver = null;
  for (const card of observedCards) card.style.removeProperty('grid-row-end');
  observedCards.clear();
}
function measure() {
  frame = null;
  if (!props.enabled || !grid.value || !root || !grid.value.clientWidth) return;
  if (measureStyles) {
    const style = getComputedStyle(grid.value);
    columns.value = props.list ? 1 : Math.max(1, style.gridTemplateColumns.split(' ').filter(Boolean).length);
    gap.value = parseFloat(style.rowGap) || 0;
    measureStyles = false;
  }
  const top = grid.value.getBoundingClientRect().top - root.getBoundingClientRect().top + root.scrollTop;
  scroll.value = root.scrollTop - top;
  viewport.value = root.clientHeight;
  const last = rows.value.at(-1);
  if (last && scroll.value + viewport.value + 600 >= last.top + last.height) emit('near-end');
}
function schedule() { if (props.enabled && frame == null) frame = requestAnimationFrame(measure); }
function scheduleLayout() { measureStyles = true; schedule(); }
function attach() {
  root?.removeEventListener('scroll', schedule);
  observer?.disconnect();
  if (frame != null) { cancelAnimationFrame(frame); frame = null; }
  root = props.scrollRoot;
  clearCards();
  if (!props.enabled) { observeCards(); return; }
  root?.addEventListener('scroll', schedule, { passive: true });
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(scheduleLayout);
    if (grid.value) observer.observe(grid.value);
    if (root) observer.observe(root);
  }
  scheduleLayout();
}
onMounted(attach);
onUpdated(observeCards);
watch(() => [props.scrollRoot, props.enabled], attach);
watch(() => props.list, async () => { await nextTick(); attach(); });
watch(() => props.items, async () => { await nextTick(); scheduleLayout(); });
onUnmounted(() => { clearCards(); root?.removeEventListener('scroll', schedule); observer?.disconnect(); if (frame != null) cancelAnimationFrame(frame); });
</script>

<style>
.content-grid { grid-auto-rows: 1px; row-gap: 0 !important; align-items: start; }
.content-grid > .prompt-card { grid-row-end: span 400; min-height: 0; }
.prompt-grid.content-grid > .prompt-card:not(.as-row) h3 { min-height: 0; }
.grid-spacer { grid-column: 1 / -1; pointer-events: none; }
.windowed-grid:not(.list-view) { grid-template-columns: repeat(auto-fill, minmax(min(100%, 280px), 1fr)); }
.windowed-grid > .prompt-card { box-sizing: border-box; overflow: hidden; }
.windowed-grid > .prompt-card:not(.as-row) .card-footer { flex-wrap: wrap; align-items: center; }
.windowed-grid .card-action { flex-shrink: 0; white-space: nowrap; }
.windowed-grid > .prompt-card:not(.as-row) .prompt-excerpt { -webkit-line-clamp: 2; flex-shrink: 0; }
.windowed-grid .prompt-author { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.windowed-grid .square-reference-cover { height: 150px; flex-shrink: 0; aspect-ratio: auto; }
</style>
