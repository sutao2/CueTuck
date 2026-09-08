<template>
  <section v-if="site" class="site-notice" aria-label="社区站点信息">
    <div class="site-identity"><img v-if="safeLogo&&!imageFailed" :src="safeLogo" alt="" referrerpolicy="no-referrer" @error="imageFailed=true"><div><component :is="heading ? 'h1' : 'strong'">{{ site.name }}</component><p v-if="site.description">{{ site.description }}</p></div><a v-if="site.support_email" :href="`mailto:${site.support_email}`">联系支持</a></div>
    <p v-if="activeAnnouncement" class="site-announcement" role="status">{{ site.announcement }}</p><p v-if="site.publishing_open===false" class="site-announcement">社区暂时关闭新投稿；本地编辑和使用不受影响。</p>
  </section>
</template>
<script setup>
import {computed,onMounted,onUnmounted,ref,watch} from 'vue';
const props=defineProps({site:{type:Object,default:null},heading:Boolean});const now=ref(Date.now()),imageFailed=ref(false);let timer;
const safeLogo=computed(()=>{try{const url=new URL(props.site?.logo_url);return url.protocol==='https:'&&!url.username&&!url.password?url.href:'';}catch{return '';}});
const activeAnnouncement=computed(()=>!!props.site?.announcement&&(!props.site.announcement_start||Date.parse(props.site.announcement_start)<=now.value)&&(!props.site.announcement_end||Date.parse(props.site.announcement_end)>now.value));
watch(safeLogo,()=>imageFailed.value=false);onMounted(()=>timer=setInterval(()=>now.value=Date.now(),1000));onUnmounted(()=>clearInterval(timer));
</script>
<style scoped>
.site-notice{padding:16px 24px;border-bottom:1px solid var(--border-color,#e8eaec);font-size:13px}.site-identity{display:flex;align-items:center;gap:12px;min-width:0}.site-identity img{width:34px;height:34px;border-radius:8px;object-fit:contain}.site-identity strong{font-size:14px}.site-identity p{margin:4px 0 0;color:var(--text-secondary,#7a838a);line-height:1.5}.site-identity a{margin-left:auto;color:inherit;white-space:nowrap}.site-announcement{white-space:pre-wrap;overflow-wrap:anywhere;background:var(--surface-muted,#f5f6f7);border-radius:8px;padding:12px;margin:12px 0 0;line-height:1.6}@media(max-width:600px){.site-identity{flex-wrap:wrap}.site-identity a{margin-left:0}}
</style>
