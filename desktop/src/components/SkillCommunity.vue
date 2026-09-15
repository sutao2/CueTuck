<template>
 <section class="community-skills">
  <div v-if="!selected" class="community-toolbar">
   <form class="community-search skills-search" @submit.prevent="search"><AppIcon name="search"/><input v-model="query" :disabled="busy" aria-label="搜索社区 Skills" :placeholder="mine?'搜索我的发布…':'搜索 Skill 名称或描述…'"/><button type="submit" :disabled="busy" aria-label="搜索" title="搜索 · Enter">↵</button></form>
   <button class="button community-refresh" :disabled="busy" aria-label="刷新" title="刷新" @click="load"><AppIcon name="refresh"/></button>
  </div>
  <p v-if="error" role="alert" class="skills-alert">{{ error }} <button class="button" :disabled="busy" @click="retry">重试</button></p>
  <template v-if="selected">
   <button class="button" :disabled="busy" @click="selected=null">← 返回列表</button><h2>{{ selected.title }}</h2><p>{{ selected.description }}</p>
   <p class="muted">发布者：{{ selected.publisher?.display_name }} · {{ skillStatus(selected.status) }} · 许可：{{ selected.license }}</p>
   <p v-if="selected.reason" class="skills-alert">{{ selected.reason }}</p><pre class="community-body">{{ selected.body }}</pre>
   <h3>完整文件包 · {{ selected.files.length }} 个文件</h3><ul><li v-for="file in selected.files" :key="file.path">{{ file.path }} · {{ file.size }} B{{ file.executable?' · 可执行文件':'' }}</li></ul>
   <p class="muted">版本校验：{{ selected.digest }}</p><p class="muted">安装只写入文件，不会自动配置 MCP、安装运行时或执行脚本。</p>
   <div class="skills-actions"><button v-if="!mine" class="button primary-button" :disabled="busy" @click="install">{{ busy?'正在处理…':'安装到智能体…' }}</button><button v-if="mine&&selected.status!=='withdrawn'" class="button" :disabled="busy" @click="withdrawing=true">撤回发布</button></div>
   <div v-if="withdrawing" class="skills-panel"><p>撤回后其他用户将无法从广场下载，已安装副本会保留。</p><button class="button" :disabled="busy" @click="withdraw">确认撤回</button><button class="button" :disabled="busy" @click="withdrawing=false">取消</button></div>
  </template>
  <template v-else>
   <div class="community-results"><span>{{ mine?'我的发布':'社区作品' }}<small v-if="mine">审核通过后公开展示</small></span><span class="muted" role="status">{{ busy?'正在加载…':`共 ${total} 个 Skill` }}</span></div>
   <section class="skills-list"><button v-for="item in items" :key="item.id" class="skills-list-row" :disabled="busy" @click="open(item.id)"><span class="skills-list-icon"><AppIcon name="skills"/></span><span class="skills-row-main"><strong>{{ item.title }}</strong><span>{{ item.description }}</span><small>{{ skillCategoryName(item.category) }} · {{ item.publisher?.display_name }} · {{ item.license }}</small><span v-if="mine&&item.reason">{{ item.reason }}</span></span><span class="skills-badge">{{ mine?skillStatus(item.status):'查看并安装' }}</span><span>›</span></button></section>
   <div v-if="!busy&&!items.length&&!error" class="community-empty">
    <span class="community-empty-icon"><AppIcon :name="query.trim()||category?'search':mine?'file':'skills'"/></span>
    <h2>{{ query.trim()||category?'没有找到匹配的 Skill':mine?'你还没有发布 Skill':'社区还没有公开的 Skill' }}</h2>
    <p>{{ query.trim()||category?'试试其他关键词，或在左侧切换分类。':mine?'在右上角选择「发布到社区」，提交后可在这里查看审核进度。':'已有开源 Skill 可在 GitHub 来源中浏览和安装。你也可以创建并发布自己的作品。' }}</p>
    <button v-if="!mine&&!query.trim()&&!category" class="button" @click="emit('browse-github')">浏览 GitHub 来源 <span aria-hidden="true">→</span></button>
   </div>
   <div v-if="total>24&&!error" class="skills-pagination"><button :disabled="busy||offset===0" @click="offset-=24;load()">上一页</button><span>第 {{ Math.floor(offset/24)+1 }} / {{ Math.ceil(total/24) }} 页</span><button :disabled="busy||offset+24>=total" @click="offset+=24;load()">下一页</button></div>
  </template>
 </section>
