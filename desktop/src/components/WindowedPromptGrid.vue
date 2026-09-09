<template>
  <div ref="grid" class="prompt-grid" :class="{ 'list-view': list, 'windowed-grid': enabled }">
    <div v-if="enabled && range.top" aria-hidden="true" class="grid-spacer" :style="{ height: `${Math.max(0, range.top - gap)}px` }" />
    <template v-for="(item, index) in visible" :key="item.kind + item.id">
      <slot :item="item" :card-height="enabled ? rows[Math.floor((range.start + index) / columns)].height : undefined" />
    </template>
    <div v-if="enabled && range.bottom" aria-hidden="true" class="grid-spacer" :style="{ height: `${Math.max(0, range.bottom - gap)}px` }" />
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { buildRows, visibleRange } from '../platform/squareWindow.js';
const props = defineProps({ items: { type: Array, required: true }, enabled: Boolean, list: Boolean, scrollRoot: Object });
const emit = defineEmits(['near-end']);
const grid = ref(null), columns = ref(3), gap = ref(14), scroll = ref(0), viewport = ref(720);
const rows = computed(() => buildRows(props.items, columns.value, props.list, gap.value));
const range = computed(() => visibleRange(rows.value, scroll.value, viewport.value, columns.value, props.items.length));
const visible = computed(() => props.enabled ? props.items.slice(range.value.start, range.value.end) : props.items);
let observer, frame, root;
function measure() {
  frame = null;
  if (!props.enabled || !grid.value || !root || !grid.value.clientWidth) return;
  const style = getComputedStyle(grid.value);
  columns.value = props.list ? 1 : Math.max(1, style.gridTemplateColumns.split(' ').filter(Boolean).length);
  gap.value = parseFloat(style.rowGap) || 0;
  const top = grid.value.getBoundingClientRect().top - root.getBoundingClientRect().top + root.scrollTop;
  scroll.value = root.scrollTop - top;
  viewport.value = root.clientHeight;
  const last = rows.value.at(-1);
  if (last && scroll.value + viewport.value + 600 >= last.top + last.height) emit('near-end');
}
function schedule() { if (frame == null) frame = requestAnimationFrame(measure); }
function attach() {
  root?.removeEventListener('scroll', schedule);
  observer?.disconnect();
  root = props.scrollRoot;
  root?.addEventListener('scroll', schedule, { passive: true });
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(schedule);
    if (grid.value) observer.observe(grid.value);
    if (root) observer.observe(root);
  }
  schedule();
}
onMounted(attach);
watch(() => props.scrollRoot, attach);
watch(() => [props.items, props.list, props.enabled], async () => { await nextTick(); schedule(); });
onUnmounted(() => { root?.removeEventListener('scroll', schedule); observer?.disconnect(); if (frame != null) cancelAnimationFrame(frame); });
</script>

<style>
.grid-spacer { grid-column: 1 / -1; pointer-events: none; }
.windowed-grid > .prompt-card { box-sizing: border-box; overflow: hidden; }
.windowed-grid .square-reference-cover { height: 150px; flex-shrink: 0; aspect-ratio: auto; }
</style>
