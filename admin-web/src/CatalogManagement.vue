<template>
  <div class="catalog-workspace">
    <div class="catalog-toolbar"><label class="search-label">搜索{{ label }}<input v-model="query" :placeholder="isCategory ? '名称或分类 ID' : '名称、厂商或模型值'" type="search"></label><label>状态<select v-model="status"><option value="">全部状态</option><option value="enabled">启用</option><option value="disabled">停用</option></select></label><button :disabled="loading || busy" @click="load">刷新</button><button class="primary" data-testid="catalog-new" :disabled="!loaded || busy" @click="open()">＋ 新建{{ label }}</button></div>
    <p v-if="error" class="error-banner" role="alert">{{ error }}</p><p v-if="message" class="success-message" role="status">{{ message }}</p>
    <div class="catalog-view-tools"><template v-if="isCategory"><button @click="collapsed=new Set(rows.filter(r=>!r.parent_id).map(r=>r.id))">折叠大分类</button><button @click="collapsed=new Set()">展开全部</button></template><label v-else>模型类型<select v-model="groupFilter"><option value="">全部类型</option><option v-for="(text,key) in groups" :key="key" :value="key">{{ text }}</option></select></label></div>
    <section class="panel"><div class="panel-heading"><h2>{{ isCategory ? '广场分类' : '提示词适用模型' }}</h2><span class="badge">{{ filtered.length }} 项</span></div>
      <p class="panel-note">{{ isCategory ? '仅管理公开广场的两级分类，不影响用户本地分类。' : '这里维护提示词的模型标签，不是 AI 审核 API 配置。模型值保持不变，显示名称可修改。' }} 停用不下架已有内容；引用数包含广场条目及投稿历史（含合集成员）。</p>
      <p v-if="loading" class="empty-state" role="status">正在加载…</p><ul v-else class="catalog-list"><li v-for="item in filtered" :key="item.id" :class="{ child: isCategory && item.parent_id }"><span v-if="isCategory" class="category-color" :style="{ background: item.color }"></span><div><strong>{{ item.name }}</strong><small>{{ isCategory && item.parent_id ? `${parentName(item.parent_id)} / ` : '' }}{{ item.id }}{{ !isCategory ? ` · ${item.vendor || '未指定厂商'} · ${groups[item.group]}` : '' }}</small></div><span class="badge" :class="{ enabled:item.enabled }">{{ item.enabled ? '启用' : '停用' }}</span><small>{{ item.references }} 处引用</small><button data-testid="catalog-edit" :disabled="busy" @click="open(item)">编辑</button></li><li v-if="loaded && !filtered.length" class="empty-state">没有符合条件的{{ label }}</li></ul>
    </section>
    <section v-if="draft" ref="editor" class="panel catalog-editor" :aria-label="`${label}编辑`"><div class="panel-heading"><h2>{{ editing ? `编辑${label}` : `新建${label}` }}</h2><button data-testid="catalog-close" :disabled="busy" @click="close">关闭</button></div>
      <form @submit.prevent="save"><fieldset :disabled="busy"><div class="catalog-fields">
        <label>显示名称<input v-model="draft.name" data-testid="catalog-name" maxlength="80" required autofocus></label>
        <label v-if="!isCategory && !editing">模型值<input v-model="draft.id" data-testid="catalog-id" maxlength="100" required placeholder="例如：my-model"><small>保存后不可修改，用于提示词筛选与关联。</small></label>
        <label v-if="isCategory">上级分类<select v-model="draft.parent_id" data-testid="catalog-parent" :disabled="hasChildren"><option :value="null">无（大分类）</option><option v-for="parent in parents" :key="parent.id" :value="parent.id">{{ parent.name }}</option></select><small v-if="hasChildren">已有子分类，不能降为小分类。</small></label>
        <template v-if="!isCategory"><label>厂商<input v-model="draft.vendor" maxlength="100" placeholder="厂商名称"></label><label>模型类型<select v-model="draft.group"><option v-for="(text,key) in groups" :key="key" :value="key">{{ text }}</option></select></label><label>地区<select v-model="draft.region"><option value="">未指定</option><option>国内</option><option>国外</option></select></label></template>
        <template v-else><label>分类图标<select v-model="draft.icon"><option v-for="(name,id) in icons" :key="id" :value="id">{{ name }}</option></select></label><label>分类颜色<input v-model="draft.color" type="color"></label></template>
        <label>排序权重<input v-model.number="draft.sort_index" type="number" min="-1000000" max="1000000" step="1" required><small>数值越小越靠前。</small></label><label class="check"><input v-model="draft.enabled" type="checkbox">启用{{ label }}</label>
      </div><p v-if="formError" class="error-banner" role="alert">{{ formError }}</p><footer class="catalog-actions"><div><small v-if="editing">{{ draft.id }} · 版本 {{ draft.revision }}</small><small v-if="deleteBlocked">存在内容引用或子分类，不能删除；可先停用。</small></div><button v-if="editing" class="danger" type="button" data-testid="catalog-delete" :disabled="deleteBlocked" @click="remove">删除</button><button class="primary" :disabled="!dirty" type="submit">{{ busy ? '正在保存…' : '保存变更' }}</button></footer></fieldset></form>
    </section>
    <section v-if="editing && draft" class="panel catalog-migration"><h2>迁移引用并移除{{ label }}</h2><p class="muted">公开条目和合集成员迁移到所选{{ label }}，原始投稿、正文和本地库保持不变。大分类的子分类将一起移到目标下；历史投稿以后上架会使用新目标。</p><p v-if="dirty" class="muted">请先保存或放弃上面的编辑，再迁移引用。</p><form @submit.prevent="migrate"><fieldset :disabled="busy || dirty"><label>迁移到<select v-model="migrationTarget" data-testid="catalog-migration-target" required><option value="">选择已启用的目标</option><option v-for="item in migrationTargets" :key="item.id" :value="item.id">{{ item.parent_id ? `${parentName(item.parent_id)} / ` : '' }}{{ item.name }}</option></select></label><label>迁移原因<textarea v-model="migrationReason" data-testid="catalog-migration-reason" maxlength="1000" required rows="2"></textarea></label><small>当前审核 Skill 引用或同名子分类冲突会阻止迁移，需要先处理。此操作不会更改审核策略。</small><button class="danger" data-testid="catalog-migrate" :disabled="!migrationTarget || !migrationReason.trim()">确认迁移并移除</button></fieldset></form></section>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { rememberListControls } from './listState.js';
