<template>
  <div class="sidebar-toolbar"><span>{{ tr('资料分类') }}</span>
    <button type="button" class="mini-button" :disabled="disabled || busy" :title="tr('新建分类')" :aria-label="tr('新建分类')" @click="open()"><AppIcon name="plus"/></button>
  </div>
  <nav class="category-tree" :aria-label="tr('资料分类')">
    <div v-for="group in entries" :key="group.id ?? 'all'" class="tree-child-row">
      <button type="button" class="tree-row" :class="{active:selected===group.id}" :aria-current="selected===group.id ? 'true' : undefined" :disabled="disabled || busy" @click="choose(group.id)">
        <span class="tree-icon"><AppIcon :name="group.id===null ? 'square' : 'folder'"/></span>
        <span>{{ group.name }}</span><span class="tree-count">{{ group.count }}</span>
      </button>
      <button v-if="group.id" type="button" class="mini-button manage" :title="tr('管理分类')" :aria-label="tr('管理分类')+' '+group.name" :disabled="disabled || busy" @click="open(group)">···</button>
    </div>
    <form v-if="editing" class="group-form" @submit.prevent="submit" @keydown.esc.stop.prevent="close">
      <label v-if="!deleting">{{ tr(editing.id ? '重命名分类' : '新建分类') }}<input ref="nameInput" v-model="name" required maxlength="60" :disabled="busy"></label>
      <p v-else>{{ tr('删除「{0}」？其中的资料将移至未分类。',[editing.name]) }}</p>
      <p v-if="error" role="alert">{{ tr(error) }}</p>
      <div><button v-if="editing.id && !deleting" type="button" class="remove" :disabled="busy" @click="deleting=true">{{ tr('删除') }}</button>
        <button type="button" :disabled="busy" @click="close">{{ tr('取消') }}</button>
        <button type="submit" :disabled="busy || (!deleting && !name.trim())">{{ tr(deleting ? '确认删除' : '保存') }}</button></div>
    </form>
  </nav>
</template>
<script setup>
import {computed,nextTick,ref,watch} from 'vue';
import AppIcon from './AppIcon.vue';
import {clipRequest} from '../platform/quickClips.js';
import {tr} from '../platform/interfaceLanguage.js';
const props=defineProps({summary:{type:Object,required:true},selected:{default:null},disabled:Boolean});
const emit=defineEmits(['select','changed']);
const editing=ref(null),name=ref(''),nameInput=ref(null),deleting=ref(false),busy=ref(false),error=ref('');
const entries=computed(()=>[{id:null,name:tr('全部资料'),count:props.summary.total},{id:'',name:tr('未分类'),count:props.summary.uncategorized},...props.summary.categories]);
function close(){if(!busy.value){editing.value=null;error.value='';deleting.value=false;}}
function choose(id){close();emit('select',id);}
async function open(group={}){close();editing.value=group;name.value=group.name || '';await nextTick();nameInput.value?.focus();}
watch(()=>props.disabled,disabled=>{if(disabled)close();});
async function submit(){
  if(busy.value || props.disabled) return;
  busy.value=true;
  error.value='';
  try{
    const id=editing.value.id;
    if(deleting.value){
      await clipRequest({action:'delete_category',id});
      if(props.selected===id) emit('select','');
    }else if(id) await clipRequest({action:'rename_category',id,name:name.value.trim()});
    else emit('select',await clipRequest({action:'create_category',name:name.value.trim()}));
    editing.value=null;
    emit('changed');
  }catch(e){error.value=String(e.message || e);}
  finally{busy.value=false;}
}
</script>
<style scoped>
.mini-button svg{width:14px;height:14px;vertical-align:middle}
.tree-child-row{display:grid;grid-template-columns:minmax(0,1fr) 22px}
.manage{opacity:0}
.tree-child-row:hover .manage,.tree-child-row:focus-within .manage{opacity:1}
.group-form{display:grid;gap:8px;padding:10px 6px;font-size:12px}
.group-form label,.group-form input{display:block;width:100%}
.group-form input{margin-top:5px;border:1px solid var(--control-border);border-radius:8px;background:var(--surface);padding:6px 8px}
.group-form div{display:flex;justify-content:flex-end;gap:6px}
.group-form button{background:var(--surface);border:1px solid var(--line);border-radius:7px;padding:3px 7px}
.group-form p{margin:0}.group-form [role=alert],.group-form .remove{color:var(--danger)}.group-form .remove{margin-right:auto}
@media(hover:none){.manage{opacity:1}}
</style>
