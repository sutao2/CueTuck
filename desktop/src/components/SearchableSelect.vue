<template>
  <button ref="trigger" v-bind="$attrs" type="button" :value="modelValue" class="searchable-select" :title="$attrs.title || selected?.label || placeholder" :disabled="disabled" aria-haspopup="listbox" :aria-expanded="open" :aria-controls="open ? listId : undefined" @click="toggle" @keydown.down.prevent="show" @keydown.up.prevent="show">
    <span>{{ selected?.label || placeholder }}</span><svg class="select-chevron" aria-hidden="true" viewBox="0 0 16 16"><path d="m4 6 4 4 4-4" /></svg>
  </button>
  <Teleport to="body">
    <div v-if="open" ref="panel" class="select-popup" :style="position" @keydown.esc.stop.prevent="close" @keydown.tab="close(false)">
      <div v-if="searchable" class="select-search"><svg aria-hidden="true" viewBox="0 0 20 20"><circle cx="8.5" cy="8.5" r="5.5"/><path d="m13 13 4 4"/></svg><input ref="search" v-model="query" role="combobox" aria-label="搜索选项" aria-autocomplete="list" aria-expanded="true" :aria-controls="listId" :aria-activedescendant="visible[active] ? `${listId}-${active}` : undefined" placeholder="搜索选项…" @keydown="navigate"></div>
      <div :id="listId" ref="list" role="listbox" :aria-label="$attrs['aria-label'] || placeholder" :tabindex="searchable ? -1 : 0" :aria-activedescendant="!searchable && visible[active] ? `${listId}-${active}` : undefined" class="select-results" @keydown="!searchable && navigate($event)">
        <button v-for="(option,index) in visible" :id="`${listId}-${index}`" :key="index" type="button" role="option" tabindex="-1" :aria-selected="option.value === modelValue" :class="{ highlighted: active === index }" @mousedown.prevent @click="choose(option)"><span>{{ option.label }}</span><svg v-if="option.value === modelValue" class="select-check" aria-hidden="true" viewBox="0 0 20 20"><path d="m5 10 3 3 7-7" /></svg></button>
        <p v-if="!matches.length" role="status">没有匹配选项，请尝试其他关键词</p>
      </div>
      <small v-if="matches.length > visible.length">显示前 {{ visible.length }} / {{ matches.length }} 项，请搜索缩小范围</small>
      <button v-if="query" type="button" class="select-clear" @click="query = ''; search?.focus()">清空搜索</button>
    </div>
  </Teleport>
