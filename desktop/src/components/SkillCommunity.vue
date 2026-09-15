<template>
 <section class="community-skills">
  <div class="skills-actions"><button class="button" :class="{active:!mine}" :disabled="busy" @click="switchMode(false)">全部作品</button><button class="button" :class="{active:mine}" :disabled="busy" @click="switchMode(true)">我的发布</button><button class="button" :disabled="busy" @click="load">刷新</button></div>
  <p class="muted">{{ mine?'查看审核状态、退回原因或撤回发布；更新内容请重新投稿。':'用户发布并经人工审核的完整 Skill 文件包。查看说明与文件后，安装到指定智能体。' }}</p>
  <p v-if="error" role="alert" class="skills-alert">{{ error }} <button class="button" :disabled="busy" @click="retry">重试</button></p>
  <template v-if="selected">
   <button class="button" :disabled="busy" @click="selected=null">← 返回列表</button><h2>{{ selected.title }}</h2><p>{{ selected.description }}</p>
   <p class="muted">发布者：{{ selected.publisher?.display_name }} · {{ skillStatus(selected.status) }} · 许可：{{ selected.license }}</p>
   <p v-if="selected.reason" class="skills-alert">{{ selected.reason }}</p><pre class="community-body">{{ selected.body }}</pre>
   <h3>完整文件包 · {{ selected.files.length }} 个文件</h3><ul><li v-for="file in selected.files" :key="file.path">{{ file.path }} · {{ file.size }} B{{ file.executable?' · 可执行文件':'' }}</li></ul>
   <p class="muted">版本校验：{{ selected.digest }}</p><p class="muted">安装只写入文件，不会自动配置 MCP、安装运行时或执行脚本。</p>
   <div class="skills-actions"><button class="button primary-button" :disabled="busy" @click="install">{{ busy?'正在处理…':'安装到智能体…' }}</button><button v-if="mine&&selected.status!=='withdrawn'" class="button" :disabled="busy" @click="withdrawing=true">撤回发布</button></div>
   <div v-if="withdrawing" class="skills-panel"><p>撤回后其他用户将无法从广场下载，已安装副本会保留。</p><button class="button" :disabled="busy" @click="withdraw">确认撤回</button><button class="button" :disabled="busy" @click="withdrawing=false">取消</button></div>
  </template>
  <template v-else>
   <label class="skills-search"><AppIcon name="search"/><input v-model="query" aria-label="搜索社区 Skills" placeholder="搜索名称或描述" @keydown.enter="search"/></label><button class="button" :disabled="busy" @click="search">搜索</button>
   <p class="muted" role="status">{{ busy?'正在加载…':`共 ${total} 个 Skill` }}</p>
   <section class="skills-list"><button v-for="item in items" :key="item.id" class="skills-list-row" :disabled="busy" @click="open(item.id)"><span class="skills-list-icon"><AppIcon name="skills"/></span><span class="skills-row-main"><strong>{{ item.title }}</strong><span>{{ item.description }}</span><small>{{ skillCategoryName(item.category) }} · {{ item.publisher?.display_name }} · {{ item.license }}</small><span v-if="mine&&item.reason">{{ item.reason }}</span></span><span class="skills-badge">{{ mine?skillStatus(item.status):'查看并安装' }}</span><span>›</span></button></section>
   <div v-if="!busy&&!items.length&&!error" class="skills-empty"><h2>{{ mine?'暂无发布记录':'还没有匹配的社区 Skill' }}</h2><p>可以创建自己的 Skill 发布，也可以切换到 GitHub 来源安装现有内容。</p></div>
   <div class="skills-pagination"><button :disabled="busy||offset===0" @click="offset-=24;load()">上一页</button><span>第 {{ Math.floor(offset/24)+1 }} 页</span><button :disabled="busy||offset+24>=total" @click="offset+=24;load()">下一页</button></div>
  </template>
 </section>
</template>
<script setup>
import {ref,watch,onMounted,onUnmounted} from 'vue';
import {skillMarket,skillStatus} from '../platform/skillMarket.js';
import {skillCategoryName,skillCategoryCounts} from '../platform/skillCategories.js';
import AppIcon from './AppIcon.vue';
const props=defineProps({category:{type:String,default:''}}),emit=defineEmits(['install','busy','categories']);
const mine=ref(false),items=ref([]),total=ref(0),offset=ref(0),query=ref(''),busy=ref(false),error=ref(''),selected=ref(null),withdrawing=ref(false);
let generation=0,disposed=false,last=()=>load();
async function run(task){if(busy.value)return;last=()=>run(task);const current=++generation;busy.value=true;error.value='';try{const result=await task();return result;}catch(e){if(!disposed&&current===generation)error.value=String(e.message||e);}finally{if(current===generation)busy.value=false;}}
async function load(){await run(async()=>{const result=await skillMarket(mine.value?'mine':'browse',{query:{q:query.value,category:props.category,offset:offset.value}});if(disposed)return;items.value=result.items;total.value=result.total;emit('categories',{ready:true,counts:{...skillCategoryCounts([]),...result.category_counts,'':Object.values(result.category_counts||{}).reduce((a,b)=>a+b,0)},scope:mine.value?'我的社区发布':'社区 Skills'});});}
async function open(id){await run(async()=>{selected.value=await skillMarket('detail',{id});withdrawing.value=false;});}
async function install(){await run(async()=>{const result=await skillMarket('bundle',{id:selected.value.id});if(result.digest!==selected.value.digest)throw new Error('发布版本已变化，请返回列表重新加载');emit('install',{...result,item:selected.value});});}
async function withdraw(){const id=selected.value.id;const result=await run(()=>skillMarket('withdraw',{id,body:{status:'withdrawn',expected_status:selected.value.status,reason:'作者撤回'}}));if(result){selected.value=null;withdrawing.value=false;await load();}}
function search(){offset.value=0;load();}function switchMode(value){mine.value=value;selected.value=null;offset.value=0;load();}function retry(){last();}
watch(()=>props.category,()=>{selected.value=null;offset.value=0;load();});watch(busy,v=>emit('busy',v),{flush:'sync'});
onMounted(load);onUnmounted(()=>{disposed=true;generation++;emit('busy',false);});
defineExpose({reload:load});
</script>
<style scoped>
.community-skills{margin-top:20px}.community-skills .skills-search{display:inline-flex;width:min(600px,80%);margin-right:10px}.community-body{white-space:pre-wrap;overflow-wrap:anywhere;max-height:50vh;overflow:auto;padding:20px;background:var(--sidebar);border-radius:10px;font:12px/1.8 var(--font-code)}.community-skills ul{font:12px/1.8 var(--font-code);overflow-wrap:anywhere}.active{background:var(--text);color:var(--surface)}
</style>

<style scoped src="./skills.css"></style>
