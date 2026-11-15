<template>
  <main class="content-area clips-page">
    <header class="clips-heading"><div><h1>{{ tr('常用片段') }}</h1><p>{{ tr('只保存你主动添加的内容。用启动器搜索，按回车即可复制。') }}</p></div>
      <button v-if="!draft" class="button primary-button" @click="edit()">{{ tr('添加片段') }}</button>
    </header>
    <p v-if="error" role="alert">{{ tr(error) }}</p><p v-if="notice" role="status">{{ tr(notice) }}</p>
    <form v-if="draft" class="clip-form" @submit.prevent="save">
      <label>{{ tr('类型') }}<select v-model="draft.kind" :disabled="!!draft.id || busy" @change="draft.value=''"><option v-for="(name,kind) in clipTypes" :value="kind" :key="kind">{{ tr(name) }}</option></select></label>
      <label>{{ tr('名称') }}<input ref="titleInput" v-model="draft.title" required maxlength="120" :disabled="busy" autocomplete="off"></label>
      <label v-if="draft.kind==='password'">{{ tr('密码') }}<input type="password" v-model="draft.value" :required="!draft.id" :disabled="busy" autocomplete="new-password"></label>
      <label v-else>{{ tr('内容') }}<textarea v-model="draft.value" required :disabled="busy" rows="5" /></label>
      <p v-if="draft.kind==='password'">{{ tr('密码存入系统凭据库，仅按名称搜索。编辑时留空保留原密码。') }}</p>
      <div><button class="button primary-button" :disabled="busy">{{ tr('保存') }}</button> <button type="button" class="button" :disabled="busy" @click="draft=null;error=''">{{ tr('取消') }}</button></div>
    </form>
    <template v-else>
      <input ref="searchInput" type="search" v-model="query" :placeholder="tr('搜索名称或片段内容')" :aria-label="tr('搜索常用片段')">
      <p v-if="!rows.length && !error">{{ query ? tr('没有匹配的片段') : tr('把常用网址、账号或模板保存在这里，下次无需再找。') }}</p>
      <article v-for="row in rows" :key="row.id" class="clip-row" @focusout="leaveRow">
        <div><small>{{ tr(clipTypes[row.kind]) }}</small><strong>{{ row.title }}</strong><p :class="{'visible-secret':visibleId===row.id}">{{ row.kind==='password' ? (visibleId===row.id ? visibleValue : '••••••••') : row.content }}</p></div>
        <div class="clip-actions"><template v-if="removing===row.id"><button class="button" :disabled="busy" @click="remove(row)">{{ tr('确认删除') }}</button><button class="button" :disabled="busy" @click="removing=''">{{ tr('取消') }}</button></template>
          <template v-else><button v-if="row.kind==='password'" class="button" :aria-pressed="visibleId===row.id" :disabled="busy" @click="togglePassword(row)">{{ tr(visibleId===row.id ? '隐藏密码' : '显示密码') }}</button><button class="button" :disabled="busy" @click="copy(row)">{{ tr('复制') }}</button><button class="button" :disabled="busy" @click="edit(row)">{{ tr('编辑') }}</button><button class="button" :disabled="busy" @click="hidePassword();removing=row.id">{{ tr('删除') }}</button></template>
        </div>
      </article>
    </template>
  </main>
</template>
<script setup>
import {ref,watch,nextTick,onMounted,onUnmounted,computed} from 'vue';
import {clipTypes,clipRequest,findClips,copyClip} from '../platform/quickClips.js';
import {tr} from '../platform/interfaceLanguage.js';
const rows=ref([]),query=ref(''),draft=ref(null),busy=ref(false),error=ref(''),notice=ref(''),removing=ref(''),titleInput=ref(null),searchInput=ref(null);
const visibleId=ref(''),visibleValue=ref('');
let generation=0,revealSequence=0,hideTimer;
function hidePassword(){
  ++revealSequence;
  clearTimeout(hideTimer);
  visibleId.value='';visibleValue.value='';
}
function leaveRow(event){if(!event.currentTarget.contains(event.relatedTarget))hidePassword();}
function onVisibility(){if(document.hidden)hidePassword();}
async function togglePassword(row){
  if(busy.value)return;
  const wasVisible=visibleId.value===row.id;
  hidePassword();
  if(wasVisible)return;
  const token=revealSequence;
  error.value='';notice.value='';
  try{
    const value=await clipRequest({action:'copy_value',id:row.id});
    if(token!==revealSequence)return;
    visibleId.value=row.id;visibleValue.value=value;
    hideTimer=setTimeout(hidePassword,15000);
  }catch(e){if(token===revealSequence)error.value=String(e.message||e); }
}
onMounted(()=>{window.addEventListener('blur',hidePassword);document.addEventListener('visibilitychange',onVisibility);});
onUnmounted(()=>{hidePassword();window.removeEventListener('blur',hidePassword);document.removeEventListener('visibilitychange',onVisibility);});
async function load(){
  const token=++generation;error.value='';
  try{const result=await findClips(query.value);if(token===generation)rows.value=result;}
  catch(e){if(token===generation){rows.value=[];error.value=String(e.message||e);}}
}
watch(query,()=>{hidePassword();notice.value='';removing.value='';load();},{immediate:true});
function edit(row){hidePassword();notice.value='';error.value='';removing.value='';draft.value={id:row?.id||null,kind:row?.kind||'text',title:row?.title||'',value:row?.content||''};nextTick(()=>titleInput.value?.focus());}
async function act(fn){if(busy.value)return;busy.value=true;error.value='';notice.value='';try{await fn();}catch(e){error.value=String(e.message||e);}finally{busy.value=false;}}
function save(){return act(async()=>{const d=draft.value;await clipRequest({action:'save',...d,value:d.kind==='password'&&d.id&&!d.value?null:d.value});draft.value=null;query.value='';await load();notice.value='已保存';});}
function copy(row){return act(async()=>{await copyClip(row.id);notice.value='已复制';});}
function remove(row){return act(async()=>{await clipRequest({action:'delete',id:row.id});removing.value='';await load();notice.value='已删除';});}
defineExpose({editing:computed(()=>Boolean(draft.value)),focusSearch:()=>searchInput.value?.focus()});
</script>
<style scoped>
.clips-page{padding:24px 32px;overflow:auto}.clips-heading,.clip-row,.clip-actions{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap}.clips-page p,.clips-page small{color:var(--muted)}.clip-row{padding:14px 0;border-bottom:1px solid var(--line)}.clip-row>div:first-child{min-width:0;flex:1}.clip-row strong,.clip-row p{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;margin:4px 0}.clip-form,.clip-form label{display:grid;gap:8px}.clip-form{max-width:640px;gap:16px}.clips-page :is(input,select,textarea){box-sizing:border-box;border:1px solid var(--control-border);border-radius:var(--radius-control);background:var(--surface);color:var(--text);font:inherit;padding:10px;width:100%}
.clip-row p.visible-secret{white-space:pre-wrap;overflow-wrap:anywhere;font-family:var(--font-code);color:var(--text)}
</style>
