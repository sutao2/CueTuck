<template>
 <section class="version-history">
  <header><h3>{{ tr("版本历史") }}</h3><button class="button" :disabled="busy" @click="emit('close')">{{ tr("关闭历史") }}</button></header>
  <p v-if="error" role="alert">{{ tr(error) }}</p><p v-if="notice" role="status">{{ tr(notice) }}</p>
  <select v-model="selected" :disabled="busy" :aria-label="tr('查看版本')" @change="read"><option v-for="v in history?.versions||[]" :key="v.version" :value="v.version">v{{ v.version }} · {{ v.change_note||tr('首次发布') }} · {{ new Date(v.published_at).toLocaleDateString() }}</option></select>
  <pre v-if="revision">{{ text }}</pre>
  <template v-if="author && revision && selected!==history.current_version"><p>{{ tr("恢复历史内容会提交一个新版本，审核通过后生效。") }}</p><label>{{ tr("更新说明") }}<textarea v-model="note" maxlength="2000" :disabled="busy" /></label><button class="button" :disabled="busy||!note.trim()" @click="restore">{{ tr("将此内容提交为新版本") }}</button></template>
 </section>
</template>
<script setup>
import { tr } from '../platform/interfaceLanguage.js';
import { computed, ref, onMounted, watch } from 'vue';
import { contentVersions } from '../platform/contentVersions.js';
import { createPublication } from '../platform/square.js';
import { skillMarket } from '../platform/skillMarket.js';
const props=defineProps({kind:{type:String,required:true},id:{type:String,required:true},sourceId:String,author:Boolean});
const emit=defineEmits(['close','busy','submitted']);
const busy=ref(false),error=ref(''),notice=ref(''),history=ref(null),revision=ref(null),selected=ref(null),note=ref('');
const text=computed(()=>{const s=revision.value?.snapshot;if(!s)return '';if(props.kind==='skill'){try {const raw=atob(s.bundle.files.find(f=>f.path==='SKILL.md')?.content||'');return `${s.title}\n\n${new TextDecoder().decode(Uint8Array.from(raw,c=>c.charCodeAt(0)))}\n\n文件：\n${s.bundle.files.map(f=>f.path).join('\n')}`;}catch{return '正文无法预览';}}return [s.title,s.content,...(s.members||[]).map(m=>`${m.title}\n${m.content}`)].filter(Boolean).join('\n\n');});
async function run(work){if(busy.value)return;busy.value=true;emit('busy',true);error.value='';try{await work();}catch(e){error.value=String(e.message||e);}finally{busy.value=false;emit('busy',false);}}
async function read(){await run(async()=>{revision.value=null;revision.value=await contentVersions(props.kind,props.id,Number(selected.value));});}
const requestId=ref(crypto.randomUUID());
watch([selected,note],()=>requestId.value=crypto.randomUUID());
async function restore(){await run(async()=>{const s=revision.value.snapshot;const change={content_id:props.id,base_version:history.value.current_version,change_note:note.value.trim(),request_id:requestId.value};if(props.kind==='skill'){await skillMarket('submit',{body:{request_id:change.request_id,title:s.title,description:s.description,category:s.category,license:s.license,confirm_public:true,bundle:s.bundle,revision:change}});}else{await createPublication({sourceId:props.sourceId||`history:${props.id}`,title:s.title,content:s.content,categoryId:s.category_id,model:s.model,kind:props.kind,members:s.members,assetRefs:s.asset_refs||[],cover:s.cover,revision:change});}notice.value='已提交新版本审核，当前公开版本保持不变。';note.value='';emit('submitted');});}
onMounted(()=>run(async()=>{history.value=await contentVersions(props.kind,props.id);selected.value=history.value.current_version;revision.value=await contentVersions(props.kind,props.id,selected.value);}));
</script>
<style scoped>
.version-history{padding:20px;border:1px solid var(--line);border-radius:14px;margin:16px 0}header{display:flex;justify-content:space-between;align-items:center;gap:16px}select,textarea{display:block;max-width:100%;padding:10px;border:1px solid var(--line);border-radius:8px;color:var(--text);background:var(--surface);margin:12px 0}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:inherit;line-height:1.7;max-height:440px;overflow:auto}p{color:var(--muted)}
</style>