</template>
<script setup>
import {computed,ref,watch,onMounted,onUnmounted} from 'vue';
import {skillMarket,skillStatus} from '../platform/skillMarket.js';
import {skillCategoryName,skillCategoryCounts} from '../platform/skillCategories.js';
import AppIcon from './AppIcon.vue';
const props=defineProps({category:{type:String,default:''},mine:{type:Boolean,default:false}}),emit=defineEmits(['install','busy','categories','browse-github']);
const mine=computed(()=>props.mine),items=ref([]),total=ref(0),offset=ref(0),query=ref(''),busy=ref(false),error=ref(''),selected=ref(null),withdrawing=ref(false);
let generation=0,disposed=false,last=()=>load();
async function run(task){if(busy.value)return;last=()=>run(task);const current=++generation;busy.value=true;error.value='';try{const result=await task();return result;}catch(e){if(!disposed&&current===generation)error.value=String(e.message||e);}finally{if(current===generation)busy.value=false;}}
async function load(){await run(async()=>{const result=await skillMarket(mine.value?'mine':'browse',{query:{q:query.value,category:props.category,offset:offset.value}});if(disposed)return;items.value=result.items;total.value=result.total;emit('categories',{ready:true,counts:{...skillCategoryCounts([]),...result.category_counts,'':Object.values(result.category_counts||{}).reduce((a,b)=>a+b,0)},scope:mine.value?'我的社区发布':'社区 Skills'});});}
async function open(id){await run(async()=>{selected.value=await skillMarket('detail',{id});withdrawing.value=false;});}
async function install(){await run(async()=>{const result=await skillMarket('bundle',{id:selected.value.id});if(result.digest!==selected.value.digest)throw new Error('发布版本已变化，请返回列表重新加载');emit('install',{...result,item:selected.value});});}
async function withdraw(){const id=selected.value.id;const result=await run(()=>skillMarket('withdraw',{id,body:{status:'withdrawn',expected_status:selected.value.status,reason:'作者撤回'}}));if(result){selected.value=null;withdrawing.value=false;await load();}}
function search(){offset.value=0;load();}function retry(){last();}
watch(()=>props.category,()=>{selected.value=null;offset.value=0;load();});watch(busy,v=>emit('busy',v),{flush:'sync'});
onMounted(load);onUnmounted(()=>{disposed=true;generation++;emit('busy',false);});
defineExpose({reload:load});
</script>
<style scoped src="./skills.css"></style>
<style scoped>
.community-skills{margin-top:20px}
.community-toolbar{display:flex;align-items:center;gap:10px}
.community-search{margin:0;flex:1;min-width:0;height:34px;box-sizing:border-box}
.community-search input{background:transparent}
.community-search button{border:0;border-radius:4px;background:var(--sidebar);color:var(--muted);font-size:15px;width:26px;height:24px;flex-shrink:0;cursor:pointer}
.community-search:focus-within{border-color:var(--muted)}
.community-toolbar>.button{height:34px;white-space:nowrap;display:inline-flex;align-items:center;gap:7px}
.community-refresh{width:34px;justify-content:center;padding:0}
.community-results{display:flex;justify-content:space-between;align-items:center;gap:12px;margin:24px 0 12px;font-size:12px;color:var(--text)}
.community-results small{margin-left:12px;color:var(--muted);font-size:11px}
.community-empty{display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;box-sizing:border-box;min-height:280px;padding:40px 24px;border:1px solid var(--line);border-radius:12px;background:var(--surface)}
.community-empty-icon{display:grid;place-items:center;width:48px;height:48px;border:1px solid var(--line);border-radius:12px;background:var(--sidebar);color:var(--muted);margin-bottom:16px}
.community-empty-icon .app-icon{width:24px;height:24px}
.community-empty h2{font-size:17px;font-weight:600;color:var(--text);margin:0 0 10px}
.community-empty p{max-width:370px;font-size:12px;line-height:1.8;color:var(--muted);margin:0}
.community-empty .button{margin-top:22px;display:flex;align-items:center;gap:16px}
.community-body{white-space:pre-wrap;overflow-wrap:anywhere;max-height:50vh;overflow:auto;padding:20px;background:var(--sidebar);border-radius:10px;font:12px/1.8 var(--font-code)}
.community-skills ul{font:12px/1.8 var(--font-code);overflow-wrap:anywhere}
@media(max-width:850px){.community-toolbar{flex-wrap:wrap}.community-search{flex-basis:100%;max-width:none}.community-results small{display:none}}
</style>
