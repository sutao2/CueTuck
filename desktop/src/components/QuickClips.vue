<template>
  <main class="content-area clips-page"><div class="clips-inner" :class="{editing:draft}">
    <header class="clips-heading">
      <div><button v-if="draft" class="back" :disabled="busy" @click="cancelEdit">‹ {{ tr('常用片段') }}</button>
        <h1>{{ draft ? tr(draft.id ? '编辑片段' : '添加片段') : groupTitle }}</h1>
        <p v-if="!draft">{{ tr('共 {0} 个结果',[rows.length]) }}</p></div>
      <button v-if="!draft" class="button primary-button" @click="edit()"><AppIcon name="plus"/>{{ tr('添加片段') }}</button>
    </header>
    <QuickClipEditor v-if="draft" :initial="draft" :groups="summary.categories" @cancel="cancelEdit" @saved="saved" @busy="busy=$event"/>
    <template v-else>
      <div class="clip-toolbar">
        <label class="search"><AppIcon name="search"/><input ref="searchInput" type="search" v-model="query" :placeholder="tr('搜索名称或片段内容')" :aria-label="tr('搜索常用片段')"></label>
        <SearchableSelect v-if="compactNav" :model-value="category" :options="groupOptions" :aria-label="tr('片段分类')" @update:model-value="emit('select-category',$event)"/>
        <SearchableSelect v-model="kind" :options="typeOptions" :aria-label="tr('筛选片段类型')" :searchable="false"/>
      </div>
      <p v-if="error || summaryError" role="alert" class="error">{{ tr(error || summaryError) }}</p>
      <div v-if="!rows.length && !loading && !error" class="empty">
        <AppIcon :name="filtered ? 'search' : 'folder'"/>
        <strong>{{ tr(filtered ? '没有匹配的片段' : category!==null ? '这个分类还没有片段' : '还没有常用片段') }}</strong>
        <button v-if="filtered" class="button" @click="query='';kind=''">{{ tr('清除筛选') }}</button>
        <p v-else>{{ tr('添加常用网址、账号或模板，之后可用启动器搜索并复制。') }}</p>
      </div>
      <article v-for="row in rows" :key="row.id" class="clip-row" @focusout="leaveRow">
        <span class="kind-icon"><AppIcon :name="clipIcons[row.kind]"/></span>
        <div class="clip-copy"><strong>{{ row.title }}</strong><p :class="{'visible-secret':visibleId===row.id}">{{ row.kind==='password' ? (visibleId===row.id ? visibleValue : '••••••••') : row.content }}</p>
          <small>{{ tr(clipTypes[row.kind]) }}<template v-if="category===null && groupNames.has(row.category_id)"> · {{ groupNames.get(row.category_id) }}</template></small>
        </div>
        <div class="clip-actions">
          <template v-if="removing===row.id"><button class="button" :disabled="busy" @click="remove(row)">{{ tr('确认删除') }}</button><button class="quiet" :disabled="busy" @click="removing=''">{{ tr('取消') }}</button></template>
          <template v-else><button v-if="row.kind==='password'" class="quiet" :aria-pressed="visibleId===row.id" :disabled="busy" @click="togglePassword(row)">{{ tr(visibleId===row.id ? '隐藏密码' : '显示密码') }}</button><button class="button" :disabled="busy" @click="copy(row)">{{ tr('复制') }}</button><button class="quiet" :disabled="busy" @click="edit(row)">{{ tr('编辑') }}</button><button class="quiet" :disabled="busy" @click="hidePassword();removing=row.id">{{ tr('删除') }}</button></template>
        </div>
      </article>
      <p v-if="notice" role="status" class="notice">{{ tr(notice) }}</p>
    </template>
  </div></main>
