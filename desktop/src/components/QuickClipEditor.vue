<template>
  <form class="clip-editor" @submit.prevent="save" @keydown="onShortcut">
    <div class="fields">
      <fieldset class="type-picker">
        <legend>{{ tr('类型') }}</legend>
        <div class="type-list">
          <label v-for="(label,key) in clipTypes" :key="key" :class="{chosen:kind===key}">
            <input type="radio" name="clip-kind" :value="key" v-model="kind" :disabled="busy || !!initial.id" @change="changeKind">
            <AppIcon :name="clipIcons[key]"/>{{ tr(label) }}
          </label>
        </div>
      </fieldset>
      <div class="identity">
        <label class="field">{{ tr('名称') }}
          <input ref="titleInput" v-model="title" required maxlength="120" :disabled="busy" :placeholder="tr('为片段起个名字')" autocomplete="off">
        </label>
        <div class="field"><span>{{ tr('分类') }}</span>
          <SearchableSelect v-model="category" :options="categoryOptions" :aria-label="tr('片段分类')" :disabled="busy" :searchable="groups.length>6" />
        </div>
      </div>
      <label class="field">{{ tr(valueLabel) }}
        <input v-if="kind==='password'" v-model="value" type="password" :required="!initial.id" :disabled="busy" autocomplete="new-password">
        <textarea v-else v-model="value" required :disabled="busy" :rows="kind==='url' || kind==='account' ? 2 : 4" :placeholder="placeholder" />
      </label>
      <p v-if="kind==='password'" class="hint">{{ tr(initial.id ? '密码存入系统凭据库，仅按名称搜索。编辑时留空保留原密码。' : '密码存入系统凭据库，仅按名称搜索。') }}</p>
      <p v-if="error" role="alert">{{ tr(error) }}</p>
    </div>
    <footer><button type="button" :disabled="busy" @click="emit('cancel')">{{ tr('取消') }}</button>
      <button type="submit" class="save" :disabled="busy">{{ tr('保存') }}</button></footer>
  </form>
</template>
<script setup>
import {computed,onMounted,ref} from 'vue';
import AppIcon from './AppIcon.vue';
import SearchableSelect from './SearchableSelect.vue';
import {clipIcons,clipTypes,clipRequest} from '../platform/quickClips.js';
import {tr} from '../platform/interfaceLanguage.js';

const props=defineProps({initial:{type:Object,required:true},groups:{type:Array,default:()=>[]}});
const emit=defineEmits(['saved','cancel','busy']);
const kind=ref(props.initial.kind || 'text'), title=ref(props.initial.title || '');
const value=ref(props.initial.content ?? ''), category=ref(props.initial.category_id || '');
const busy=ref(false), error=ref(''), titleInput=ref(null);
let lastKind=kind.value;
const valueLabel=computed(()=>['password','url','account'].includes(kind.value) ? clipTypes[kind.value] : '内容');
const placeholder=computed(()=>({url:'https://…',account:tr('输入账号'),template:tr('粘贴常用模板'),text:tr('输入或粘贴片段内容')}[kind.value]));
const categoryOptions=computed(()=>[{value:'',label:tr('未分类')},...props.groups.map(g=>({value:g.id,label:g.name}))]);
function changeKind(){
  if(lastKind==='password' || kind.value==='password') value.value='';
  lastKind=kind.value;
  error.value='';
}
function onShortcut(event){
  if(event.key==='Enter' && (event.ctrlKey || event.metaKey) && !event.isComposing && !event.repeat){
    event.preventDefault();
    event.currentTarget.requestSubmit();
  }
}
async function save(){
  if(busy.value) return;
  error.value='';
  if(!title.value.trim()){error.value='请填写片段名称';titleInput.value?.focus();return;}
  busy.value=true;
  emit('busy',true);
  try{
    const id=await clipRequest({action:'save',id:props.initial.id || null,kind:kind.value,title:title.value.trim(),
      value:kind.value==='password' && props.initial.id && !value.value ? null : value.value,category_id:category.value});
    value.value='';
    emit('saved',{id,title:title.value.trim(),kind:kind.value,category_id:category.value});
  }catch(e){error.value=String(e.message || e);}
  finally{busy.value=false;emit('busy',false);}
}
onMounted(()=>titleInput.value?.focus());
</script>
<style scoped>
.clip-editor{display:flex;flex-direction:column;min-height:0;max-width:660px;color:var(--text)}
.fields{display:grid;gap:19px;min-height:0}
.type-picker{border:0;margin:0;padding:0;min-width:0}
.type-picker legend,.field{font-size:12px;font-weight:550}
.type-picker legend{margin-bottom:8px;padding:0}
.type-list{display:flex;gap:4px;flex-wrap:wrap}
.type-list label{display:flex;position:relative;align-items:center;justify-content:center;gap:6px;padding:6px 12px;border:1px solid transparent;border-radius:8px;font-size:13px;color:var(--muted);cursor:pointer}
.type-list label:hover{background:var(--hover)}
.type-list label.chosen{background:var(--accent-soft);color:var(--text);font-weight:550}
.type-list label:has(:focus-visible){outline:2px solid var(--focus);outline-offset:2px}
.type-list input{position:absolute;width:1px;height:1px;opacity:0}
.type-list svg{width:15px;height:15px}
.identity{display:grid;grid-template-columns:minmax(0,1.7fr) minmax(115px,1fr);gap:14px}
.field{display:flex;min-width:0;flex-direction:column;gap:6px}
.field :is(input,textarea){display:block;width:100%;padding:9px 12px;background:var(--surface);color:var(--text);border:1px solid var(--control-border);border-radius:10px;font-size:14px;font-weight:400;line-height:1.5}
.field textarea{resize:vertical;min-height:85px}
.field :deep(.searchable-select){height:41px;font-weight:400}
.fields p{margin:0;color:var(--muted);font-size:12px}
.fields p[role=alert]{color:var(--danger)}
footer{display:flex;justify-content:flex-end;gap:8px;margin-top:20px;border-top:1px solid var(--line);padding-top:14px}
footer button{padding:7px 18px;border:1px solid var(--control-border);border-radius:var(--radius-pill);background:var(--surface);font-size:13px;cursor:pointer}
footer .save{background:var(--accent);color:var(--bg);border-color:var(--accent)}
footer button:disabled{opacity:.55}
@media(max-width:480px){.identity{grid-template-columns:1fr}.type-list label{padding:5px 7px}}
body.launcher-page .clip-editor{max-width:none;height:100%}
body.launcher-page .fields{overflow-y:auto;align-content:start;flex:1;padding:12px 18px;gap:10px}
body.launcher-page .field textarea{min-height:60px}
body.launcher-page footer{margin-top:0;padding:8px 16px}
</style>
