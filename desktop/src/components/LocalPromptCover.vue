<template>
  <button ref="cover" type="button" class="local-prompt-cover square-reference-cover" :aria-label="`查看 ${title} 的图片`" :disabled="opening" @click.stop="openOriginal">
    <img v-if="asset && !failed" :src="assetUrl(asset)" alt="" decoding="async" @error="failed = true">
    <span v-else>{{ opening ? '正在读取原图…' : failed ? '预览暂不可用，点击查看原图' : '图片' }}</span>
  </button>
  <p v-if="originalError" class="cover-error" role="alert">{{ originalError }}</p>
  <ImageViewer v-if="original" :src="assetUrl(original)" :title="title" @close="original = null" />
</template>
<script setup>
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { firstPromptImage, assetUrl } from '../platform/assets.js';
import ImageViewer from './ImageViewer.vue';
import { cachedPromptThumbnail } from '../platform/thumbnailCache.js';
const props = defineProps({ promptId: {type:String,required:true}, title:String, revision:[String,Number] });
const cover=ref(null),asset=ref(null),failed=ref(false),visible=ref(false),original=ref(null),opening=ref(false),originalError=ref('');
let observer,request=0,originalRequest=0;
async function load() {
  const current=++request;asset.value=null;failed.value=false;
  if (!visible.value) return;
  try {
    const result=await cachedPromptThumbnail(props.promptId, props.revision);
    if (current===request) {asset.value=result;failed.value=!result;}
  } catch { if(current===request) failed.value=true; }
}
watch([()=>props.promptId,()=>props.revision],()=>{
  originalRequest++; original.value=null; opening.value=false; originalError.value=''; load();
});
watch(visible,load);
async function openOriginal() {
  if (opening.value) return;
  const current=++originalRequest; opening.value=true; originalError.value='';
  try {
    const result=await firstPromptImage(props.promptId);
    if (current===originalRequest) {
      original.value=result;
      if (!result) originalError.value='原图不存在';
    }
  } catch { if (current===originalRequest) originalError.value='原图读取失败，请点击图片重试'; }
  finally { if (current===originalRequest) opening.value=false; }
}
onMounted(()=>{
  if (!globalThis.IntersectionObserver) {visible.value=true;return;}
  observer=new IntersectionObserver(entries=>{visible.value=entries.some(entry=>entry.isIntersecting);});
  observer.observe(cover.value);
});
onBeforeUnmount(()=>{request++;originalRequest++;observer?.disconnect();});
</script>
<style scoped>
.cover-error{font-size:12px;color:var(--danger)}
.local-prompt-cover{display:block;padding:0;border:0;overflow:hidden;color:var(--muted);cursor:zoom-in;flex-shrink:0}
.local-prompt-cover:disabled{opacity:1;cursor:default}
.local-prompt-cover img{display:block;width:100%;height:100%;object-fit:cover}
.local-prompt-cover span{display:grid;place-items:center;width:100%;height:100%;font-size:12px}
</style>
