<template>
  <button ref="cover" type="button" class="local-prompt-cover square-reference-cover" :aria-label="`查看 ${title} 的图片`" :disabled="!asset || failed" @click.stop="preview = true">
    <img v-if="asset && !failed" :src="assetUrl(asset)" alt="" decoding="async" @error="failed = true">
    <span v-else>{{ failed ? '图片暂不可用' : '图片' }}</span>
  </button>
  <ImageViewer v-if="preview && asset" :src="assetUrl(asset)" :title="title" @close="preview = false" />
</template>
<script setup>
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { firstPromptImage, assetUrl } from '../platform/assets.js';
import ImageViewer from './ImageViewer.vue';
const props = defineProps({ promptId: {type:String,required:true}, title:String, revision:[String,Number] });
const cover=ref(null),asset=ref(null),failed=ref(false),preview=ref(false),visible=ref(false);
let observer,request=0;
async function load() {
  const current=++request;asset.value=null;failed.value=false;preview.value=false;
  if (!visible.value) return;
  try {
    const result=await firstPromptImage(props.promptId);
    if (current===request) {asset.value=result;failed.value=!result;}
  } catch { if(current===request) failed.value=true; }
}
watch([()=>props.promptId,()=>props.revision,visible],load);
onMounted(()=>{
  if (!globalThis.IntersectionObserver) {visible.value=true;return;}
  observer=new IntersectionObserver(entries=>{visible.value=entries.some(entry=>entry.isIntersecting);});
  observer.observe(cover.value);
});
onBeforeUnmount(()=>{request++;observer?.disconnect();});
</script>
<style scoped>
.local-prompt-cover{display:block;padding:0;border:0;overflow:hidden;color:var(--muted);cursor:zoom-in;flex-shrink:0}
.local-prompt-cover:disabled{opacity:1;cursor:default}
.local-prompt-cover img{display:block;width:100%;height:100%;object-fit:cover}
.local-prompt-cover span{display:grid;place-items:center;width:100%;height:100%;font-size:12px}
</style>
