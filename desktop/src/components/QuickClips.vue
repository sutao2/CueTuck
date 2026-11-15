<template>
  <main class="content-area clips-page"><div class="clips-inner" :class="{editing:draft}">
    <header class="clips-heading">
      <div><button v-if="draft" class="back" :disabled="busy" @click="cancelEdit">‹ {{ tr('常用片段') }}</button>
        <div class="heading-line"><h1>{{ draft ? tr(draft.id ? '编辑片段' : '添加片段') : groupTitle }}</h1>
          <span v-if="!draft && !loading" class="result-count" :aria-label="tr('共 {0} 个结果',[rows.length])" :title="tr('共 {0} 个结果',[rows.length])">{{ rows.length }}</span>
        </div></div>
      <button v-if="!draft" class="button primary-button" @click="edit()"><AppIcon name="plus"/>{{ tr('添加片段') }}</button>
    </header>
    <QuickClipEditor v-if="draft" :initial="draft" :groups="summary.categories" @cancel="cancelEdit" @saved="saved" @busy="busy=$event"/>
    <template v-else>
      <div class="clip-toolbar" :class="{compact:compactNav}">
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
        <span class="kind-icon" :class="row.kind" :title="tr(clipTypes[row.kind])"><AppIcon :name="clipIcons[row.kind]"/></span>
        <div class="clip-copy">
          <div class="clip-title"><strong>{{ row.title }}</strong><small>{{ tr(clipTypes[row.kind]) }}</small></div>
          <div class="row-preview">
            <p :class="{'visible-secret':visibleId===row.id}">{{ row.kind==='password' ? (visibleId===row.id ? visibleValue : '••••••••') : row.content }}</p>
            <button v-if="row.kind==='password'" class="row-tool" :aria-pressed="visibleId===row.id" :title="tr(visibleId===row.id ? '隐藏密码' : '显示密码')" :disabled="busy" @click="togglePassword(row)">
              <AppIcon :name="visibleId===row.id ? 'eye-off' : 'eye'"/><span class="sr-only">{{ tr(visibleId===row.id ? '隐藏密码' : '显示密码') }}</span>
            </button>
          </div>
        </div>
        <span v-if="category===null && groupNames.has(row.category_id)" class="row-category"><AppIcon name="folder"/><span>{{ groupNames.get(row.category_id) }}</span></span>
        <div class="clip-actions">
          <template v-if="removing===row.id">
            <button class="quiet" :disabled="busy" @click="removing=''">{{ tr('取消') }}</button>
            <button class="confirm-delete" :disabled="busy" @click="remove(row)">{{ tr('确认删除') }}</button>
          </template>
          <template v-else>
            <button class="row-tool" :disabled="busy" :title="tr('复制')" @click="copy(row)"><AppIcon name="copy"/><span class="sr-only">{{ tr('复制') }}</span></button>
            <div class="row-more" @focusout="leaveMore" @keydown.esc.stop.prevent="closeMore(true)">
              <button class="row-tool" :aria-expanded="moreId===row.id" :aria-label="tr('{0}的更多操作',[row.title])" :title="tr('更多操作')" :disabled="busy" @click="toggleMore($event,row.id)"><AppIcon name="more"/></button>
              <div v-if="moreId===row.id" class="more-items" :class="{above:openAbove}">
                <button type="button" @click="edit(row)">{{ tr('编辑') }}</button>
                <button type="button" class="delete" @click="confirmRemove(row)">{{ tr('删除') }}</button>
              </div>
            </div>
          </template>
        </div>
      </article>
      <p v-if="notice" role="status" class="notice">{{ tr(notice) }}</p>
    </template>
  </div></main>
