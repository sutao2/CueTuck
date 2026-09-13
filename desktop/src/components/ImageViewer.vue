<template>
  <Teleport to="body">
    <div class="image-backdrop" @click.self="$emit('close')">
      <section v-dialog-focus="() => $emit('close')" class="image-viewer" role="dialog" aria-modal="true" :aria-label="`图片预览：${title}`" @keydown="onKey">
        <header class="image-heading">
          <div class="image-caption"><strong>{{ title }}</strong><span v-if="loaded">{{ naturalWidth }} × {{ naturalHeight }}</span></div>
          <button type="button" data-dialog-autofocus aria-label="关闭图片预览" title="关闭 (Esc)" @click="$emit('close')"><AppIcon name="close" /></button>
        </header>
        <div ref="canvas" class="image-canvas" @click.self="$emit('close')" @wheel.prevent="onWheel">
          <p v-if="failed" class="image-message" role="alert"><AppIcon name="image" />图片无法加载，请关闭后重试。</p>
          <p v-else-if="!loaded" class="image-message" role="status">正在加载图片…</p>
          <img v-if="!failed" :key="src" :src="src" :alt="title" referrerpolicy="no-referrer" :class="{ loaded, draggable: canPan, dragging }" :style="imageStyle" draggable="false" @load="onLoad" @error="failed = true; loaded = false" @dblclick.prevent="toggleSize" @pointerdown="startPan" @pointermove="movePan" @pointerup="endPan" @pointercancel="endPan" @lostpointercapture="endPan">
        </div>
        <footer class="image-toolbar" aria-label="图片工具栏">
          <button type="button" :disabled="!loaded || scale <= .1" aria-label="缩小图片" title="缩小 (−)" @click="resize(-.25)"><AppIcon name="minus" /></button>
          <span class="image-scale">{{ loaded ? Math.round(scale * 100) + '%' : '—' }}</span>
          <button type="button" :disabled="!loaded || scale >= 4" aria-label="放大图片" title="放大 (+)" @click="resize(.25)"><AppIcon name="plus" /></button>
          <i aria-hidden="true" />
          <button type="button" :disabled="!loaded" :aria-pressed="zoom === null" title="适应窗口 (0)" @click="setZoom(null)"><AppIcon name="fit" /><span>适应</span></button>
          <button type="button" :disabled="!loaded" :aria-pressed="zoom === 1" title="原始尺寸 (1)" @click="setZoom(1)">1:1</button>
        </footer>
      </section>
    </div>
  </Teleport>