import { listCatalog, saveCatalog, deleteCatalog, migrateCatalog } from './adminApi.js';
const props = defineProps({ kind: { type:String, required:true } });
const emit = defineEmits(['busy-change']);
const rows=ref([]), query=ref(''), status=ref(''), loaded=ref(false), loading=ref(false), busy=ref(false), error=ref(''), formError=ref(''), message=ref('');
const draft=ref(null), baseline=ref(''), editing=ref(false), editor=ref(null), references=ref(0);
const migrationTarget=ref(''),migrationReason=ref(''),collapsed=ref(new Set()),groupFilter=ref('');
rememberListControls(props.kind, { query, status, groupFilter });
let version=0;
const isCategory=computed(()=>props.kind==='categories'), label=computed(()=>isCategory.value?'分类':'模型');
const groups={language:'语言 / 推理',image:'图片生成',video:'视频生成'};
const icons={folder:'文件夹',blue:'代码',violet:'图片',coral:'视频',amber:'办公',green:'写作',teal:'设计',rose:'营销',cyan:'数据',gold:'教育',lime:'生活'};
const parents=computed(()=>rows.value.filter(r=>!r.parent_id && r.id!==draft.value?.id));
const hasChildren=computed(()=>editing.value && isCategory.value && rows.value.some(r=>r.parent_id===draft.value?.id));
const deleteBlocked=computed(()=>references.value>0 || hasChildren.value);
const dirty=computed(()=>draft.value!==null && JSON.stringify(draft.value)!==baseline.value);
const unsaved=computed(()=>dirty.value || Boolean(draft.value && (migrationTarget.value || migrationReason.value)));
const migrationTargets=computed(()=>rows.value.filter(r=>r.id!==draft.value?.id && r.enabled && (!isCategory.value || ((!draft.value?.parent_id ? !r.parent_id : true) && r.parent_id!==draft.value?.id && (!r.parent_id || rows.value.find(p=>p.id===r.parent_id)?.enabled)))));
defineExpose({hasUnsavedChanges:unsaved,isBusy:busy});
function parentName(id){return rows.value.find(r=>r.id===id)?.name || id;}
const filtered=computed(()=>{
  const needle=query.value.trim().toLowerCase();
  const ordered=isCategory.value ? rows.value.filter(r=>!r.parent_id).flatMap(root=>[root,...rows.value.filter(r=>r.parent_id===root.id)]) : rows.value;
  return ordered.filter(r=>(!isCategory.value || needle || !r.parent_id || !collapsed.value.has(r.parent_id)) && (isCategory.value || !groupFilter.value || r.group===groupFilter.value) && (!status.value || r.enabled===(status.value==='enabled')) && (!needle || `${r.name} ${r.id} ${r.vendor} ${parentName(r.parent_id)}`.toLowerCase().includes(needle)));
});
function canLeave(){return !busy.value && (!unsaved.value || window.confirm('有未保存的修改，确定放弃吗？'));}
async function load(){
  if(busy.value)return;
  const current=++version; loading.value=true; error.value='';
  try{const result=await listCatalog(props.kind); if(current!==version)return; if(!Array.isArray(result.items))throw new Error('字典响应无效'); rows.value=result.items; loaded.value=true;}
  catch(caught){if(current===version){error.value=caught.message;loaded.value=false;}}
  finally{if(current===version)loading.value=false;}
}
async function open(item){
  if(!canLeave())return;
  editing.value=Boolean(item); references.value=item?.references??0; formError.value='';
  migrationTarget.value='';migrationReason.value='';
  const {references:unused,...fields}=item??{id:isCategory.value?`cat-${crypto.randomUUID()}`:'',name:'',parent_id:null,icon:'folder',color:'#728080',vendor:'',group:'language',enabled:true,sort_index:0,revision:0};
  draft.value={region:'',...fields}; baseline.value=JSON.stringify(draft.value); await nextTick(); editor.value?.scrollIntoView?.({block:'nearest'}); editor.value?.querySelector('input')?.focus({preventScroll:true});
}
function close(){if(canLeave()){draft.value=null;formError.value='';}}
function setBusy(value){busy.value=value;emit('busy-change',value);}
async function migrate(){
  if(busy.value||dirty.value||!migrationTarget.value||!migrationReason.value.trim())return;
  const target=rows.value.find(r=>r.id===migrationTarget.value);if(!target)return;
  if(!window.confirm(`将「${draft.value.name}」的公开引用迁移到「${target.name}」并移除原${label.value}？投稿历史和正文不会更改。`))return;
  setBusy(true);error.value='';message.value='';
  try{const result=await migrateCatalog(props.kind,draft.value.id,{target:target.id,revision:draft.value.revision,target_revision:target.revision,reason:migrationReason.value.trim()});if(result.migrated!==true)throw new Error(result.message||'服务端未确认迁移');draft.value=null;migrationTarget.value='';migrationReason.value='';message.value=`已迁移 ${result.content} 条公开内容、${result.children} 个子分类，历史引用已保留。`}
  catch(e){error.value=e.message}finally{setBusy(false)}
  if(!draft.value)await load();
}
async function save(){
  if(busy.value || !dirty.value)return;
  if(!draft.value.name.trim() || !draft.value.id.trim() || !Number.isInteger(draft.value.sort_index)){formError.value='请填写名称、模型值和整数排序权重';return;}
  setBusy(true);formError.value='';message.value='';
  try{const result=await saveCatalog(props.kind,{...draft.value,name:draft.value.name.trim(),id:draft.value.id.trim()},editing.value); if(result.id!==draft.value.id.trim() || !Number.isSafeInteger(result.revision))throw new Error('服务端未确认保存，请刷新后检查'); draft.value=null;message.value='已保存，广场下次加载时生效。';}
  catch(caught){formError.value=caught.message;}
  finally{setBusy(false);}
  if(!draft.value)await load();
}
async function remove(){
  if(busy.value || deleteBlocked.value || !window.confirm(`确认删除「${draft.value.name}」？不会删除任何提示词。`))return;
  setBusy(true);formError.value='';message.value='';
  try{const result=await deleteCatalog(props.kind,draft.value.id,draft.value.revision);if(result.deleted!==true)throw new Error('服务端未确认删除');draft.value=null;message.value='已删除，不会在重启后恢复。';}
  catch(caught){formError.value=caught.message;}
  finally{setBusy(false);}
  if(!draft.value)await load();
}
onMounted(load);onUnmounted(()=>{++version;});
</script>