</template>
<script setup>
import {ref,watch,nextTick,onMounted,onUnmounted,computed} from 'vue';
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
const moreId=ref(null),openAbove=ref(false);
let moreTrigger=null;
function closeMore(restoreFocus=false){
  moreId.value=null;
  if(restoreFocus)moreTrigger?.focus();
  moreTrigger=null;
}
function toggleMore(event,id){
  if(id===moreId.value){closeMore();return;}
  moreTrigger=event.currentTarget;
  openAbove.value=moreTrigger.getBoundingClientRect().bottom+96>window.innerHeight;
  moreId.value=id;
}
function leaveMore(event){if(!event.currentTarget.contains(event.relatedTarget))closeMore();}
function outsideMore(event){if(moreId.value && !moreTrigger?.parentElement?.contains(event.target))closeMore();}
async function confirmRemove(row){
  const actions=moreTrigger?.closest('.clip-actions');
  closeMore();
  hidePassword();
  removing.value=row.id;
  await nextTick();
  actions?.querySelector('button')?.focus();
}
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
onMounted(()=>{document.addEventListener('pointerdown',outsideMore);window.addEventListener('blur',hidePassword);document.addEventListener('visibilitychange',onVisibility);});
onUnmounted(()=>{++generation;++summaryVersion;clearTimeout(noticeTimer);document.removeEventListener('pointerdown',outsideMore);hidePassword();window.removeEventListener('blur',hidePassword);document.removeEventListener('visibilitychange',onVisibility);});
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
watch([query,kind,()=>props.category],()=>{closeMore();hidePassword();rows.value=[];notice.value='';removing.value='';load();},{immediate:true});
function edit(row){
  closeMore();hidePassword();notice.value='';error.value='';removing.value='';
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
.clips-page{padding:26px clamp(18px,3vw,36px);overflow:auto}
.clips-inner{width:100%;max-width:920px;margin-inline:auto;container-type:inline-size}
.clips-inner.editing{max-width:660px}
.clips-heading,.heading-line,.clip-title,.clip-actions,.row-category{display:flex;align-items:center}
.clips-heading{justify-content:space-between;gap:12px;margin-bottom:18px}
.heading-line{gap:9px;min-width:0}
.clips-heading h1{margin:0;font-size:23px;letter-spacing:-.035em;font-weight:620;overflow-wrap:anywhere}
.result-count{padding:1px 7px;border-radius:6px;font:500 12px/19px var(--font-ui);font-variant-numeric:tabular-nums;color:var(--muted);background:var(--surface-secondary)}
.clips-heading .primary-button{min-height:32px;white-space:nowrap}
.clips-heading svg,.search svg{width:15px;height:15px;flex:none}
.back,.quiet,.confirm-delete{background:transparent;border:0;color:var(--muted);cursor:pointer;font-size:12px}
.back{margin:0 0 10px;padding:2px 0}.back:hover,.quiet:hover{color:var(--text)}
.clip-toolbar{display:flex;align-items:center;min-height:40px;gap:2px;padding:3px 5px 3px 12px;background:var(--surface-secondary);border-radius:11px;margin-bottom:14px}
.search{flex:1 1 130px;display:flex;align-items:center;gap:9px;min-width:0;color:var(--muted)}
.search input{background:transparent;border:0;color:var(--text);height:32px;min-width:0;width:100%;outline:none;font-size:13px}
.clip-toolbar:has(.search input:focus-visible){outline:2px solid var(--focus);outline-offset:2px}
.clip-toolbar :deep(.searchable-select){height:30px;min-height:30px;min-width:92px;width:auto;max-width:150px;border:0;border-radius:7px;background:transparent;padding:4px 8px;color:var(--muted);gap:8px}
.clip-toolbar :deep(.searchable-select:hover),.clip-toolbar :deep(.searchable-select[aria-expanded=true]){background:var(--hover);color:var(--text)}
.search::after{content:"";width:1px;height:16px;background:var(--line-strong);margin-right:3px}
.clip-row{position:relative;display:flex;align-items:center;gap:12px;padding:13px 4px;border-bottom:1px solid var(--line)}
.clip-row:last-of-type{border-bottom-color:transparent}
.kind-icon{width:34px;height:34px;flex:none;display:grid;place-items:center;border-radius:9px;background:var(--surface-secondary);color:var(--muted)}
.kind-icon.url{background:color-mix(in srgb,var(--blue) 8%,var(--surface));color:color-mix(in srgb,var(--blue) 60%,var(--text))}
.kind-icon.password{background:color-mix(in srgb,#9782bd 12%,var(--surface));color:color-mix(in srgb,#9782bd 60%,var(--text))}
.kind-icon svg{width:17px;height:17px}
.clip-copy{flex:1;min-width:0}
.row-preview{display:flex;align-items:center;gap:5px;min-height:20px;margin-top:2px}.row-preview .row-tool{height:20px;width:24px}.row-preview .row-tool svg{width:14px;height:14px}.clip-title{gap:9px}.clip-title strong,.clip-copy p,.row-category{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.clip-title strong{font-size:14px;font-weight:590}
.clip-title small,.row-category{font-size:11px;color:var(--muted)}
.clip-title small{flex:none}.clip-copy p{margin:0;color:var(--muted);font-size:12px}
.clip-copy .visible-secret{white-space:pre-wrap;overflow-wrap:anywhere;font-family:var(--font-code);color:var(--text)}
.row-category{gap:5px;max-width:110px}.row-category span{overflow:hidden;text-overflow:ellipsis}.row-category svg{width:13px;height:13px;flex:none}
.clip-actions{gap:2px;flex:none}
.row-tool{display:grid;place-items:center;width:30px;height:30px;padding:0;border:0;background:transparent;border-radius:7px;color:var(--muted);cursor:pointer}
.row-tool:hover,.row-tool[aria-expanded=true],.row-tool[aria-pressed=true]{background:var(--accent-soft);color:var(--text)}
.row-tool svg{width:16px;height:16px}.row-more{position:relative}
.more-items{position:absolute;right:0;top:calc(100% + 3px);z-index:3;min-width:108px;display:grid;gap:2px;padding:4px;border:1px solid var(--line);border-radius:9px;background:var(--popover);box-shadow:var(--shadow-menu)}
.more-items.above{top:auto;bottom:calc(100% + 3px)}
.more-items button{border:0;background:transparent;padding:6px 10px;border-radius:6px;text-align:left;font-size:13px;cursor:pointer}
.more-items button:hover,.more-items button:focus-visible{background:var(--hover)}.more-items .delete,.confirm-delete,.error{color:var(--danger)}
.empty{display:grid;justify-items:center;gap:8px;text-align:center;padding:42px 12px}.empty svg{width:23px;height:23px;color:var(--muted)}.empty p{margin:0;color:var(--muted);font-size:13px}
.notice{position:sticky;bottom:10px;width:fit-content;margin:10px auto 0;padding:5px 14px;background:var(--popover);border:1px solid var(--line);border-radius:20px;box-shadow:var(--shadow-menu)}
@container(max-width:470px){.row-category{display:none}.clip-row{gap:8px}.clip-toolbar.compact{flex-wrap:wrap}.clip-toolbar.compact .search{flex-basis:100%}.clip-toolbar.compact .search::after{display:none}}
</style>