</template>
<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { vDialogFocus } from '../lib/dialogFocus.js';
import AppIcon from './AppIcon.vue';
const props = defineProps({ src: { type: String, required: true }, title: { type: String, default: '图片预览' } });
defineEmits(['close']);
const canvas = ref(null), zoom = ref(null), naturalWidth = ref(0), naturalHeight = ref(0), loaded = ref(false), failed = ref(false);
const width = ref(1), height = ref(1), x = ref(0), y = ref(0), dragging = ref(false);
const fitScale = computed(() => Math.min(1, width.value / (naturalWidth.value || 1), height.value / (naturalHeight.value || 1)));
const scale = computed(() => zoom.value ?? fitScale.value);
const canPan = computed(() => naturalWidth.value * scale.value > width.value || naturalHeight.value * scale.value > height.value);
const imageStyle = computed(() => ({ width: `${naturalWidth.value * scale.value}px`, height: `${naturalHeight.value * scale.value}px`, transform: `translate(${x.value}px, ${y.value}px)` }));
let observer, drag;
function measure() { width.value = Math.max(1, canvas.value.clientWidth - 48); height.value = Math.max(1, canvas.value.clientHeight - 32); clampPan(); }
function clampPan() { const dx = Math.max(0, (naturalWidth.value * scale.value - width.value) / 2), dy = Math.max(0, (naturalHeight.value * scale.value - height.value) / 2); x.value = Math.max(-dx, Math.min(dx, x.value)); y.value = Math.max(-dy, Math.min(dy, y.value)); }
function onLoad(event) { naturalWidth.value = event.target.naturalWidth; naturalHeight.value = event.target.naturalHeight; loaded.value = true; measure(); }
function setZoom(value) { if (!loaded.value) return; zoom.value = value; x.value = y.value = 0; }
function resize(delta) { if (loaded.value) { zoom.value = Math.max(.1, Math.min(4, scale.value + delta)); clampPan(); } }
function toggleSize() { setZoom(zoom.value === 1 ? null : 1); }
function onWheel(event) { if (event.deltaY) resize(event.deltaY < 0 ? .125 : -.125); }
function startPan(event) { if (event.button !== 0 || !canPan.value) return; event.preventDefault(); drag = { id: event.pointerId, left: event.clientX - x.value, top: event.clientY - y.value }; dragging.value = true; event.currentTarget.setPointerCapture?.(event.pointerId); }
function movePan(event) { if (!drag || drag.id !== event.pointerId) return; x.value = event.clientX - drag.left; y.value = event.clientY - drag.top; clampPan(); }
function endPan() { drag = null; dragging.value = false; }
function onKey(event) { if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey) return; if (['+', '=', '-', '0', '1'].includes(event.key)) { event.preventDefault(); if (event.key === '0') setZoom(null); else if (event.key === '1') setZoom(1); else resize(event.key === '-' ? -.25 : .25); } }
watch(() => props.src, () => { zoom.value = null; loaded.value = failed.value = false; naturalWidth.value = naturalHeight.value = x.value = y.value = 0; endPan(); });
onMounted(() => { measure(); if (typeof ResizeObserver !== 'undefined') { observer = new ResizeObserver(measure); observer.observe(canvas.value); } });
onUnmounted(() => observer?.disconnect());
</script>
<style scoped>
.image-backdrop { position: fixed; inset: 0; z-index: 2200; background: rgba(15, 17, 20, .96); }
.image-viewer { position: absolute; inset: 0; display: flex; flex-direction: column; color: #f4f4f5; outline: none; }
.image-heading { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 18px 24px 10px; flex-shrink: 0; }
.image-caption { min-width: 0; display: flex; align-items: baseline; gap: 16px; }
.image-caption strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px; font-weight: 500; }
.image-caption span { color: #9da1a8; font-size: 11px; white-space: nowrap; font-variant-numeric: tabular-nums; }
.image-viewer button { display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; gap: 7px; min-width: 34px; height: 34px; padding: 0 8px; border: 0; border-radius: 7px; background: transparent; color: #e7e8eb; font: 12px var(--font-body, sans-serif); cursor: pointer; }
.image-viewer button:hover, .image-viewer button[aria-pressed=true] { background: #ffffff18; }
.image-viewer button:focus-visible { outline: 2px solid #adbfcf; outline-offset: 2px; }
.image-viewer button:disabled { opacity: .3; cursor: default; }
.image-viewer .app-icon { width: 18px; height: 18px; }
.image-canvas { flex: 1; min-height: 0; position: relative; display: flex; align-items: center; justify-content: center; overflow: hidden; margin-bottom: 76px; }
.image-canvas img { flex: none; max-width: none; max-height: none; object-fit: contain; opacity: 0; user-select: none; -webkit-user-select: none; touch-action: none; box-shadow: 0 4px 32px #0004; }
.image-canvas img.loaded { opacity: 1; }
.image-canvas img.draggable { cursor: grab; }
.image-canvas img.dragging { cursor: grabbing; }
.image-message { position: absolute; display: flex; align-items: center; gap: 10px; color: #b6bac2; font-size: 13px; }
.image-toolbar { position: absolute; bottom: 22px; left: 50%; transform: translateX(-50%); display: flex; align-items: center; gap: 4px; padding: 5px 7px; border: 1px solid #ffffff1a; border-radius: 12px; background: #272a2f; box-shadow: 0 4px 18px #0003; }
.image-toolbar i { width: 1px; height: 18px; margin: 0 6px; background: #ffffff24; }
.image-scale { min-width: 46px; text-align: center; font-size: 12px; font-variant-numeric: tabular-nums; color: #c8cbd1; }
@media (max-width: 600px) { .image-heading { padding: 12px 14px 8px; } .image-caption { display: grid; gap: 3px; } }
</style>