</template>
<script setup>
import { computed, nextTick, onUnmounted, ref, useId, watch } from 'vue';
defineOptions({ inheritAttrs: false });
const props = defineProps({ modelValue: { default: '' }, options: { type:Array, default:() => [] }, placeholder: { type:String, default:'请选择' }, disabled:Boolean, searchable:{type:Boolean,default:true} });
const emit = defineEmits(['update:modelValue','change']);
const open=ref(false), query=ref(''), active=ref(0), trigger=ref(null), search=ref(null), list=ref(null), panel=ref(null), position=ref({});
const listId=`select-${useId()}`;
const selected=computed(() => props.options.find(item => item.value === props.modelValue));
const matches=computed(() => props.options.filter(item => String(item.label).toLocaleLowerCase().includes(query.value.trim().toLocaleLowerCase())));
const visible=computed(() => {
  const rows=matches.value.slice(0,100);
  if(!query.value.trim() && selected.value && !rows.includes(selected.value)) rows.splice(99,1,selected.value);
  return rows;
});
watch(query, () => { active.value=0; });
watch(() => props.disabled, value => { if(value) close(false); });
async function show() {
  if(props.disabled) return;
  query.value=''; open.value=true; active.value=Math.max(0,visible.value.findIndex(item => item.value === props.modelValue));
  const rect=trigger.value.getBoundingClientRect();
  const below=window.innerHeight-rect.bottom, above=rect.top;
  const width=Math.min(Math.max(rect.width,props.searchable ? 260 : 190),window.innerWidth-16);
  position.value={ left:`${Math.max(8, Math.min(rect.left,window.innerWidth-width-8))}px`, width:`${width}px`, maxHeight:`${Math.max(140,Math.min(380,Math.max(below,above)-16))}px`, ...(below >= 280 || below >= above ? {top:`${rect.bottom+6}px`} : {bottom:`${window.innerHeight-rect.top+6}px`}) };
  document.addEventListener('pointerdown', outside); window.addEventListener('resize', resize); document.addEventListener('scroll', scroll, true);
  await nextTick(); if (!open.value) return; (props.searchable ? search.value : list.value)?.focus(); scrollActive();
}
function toggle() { open.value ? close() : show(); }
function close(focus=true) { open.value=false; document.removeEventListener('pointerdown',outside); window.removeEventListener('resize',resize); document.removeEventListener('scroll',scroll,true); if(focus) trigger.value?.focus(); }
function outside(event) { if(!trigger.value?.contains(event.target) && !panel.value?.contains(event.target)) close(false); }
function resize() { close(false); }
function scroll(event) { if(!panel.value?.contains(event.target)) close(false); }
function choose(option) { emit('update:modelValue',option.value); emit('change',{target:{value:option.value}}); close(); }
function scrollActive() { panel.value?.querySelector('.highlighted')?.scrollIntoView?.({block:'nearest'}); }
async function navigate(event) {
  if(event.isComposing) return;
  if(event.key==='ArrowDown' || event.key==='ArrowUp') { event.preventDefault(); active.value=Math.max(0,Math.min(visible.value.length-1,active.value+(event.key==='ArrowDown'?1:-1))); await nextTick(); scrollActive(); }
  else if(!props.searchable && (event.key==='Home' || event.key==='End')) { event.preventDefault(); active.value=event.key==='Home' ? 0 : Math.max(0,visible.value.length-1); await nextTick(); scrollActive(); }
  else if(event.key==='Enter') { event.preventDefault(); if(visible.value[active.value]) choose(visible.value[active.value]); }
}
onUnmounted(() => close(false));
</script>
<style>
.searchable-select { box-sizing:border-box; display:flex; align-items:center; justify-content:space-between; gap:12px; width:100%; min-width:0; min-height:36px; padding:7px 12px; border:1px solid var(--control-border,var(--line,#ddd)); border-radius:10px; background:var(--surface,#fff); color:var(--text,#222); font:inherit; font-size:13px; line-height:20px; text-align:left; cursor:pointer; transition:background-color .15s,border-color .15s,box-shadow .15s; }
.searchable-select > span:first-child { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.select-chevron { flex-shrink:0; width:13px; height:13px; fill:none; stroke:var(--muted,#777); stroke-width:1.5; stroke-linecap:round; stroke-linejoin:round; transition:transform .15s; }
.searchable-select[aria-expanded=true] .select-chevron { transform:rotate(180deg); }
.searchable-select:hover:not(:disabled),.searchable-select[aria-expanded=true] { border-color:var(--line-strong,#ccc); background:var(--hover,#f5f5f5); }
.searchable-select:disabled { cursor:default; opacity:.45; }
.searchable-select:focus-visible { outline:2px solid var(--focus,#3478f6); outline-offset:2px; }
.select-popup { position:fixed; z-index:2300; box-sizing:border-box; display:flex; flex-direction:column; gap:6px; padding:6px; border:1px solid var(--control-border,var(--line,#ddd)); background:var(--popover,var(--surface,#fff)); color:var(--text,#222); border-radius:12px; box-shadow:var(--shadow-menu,0 8px 28px #0002); font:13px/1.5 var(--font-ui,system-ui); }
.select-search { display:flex; align-items:center; gap:8px; margin:2px 2px 4px; padding:0 10px; background:var(--surface-secondary,#f5f5f5); border:1px solid transparent; border-radius:8px; flex-shrink:0; }
.select-search:focus-within { border-color:var(--line-strong,#ccc); }
.select-search svg { width:16px; height:16px; flex-shrink:0; fill:none; stroke:var(--muted,#777); stroke-width:1.4; stroke-linecap:round; }
.select-search input { width:100%; min-width:0; box-sizing:border-box; padding:8px 0; border:0; background:transparent; color:inherit; font:inherit; outline:none; box-shadow:none; }
.select-results { overflow:auto; min-height:0; overscroll-behavior:contain; border-radius:8px; }
.select-results:focus-visible { outline:none; }
.select-results button { display:flex; align-items:center; justify-content:space-between; gap:16px; width:100%; min-height:36px; padding:8px 10px; border:0; border-radius:7px; text-align:left; background:transparent; color:inherit; font:inherit; cursor:pointer; }
.select-results button > span { min-width:0; overflow-wrap:anywhere; }
.select-results button.highlighted,.select-results button:hover { background:var(--hover,#eee); }
.select-results:focus-visible button.highlighted { box-shadow:inset 0 0 0 1px var(--line-strong,#ccc); }
.select-results button[aria-selected=true] { font-weight:500; }
.select-check { width:16px; height:16px; flex-shrink:0; fill:none; stroke:currentColor; stroke-width:1.6; stroke-linecap:round; stroke-linejoin:round; }
.select-popup small,.select-popup p { padding:4px 8px; font-size:12px; color:var(--muted,#777); }
.select-clear { border:0; border-radius:6px; background:transparent; color:var(--muted,#777); align-self:flex-start; padding:5px 10px; font:inherit; cursor:pointer; }
.select-clear:hover { background:var(--hover,#eee); color:var(--text,#222); }
@media(prefers-reduced-motion:reduce) { .searchable-select,.select-chevron { transition:none; } }
</style>
