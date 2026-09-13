<template>
  <Teleport to="body">
    <div class="image-backdrop" @click.self="$emit('close')">
    <section v-dialog-focus="() => $emit('close')" class="image-viewer" role="dialog" aria-modal="true" :aria-label="`图片预览：${title}`">
      <header><strong>{{ title }}</strong><div><button type="button" :disabled="!loaded" @click="zoom = 0">适应窗口</button><button type="button" :disabled="!loaded" @click="zoom = 1">100%</button><button type="button" aria-label="缩小图片" :disabled="!loaded" @click="resize(-.25)">−</button><span>{{ zoom ? `${Math.round(zoom * 100)}%` : '适应' }}</span><button type="button" aria-label="放大图片" :disabled="!loaded" @click="resize(.25)">＋</button><button type="button" data-dialog-autofocus aria-label="关闭图片预览" @click="$emit('close')">关闭 <kbd>Esc</kbd></button></div></header>
      <div class="image-canvas"><div class="image-surface" :class="{ fit: !zoom }">
        <p v-if="failed" role="alert">图片无法加载，请关闭后重试。</p>
        <img v-else :src="src" :alt="title" referrerpolicy="no-referrer" :style="zoom ? {width: `${naturalWidth * zoom}px`} : {}" @load="onLoad" @error="failed = true">
      </div></div>
    </section>
    </div>
  </Teleport>
</template>
<script setup>
import { ref, watch } from 'vue';
import { vDialogFocus } from '../lib/dialogFocus.js';
const props=defineProps({src:{type:String,required:true},title:{type:String,default:'图片'}});
defineEmits(['close']);
const zoom=ref(0),naturalWidth=ref(0),loaded=ref(false),failed=ref(false);
function onLoad(event){naturalWidth.value=event.target.naturalWidth;loaded.value=true;}
function resize(delta){zoom.value=Math.min(4,Math.max(.25,(zoom.value||1)+delta));}
watch(()=>props.src,()=>{zoom.value=0;loaded.value=false;failed.value=false;});
</script>
<style scoped>
.image-backdrop{position:fixed;inset:0;z-index:2000;background:#0008;display:grid;place-items:center;padding:24px}.image-viewer{width:min(1100px,100%);height:min(800px,90dvh);min-width:0;overflow:hidden;border:1px solid var(--line,#ddd);border-radius:16px;box-shadow:0 20px 70px #0004;display:flex;flex-direction:column;background:var(--surface,#fff);color:var(--text,#242424)}
header{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:16px 24px;border-bottom:1px solid var(--line,#ddd);flex-wrap:wrap}header strong{font-size:14px;overflow-wrap:anywhere}header>div{display:flex;align-items:center;gap:8px;flex-wrap:wrap}button{background:transparent;border:1px solid var(--line,#ddd);border-radius:7px;padding:7px 10px;color:inherit;font:inherit}header span{min-width:42px;text-align:center;font-size:12px}
.image-canvas{overflow:auto;flex:1;min-height:0;background:var(--sidebar,#f5f5f5)}.image-surface{min-width:100%;min-height:100%;width:max-content;display:flex;align-items:center;justify-content:center}.image-surface.fit{width:100%;height:100%;padding:20px}.image-surface img{display:block;max-width:none;flex-shrink:0;height:auto}.image-surface.fit img{max-width:100%;max-height:100%;object-fit:contain}
@media(max-width:600px){header{padding:12px;gap:10px}header strong{max-width:100%}.image-surface.fit{padding:8px}}
</style>
