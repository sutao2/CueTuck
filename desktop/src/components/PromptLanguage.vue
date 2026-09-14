<template>
  <section class="prompt-language" aria-label="提示词语言版本">
    <div class="language-bar">
      <div class="language-tabs" role="group" aria-label="正文语言"><button v-for="tab in tabs" :key="tab.id" type="button" :aria-pressed="selected === tab.id" :disabled="disabled" @click="select(tab.id)">{{ tab.label }}<span v-if="tab.id !== 'original' && versions[tab.id]"> · {{ versions[tab.id].original ? '原稿' : '已就绪' }}</span></button></div>
      <label v-if="selected !== 'original' && current && !current.original"><input v-model="compare" type="checkbox">对照原文</label>
    </div>
    <p v-if="busy" role="status">{{ squareId ? '译文正在生成，可先查看原文…' : '正在使用本机配置的模型翻译…' }}</p>
    <p v-else-if="error" role="alert">{{ error }} <button type="button" class="button" @click="generate">重试</button></p>
    <p v-else-if="selected !== 'original' && !current" class="language-note">{{ squareId ? '此语言版本尚未生成。生成后大家都可复用。' : '仅将此模板发送到你配置的翻译模型，原文会保留。' }} <button class="button" type="button" :disabled="disabled" @click="generate">{{ selected === 'en' ? '生成英文版本' : '翻译成中文' }}</button></p>
    <p v-else-if="selected !== 'original' && current" class="language-note">{{ current.original ? '直接使用原稿，无需翻译。' : 'AI 译文 · 已保留原文与变量，使用前请核对。' }}</p>
    <pre v-if="compare && current && selected !== 'original'" class="language-original">{{ text }}</pre>
  </section>
</template>
<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { localVersion, squareVersions, translateOnce, sourceLanguage } from '../platform/translation.js';
const props = defineProps({text:{type:String,default:''},squareId:{type:String,default:''},memberIndex:{type:Number,default:-1},initialVersions:{type:Object,default:null},defaultLanguage:{type:String,default:'original'},disabled:Boolean});
const emit=defineEmits(['change']);
const tabs=[{id:'zh',label:'中文'},{id:'original',label:'原文'},{id:'en',label:'English'}];
const selected=ref(props.defaultLanguage),versions=ref({}),busy=ref(false),error=ref(''),compare=ref(false);
const current=computed(()=>versions.value[selected.value]);
let generation=0,timer;
function output(){emit('change',selected.value==='original'?props.text:current.value?.text||props.text);}
function acceptSquare(rows){for(const target of ['zh','en']){const entry=rows[target];if(entry?.status==='ready'){const source=props.memberIndex>=0?entry.version?.source?.members?.[props.memberIndex]?.content:entry.version?.source?.content;if(source!==undefined && source!==props.text)continue;const text=props.memberIndex>=0?entry.version?.members?.[props.memberIndex]?.content:entry.version?.content;if(typeof text==='string')versions.value[target]={text};}}}
async function load(){const run=++generation;clearTimeout(timer);busy.value=false;error.value='';versions.value={};selected.value=props.defaultLanguage;compare.value=false;
  const lang=sourceLanguage(props.text);if(lang)versions.value[lang]={text:props.text,original:true};output();
  try{if(props.squareId){const rows=props.initialVersions ?? await squareVersions(props.squareId);if(run!==generation)return;acceptSquare(rows);}else{for(const target of ['zh','en']){const cached=await localVersion(props.text,target);if(run!==generation)return;if(cached)versions.value[target]=cached;}}if(run===generation)output();}catch(e){/* Reading a cache must not block the original. Explicit generation reports errors. */}
}
function select(language){selected.value=language;error.value='';compare.value=false;output();}
async function poll(run,target,attempt=0){try{const rows=await squareVersions(props.squareId);if(run!==generation)return;acceptSquare(rows);if(versions.value[target]){busy.value=false;output();return;}const state=rows[target];if(state?.status==='failed')throw Error(state.error||'翻译失败，原文仍可用');if(attempt>=60)throw Error('译文仍在后台排队，稍后重新打开即可查看');timer=setTimeout(()=>poll(run,target,attempt+1),3000);}catch(e){if(run===generation){busy.value=false;error.value=String(e.message||e);}}}
async function generate(){if(busy.value||props.disabled||selected.value==='original')return;const run=generation,target=selected.value;busy.value=true;error.value='';try{if(props.squareId){const rows=await squareVersions(props.squareId,target);if(run!==generation)return;acceptSquare(rows);if(!versions.value[target]){poll(run,target);return;}}else{const result=await translateOnce(props.text,target);if(run!==generation)return;versions.value[target]=result;}output();}catch(e){if(run===generation)error.value=String(e.message||e);}finally{if(run===generation&&(!props.squareId||versions.value[target]||error.value))busy.value=false;}}
watch(()=>[props.text,props.squareId,props.memberIndex],load,{immediate:true});
onBeforeUnmount(()=>{generation++;clearTimeout(timer);});
</script>
<style scoped>
.prompt-language{margin:0 0 18px;min-width:0}.language-bar{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap}.language-tabs{display:flex;padding:3px;background:var(--sidebar);border:1px solid var(--line);border-radius:9px;gap:3px}.language-tabs button{border:0;background:transparent;color:var(--muted);font:inherit;font-size:12px;padding:7px 12px;border-radius:6px}.language-tabs button[aria-pressed=true]{background:var(--surface);color:var(--text);box-shadow:0 1px 3px #0001}.language-tabs span{font-size:10px}.language-bar label,.language-note,.prompt-language>p{font-size:12px;color:var(--muted);line-height:1.7}.language-original{max-height:260px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;border-left:2px solid var(--line);padding:12px 16px;color:var(--muted);font:inherit;font-size:13px;background:var(--sidebar);border-radius:0 8px 8px 0}
</style>
