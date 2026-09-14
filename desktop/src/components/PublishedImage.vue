<template>
  <button ref="element" v-bind="$attrs" type="button" class="published-image" :aria-label="`查看 ${title || file.name} 的图片`" @click.stop="open">
    <img v-if="asset && !failed" :src="assetUrl(asset)" :alt="title || file.name" decoding="async" @error="failed = true">
    <span v-else>{{ failed ? '图片加载失败，点击重试' : '正在加载图片…' }}</span>
  </button>
  <ImageViewer v-if="viewing && asset" :src="assetUrl(asset)" :title="title || file.name" @close="viewing = false" />
</template>
<script setup>
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { assetUrl } from '../platform/assets.js';
import { getSession } from '../platform/session.js';
import { publishedImage } from '../platform/publishedImageCache.js';
import ImageViewer from './ImageViewer.vue';
defineOptions({ inheritAttrs: false });
const props = defineProps({ itemId: {type:String,required:true}, file: {type:Object,required:true}, title:String });
const element=ref(null), asset=ref(null), failed=ref(false), viewing=ref(false), visible=ref(false);
let observer, request=0;
async function load() {
  const current=++request; asset.value=null; failed.value=false;
  if (!visible.value) return;
  try { const value=await publishedImage(props.itemId, props.file, getSession().accessToken); if(current===request) asset.value=value; }
  catch { if(current===request) failed.value=true; }
}
async function open() { if(failed.value) await load(); if(asset.value) viewing.value=true; }
watch([()=>props.itemId,()=>props.file], () => { viewing.value=false; load(); });
watch(visible, () => { if(visible.value && !asset.value) load(); });
onMounted(() => {
  if(!globalThis.IntersectionObserver) { visible.value=true; return; }
  observer=new IntersectionObserver(entries => { visible.value=entries.some(entry => entry.isIntersecting); }); observer.observe(element.value);
});
onBeforeUnmount(() => { request++; observer?.disconnect(); });
</script>
<style scoped>
.published-image { display:block; padding:0; border:0; background:var(--sidebar); color:var(--muted); overflow:hidden; cursor:zoom-in; min-width:0; }
.published-image img { display:block; width:100%; height:100%; object-fit:cover; }
.published-image span { display:grid; place-items:center; width:100%; height:100%; min-height:0; font-size:12px; }
</style>