<style scoped>
.catalog-view-tools{display:flex;gap:10px}.catalog-view-tools select{min-width:180px}.catalog-migration{padding:24px}.catalog-migration h2{font-size:15px}.catalog-migration p{line-height:1.7}.catalog-migration fieldset{display:grid;gap:16px}.catalog-migration textarea{width:100%;font:inherit;border:1px solid #dce1e4;border-radius:7px;padding:10px;margin-top:8px;resize:vertical}.catalog-migration button{justify-self:start}
.catalog-workspace{display:grid;gap:20px}.catalog-toolbar{display:flex;gap:12px;align-items:end;flex-wrap:wrap}.search-label{flex:1;min-width:180px}select{display:block;width:100%;font:inherit;color:inherit;background:white;border:1px solid #dce1e4;border-radius:7px;padding:10px 12px;margin-top:8px}.catalog-list{list-style:none;margin:0;padding:0}.catalog-list li{display:flex;align-items:center;gap:14px;padding:14px 24px;border-top:1px solid #edf0f2}.catalog-list li.child{padding-left:48px}.catalog-list li>div{flex:1;min-width:0;overflow-wrap:anywhere}.catalog-list strong{font-size:13px;font-weight:500}small{display:block;color:#8a929a;font-size:11px;margin-top:4px}.category-color{width:10px;height:10px;border-radius:3px;flex-shrink:0}.catalog-editor{scroll-margin:24px}.catalog-editor form{padding:24px}.catalog-fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:20px}.check{display:flex;align-items:center;gap:8px}.check input{width:auto;margin:0}.catalog-actions{display:flex;align-items:center;gap:12px;margin-top:24px;border-top:1px solid #edf0f2;padding-top:20px}.catalog-actions>div{flex:1;overflow-wrap:anywhere}.danger{color:#b34848}.error-banner{margin:16px 0 0}input[type=color]{height:40px;width:100%}@media(max-width:640px){.catalog-fields{grid-template-columns:1fr}.catalog-list li{flex-wrap:wrap;padding:14px 16px}.catalog-list li.child{padding-left:30px}.catalog-list li>div{min-width:140px}.catalog-editor form{padding:16px}}
</style>