</template>
<script setup>
import {ref,watch,onMounted,onUnmounted,computed} from 'vue';
import AppIcon from './AppIcon.vue';
import SearchableSelect from './SearchableSelect.vue';
import QuickClipEditor from './QuickClipEditor.vue';
import {clipTypes,clipIcons,clipRequest,clipCategories,findClips,copyClip} from '../platform/quickClips.js';
import {tr} from '../platform/interfaceLanguage.js';
const props=defineProps({category:{default:null},compactNav:Boolean});
const emit=defineEmits(['categories','select-category']);
const rows=ref([]),query=ref(''),kind=ref(''),draft=ref(null),busy=ref(false),loading=ref(true);
const error=ref(''),summaryError=ref(''),notice=ref(''),removing=ref(''),searchInput=ref(null);
const summary=ref({categories:[],total:0,uncategorized:0});
const groupNames=computed(()=>new Map(summary.value.categories.map(g=>[g.id,g.name])));
const groupTitle=computed(()=>props.category===null ? tr('常用片段') : groupNames.value.get(props.category) || tr('未分类'));
const groupOptions=computed(()=>[{value:null,label:tr('全部片段')},{value:'',label:tr('未分类')},...summary.value.categories.map(g=>({value:g.id,label:g.name}))]);
const typeOptions=computed(()=>[{value:'',label:tr('全部类型')},...Object.entries(clipTypes).map(([value,label])=>({value,label:tr(label)}))]);
const filtered=computed(()=>Boolean(query.value.trim() || kind.value));
let summaryVersion=0,noticeTimer;
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
onUnmounted(()=>{++generation;++summaryVersion;clearTimeout(noticeTimer);hidePassword();window.removeEventListener('blur',hidePassword);document.removeEventListener('visibilitychange',onVisibility);});
async function load(){
  const token=++generation;
  error.value='';
  loading.value=true;
  try{
    const result=await findClips(query.value,null,{kind:kind.value || null,category_id:props.category});
    if(token===generation) rows.value=result;
  }catch(e){if(token===generation){rows.value=[];error.value=String(e.message || e);}}
  finally{if(token===generation)loading.value=false;}
}
async function loadSummary(){
  const token=++summaryVersion;
  try{
    const result=await clipCategories();
    if(token!==summaryVersion)return;
    summary.value=result;
    summaryError.value='';
    emit('categories',result);
    if(props.category && !result.categories.some(g=>g.id===props.category)) emit('select-category',null);
  }catch(e){if(token===summaryVersion)summaryError.value=String(e.message || e);}
}
function refresh(){return Promise.all([load(),loadSummary()]);}
onMounted(loadSummary);
watch([query,kind,()=>props.category],()=>{hidePassword();notice.value='';removing.value='';load();},{immediate:true});
function edit(row){
  hidePassword();notice.value='';error.value='';removing.value='';
  draft.value=row || {kind:kind.value || 'text',category_id:props.category};
}
function cancelEdit(){draft.value=null;error.value='';}
function notify(message){
  clearTimeout(noticeTimer);
  notice.value=message;
  noticeTimer=setTimeout(()=>notice.value='',2200);
}
async function saved(item){
  const creating=!draft.value?.id;
  cancelEdit();
  if(creating){
    query.value='';
    if(kind.value)kind.value=item.kind;
    if(props.category!==null)emit('select-category',item.category_id);
  }
  await refresh();notify('已保存');
}
async function act(fn){
  if(busy.value)return;
  busy.value=true;error.value='';notice.value='';
  try{await fn();}catch(e){error.value=String(e.message || e);}
  finally{busy.value=false;}
}
function copy(row){return act(async()=>{await copyClip(row.id);notify('已复制');});}
function remove(row){return act(async()=>{await clipRequest({action:'delete',id:row.id});removing.value='';await refresh();notify('已删除');});}
defineExpose({editing:computed(()=>Boolean(draft.value)),focusSearch:()=>searchInput.value?.focus(),refresh});
</script>
<style scoped>
.clips-page{padding:24px clamp(16px,3vw,36px);overflow:auto}
.clips-inner{width:100%;max-width:920px;margin-inline:auto}.clips-inner.editing{max-width:660px}
.clips-heading{display:flex;justify-content:space-between;align-items:flex-start;gap:12px;margin-bottom:24px}
.clips-heading h1{margin:0;font-size:24px;letter-spacing:-.035em;font-weight:620;overflow-wrap:anywhere}
.clips-heading p,.clip-copy p,.empty p{margin:4px 0 0;color:var(--muted);font-size:13px}
.back,.quiet{background:transparent;border:0;color:var(--muted);cursor:pointer;font-size:12px}
.back{margin:0 0 10px;padding:2px 0}.back:hover,.quiet:hover{color:var(--text)}
.clip-toolbar{display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:10px}
.clip-toolbar>.searchable-select{width:auto;max-width:180px;min-width:110px;border-radius:var(--radius-pill)}
.search{flex:1 1 180px;display:flex;align-items:center;gap:8px;min-width:0;padding:0 14px;background:var(--surface-secondary);border-radius:var(--radius-pill)}
.search:focus-within{outline:2px solid var(--focus);outline-offset:1px}
.search svg,.clips-heading svg{width:16px;height:16px}
.search input{background:transparent;border:0;color:var(--text);height:37px;min-width:0;width:100%;outline:none}
.clip-row{display:flex;align-items:center;gap:12px;border-bottom:1px solid var(--line);padding:14px 2px}
.kind-icon{width:34px;height:34px;flex:none;display:grid;place-items:center;background:var(--surface-secondary);border-radius:10px}
.kind-icon svg,.empty svg{width:18px;height:18px;color:var(--muted)}
.clip-copy{flex:1;min-width:0}.clip-copy strong,.clip-copy p,.clip-copy small{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.clip-copy strong{font-weight:600}.clip-copy small{margin-top:3px;color:var(--muted);font-size:11px}
.clip-copy .visible-secret{white-space:pre-wrap;overflow-wrap:anywhere;font-family:var(--font-code);color:var(--text)}
.clip-actions{display:flex;align-items:center;gap:3px;flex-wrap:wrap}.clip-actions .button{min-height:28px;padding:3px 10px}.quiet{padding:5px 7px;border-radius:6px}.quiet:hover{background:var(--hover)}
.empty{display:grid;justify-items:center;gap:8px;text-align:center;padding:38px 12px}.empty svg{width:24px;height:24px}
.notice{position:sticky;bottom:10px;width:fit-content;margin:10px auto 0;padding:5px 14px;background:var(--popover);border:1px solid var(--line);border-radius:20px;box-shadow:var(--shadow-menu)}
.error{color:var(--danger)}
@media(max-width:700px){.clip-row{flex-wrap:wrap}.clip-actions{margin-left:auto}.clips-page{padding:18px}.clip-copy{min-width:120px}}
</style>
