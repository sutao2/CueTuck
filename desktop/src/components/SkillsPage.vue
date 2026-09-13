<template>
  <main ref="page" class="content-area skills-page" data-testid="skills-page">
    <section v-if="!supported" class="skills-empty"><AppIcon name="skills"/><h1>{{ mode === 'local' ? '本机 Skills' : 'Skill 广场' }}</h1><p>请在 CueTuck 桌面客户端查看与管理 Skills。</p><p class="muted">浏览器无法扫描本机目录或安装文件。</p></section>
    <template v-else>
      <header class="skills-header"><div><button v-if="detail || directories || maintenance" class="skills-back" @click="back">← 返回{{ mode === 'local' ? '本机 Skills' : 'Skill 广场' }}</button><p class="skills-eyebrow">SKILLS</p><h1>{{ maintenance ? '备份与操作记录' : directories ? '管理目录' : detail ? detail.package.name : mode === 'local' ? '本机 Skills' : 'Skill 广场' }}</h1><p class="muted">{{ maintenance ? '备份保存在本机，恢复前会检查原位置是否空闲。' : directories ? '仅扫描登记的位置，取消登记不会删除任何文件。' : detail ? detail.package.description : mode === 'local' ? `已发现 ${grouped.length} 个 Skill · ${snapshot.skills.length} 处安装` : '从公开仓库发现 Skill，查看内容后安装到指定位置。' }}</p></div><div class="skills-actions" v-if="!detail && !directories && !maintenance"><button class="button primary-button" :disabled="busy" @click="importLocal">从文件夹安装</button><button class="button" :disabled="busy" @click="maintenance=true">备份与记录</button><button class="button" :disabled="busy" @click="directories=true">管理目录</button><button class="button" :disabled="busy" @click="refresh">刷新</button></div></header>
      <p v-if="error" class="skills-alert" role="alert">{{ error }}<button v-if="!busy" @click="retryRequest">重试</button></p>
      <p v-if="notice" class="skills-notice" role="status">{{ notice }}<button @click="notice=''" aria-label="关闭提示">×</button></p>
      <p v-if="busy" class="skills-loading" role="status">{{ loadingText }}<span v-if="progress"> · {{ progress.completed }} / {{ progress.total }} 文件 · {{ progress.file }}</span><button v-if="networkBusy" class="skills-inline-button" @click="cancel">取消</button></p>
      <template v-if="maintenance">
        <section class="skills-panel"><h2>可恢复备份 <small>{{ snapshot.backups.length }} 份 · {{ formatBytes(snapshot.backups.reduce((n,b)=>n+b.bytes,0)) }}</small></h2><p class="muted">替换和移除前的完整文件包。恢复不会覆盖原位置的新文件，也不会自动删除备份。</p><p v-if="!snapshot.backups.length" class="muted">暂无备份</p><article v-for="backup in [...snapshot.backups].reverse()" :key="backup.id" class="skills-root"><div><strong>{{ backup.reason }}</strong><span class="skills-badge">{{ backup.restored ? '曾恢复' : '可恢复' }}</span><p class="skills-path">{{ backup.original }}</p><p class="muted">{{ date(backup.created_at) }} · {{ formatBytes(backup.bytes) }}</p></div><div class="skills-actions"><button :disabled="busy" @click="openFolder(backup.path)">查看备份</button><button :disabled="busy" @click="confirmation={kind:'restore',backup}">恢复到原位置</button></div></article></section>
        <section class="skills-panel"><h2>最近操作</h2><p v-if="!snapshot.operations.length" class="muted">暂无操作记录</p><article v-for="(operation,index) in snapshot.operations" :key="index" class="skills-root"><div><strong>{{ operation.status === 'success' ? '已完成' : '失败' }}</strong><p class="skills-path">{{ operation.target }}</p><p class="muted">{{ operation.message }} · {{ date(operation.at) }}</p></div></article></section>
      </template>
      <template v-else-if="directories">
        <section class="skills-panel"><h2>添加扫描目录</h2><div class="skills-form"><label>目录名称<input v-model="rootDraft.name" placeholder="例如：工作项目"/></label><label>智能体<SearchableSelect v-model="rootDraft.agent" :options="agents"/></label><label>作用范围<SearchableSelect v-model="rootDraft.scope" :options="[{value:'global',label:'全局'},{value:'project',label:'项目'}]"/></label><label class="wide">标准 Skill 根目录<div class="skills-path-input"><input v-model="rootDraft.path" placeholder="每个子目录包含 SKILL.md"/><button :disabled="busy" @click="pickRoot">选择文件夹</button></div></label></div><div class="skills-actions"><button class="button primary-button" :disabled="busy || !rootDraft.path || !rootDraft.name.trim()" @click="saveRoot">{{ rootDraft.id ? '保存目录' : '登记目录' }}</button><button class="button" :disabled="busy" @click="addProject">添加项目（五客户端）</button><button v-if="rootDraft.id" class="button" @click="resetRoot">取消编辑</button></div></section>
        <section class="skills-panel"><h2>扫描位置 <small>{{ snapshot.roots.length }}</small></h2><article v-for="root in snapshot.roots" :key="root.id" class="skills-root"><div><strong>{{ root.name }}</strong><span class="skills-badge">{{ scopeName(root.scope) }}</span><p class="skills-path">{{ root.path }}</p><p class="muted">{{ root.status }}</p><p v-if="root.shared_with.length > 1" class="muted">此目录可能被 {{ root.shared_with.join('、') }} 共同读取</p></div><div class="skills-actions"><button :disabled="busy" @click="openFolder(root.path)">打开文件夹</button><button v-if="root.custom" :disabled="busy" @click="editRoot(root)">编辑</button><button v-if="root.custom" :disabled="busy" @click="forgetRoot(root)">取消登记</button></div></article></section>
      </template>
      <template v-else-if="detail">
        <div class="skills-detail-grid"><section class="skills-panel"><div class="skills-section-heading"><h2>{{ selectedFileName || 'SKILL.md' }}</h2><span class="skills-badge">只读预览</span></div><p class="skills-path">{{ prepared?.source ? `${prepared.source.repo} / ${prepared.source.directory}` : selected.path }}</p><p v-if="prepared?.source" class="muted">已固定提交 {{ prepared.source.commit.slice(0,12) }} · {{ prepared.source.reference }}</p><p v-if="detail.modified" class="skills-alert">此安装存在本地修改，替换前需要比较并备份。</p><p v-for="warning in detail.package.warnings" :key="warning" class="skills-warning">{{ warning }}</p><pre class="skills-document">{{ selectedFileText ?? detail.package.body }}</pre></section><aside><section class="skills-panel"><h2>文件 <small>{{ detail.package.files.length }}</small></h2><button class="skills-file" v-for="file in detail.package.files" :key="file.path" :disabled="busy" @click="readFile(file.path)"><span>{{ file.path }}</span><small>{{ formatBytes(file.size) }}</small></button></section><section class="skills-panel"><h2>信息</h2><button class="button primary-button" :disabled="busy" @click="beginInstall">安装到…</button><p>许可：{{ detail.package.license }}<span v-if="detail.package.license === '未知' && prepared?.source?.license !== '未知' && prepared?.source?.license">（仓库声明 {{ prepared.source.license }}，请核实目录许可）</span></p><p>大小：{{ formatBytes(detail.package.bytes) }}</p><p class="muted">文件就绪不代表目标客户端已加载；专有工具和运行时需要另行配置。</p></section></aside></div>
        <section v-if="selected.installations?.length" class="skills-panel"><h2>安装位置</h2><article v-for="item in selected.installations" :key="item.root_id+item.path" class="skills-root"><div><strong>{{ rootName(item.root_id) }}</strong><span class="skills-badge">{{ item.status }}</span><p class="skills-path">{{ item.path }}</p><p v-if="item.source" class="muted">{{ item.source.repo }} / {{ item.source.directory }} · 当前 {{ item.source.commit.slice(0,12) }}</p><p v-if="item.checked_at" class="muted">上次检查 {{ date(item.checked_at) }} · 上游 {{ item.upstream_commit?.slice(0,12) }}</p></div><div class="skills-actions"><button :disabled="busy" @click="openFolder(item.path)">打开文件夹</button><button :disabled="busy" @click="viewInstallation(item)">查看此副本</button><button v-if="item.source && !item.readonly" :disabled="busy" @click="checkUpdate(item)">检查更新</button><button v-if="!item.readonly" :disabled="busy" @click="askRemove(item)">移除</button></div></article></section>
      </template>
      <template v-else-if="mode === 'local'">
        <div class="skills-filters"><label class="skills-search"><AppIcon name="search"/><input v-model="query" aria-label="搜索本机 Skills" placeholder="搜索名称、描述或来源…"/></label><SearchableSelect v-model="agentFilter" :options="[{value:'',label:'全部智能体'},...agents]"/><SearchableSelect v-model="scopeFilter" :options="[{value:'',label:'全部范围'},{value:'global',label:'全局'},{value:'project',label:'项目'}]"/><SearchableSelect v-model="statusFilter" :options="[{value:'',label:'全部状态'},{value:'managed',label:'CueTuck 管理'},{value:'readonly',label:'只读 / 外部链接'},{value:'external',label:'外部安装'}]"/><SearchableSelect v-model="sourceFilter" :options="[{value:'',label:'全部来源'},{value:'github',label:'GitHub'},{value:'local',label:'本地导入'},{value:'cache',label:'插件缓存'},{value:'external',label:'外部来源'}]"/></div>
        <details v-if="snapshot.warnings.length" class="skills-warnings"><summary>{{ snapshot.warnings.length }} 条扫描提示</summary><p v-for="warning in snapshot.warnings" :key="warning">{{ warning }}</p></details>
        <section class="skills-list" :aria-busy="busy"><button v-for="skill in visibleLocal" :key="skill.key" class="skills-list-row" :disabled="busy" @click="openDetail(skill)"><span class="skills-list-icon"><AppIcon name="skills"/></span><span class="skills-row-main"><strong>{{ skill.name }}</strong><span>{{ skill.description || '未声明描述，打开查看说明与文件' }}</span></span><span class="skills-row-locations">{{ [...new Set(skill.installations.map(i=>`${rootName(i.root_id)} · ${scopeName(snapshot.roots.find(r=>r.id===i.root_id)?.scope)}`))].join(' · ') }}<small>{{ skill.installations.length }} 处安装</small></span><span class="skills-badge">{{ skill.source ? 'GitHub' : skill.status }}</span><span aria-hidden="true">›</span></button></section>
        <div v-if="filtered.length > pageSize" class="skills-pagination"><button :disabled="localPage===0" @click="localPage--">上一页</button><span>{{ localPage+1 }} / {{ Math.ceil(filtered.length/pageSize) }}</span><button :disabled="(localPage+1)*pageSize>=filtered.length" @click="localPage++">下一页</button></div>
        <section v-if="!filtered.length && !busy" class="skills-empty"><AppIcon name="skills"/><h2>{{ query || agentFilter || scopeFilter || statusFilter || sourceFilter ? '没有匹配的 Skill' : '尚未发现本机 Skill' }}</h2><p>可以登记项目目录，或从公开来源安装。</p><button class="button" @click="directories=true">管理目录</button></section>
      </template>
      <template v-else>
        <section class="skills-panel skills-source-panel"><div class="skills-section-heading"><h2>公开来源 <small>{{ sourceOptions.length }} 个</small></h2><span class="skills-badge">GitHub · 无需登录</span></div><div class="skills-source-controls"><SearchableSelect v-model="sourceInput" :options="sourceOptions" :disabled="busy" aria-label="选择 Skill 来源"/><button class="button" :disabled="busy" @click="loadCatalog">加载来源</button><button class="button" :disabled="busy" @click="sourceForm=!sourceForm">添加来源 / 链接</button></div><div v-if="sourceForm" class="skills-source-controls"><input v-model="sourceDraft" aria-label="GitHub 仓库或目录地址" placeholder="owner/repo 或 GitHub tree / SKILL.md 地址" @keydown.enter="addSource"/><button class="button" :disabled="busy || !sourceDraft.trim()" @click="addSource">保存并浏览</button></div><p class="muted">{{ catalog ? `${catalog.repo} · ${catalog.reference} · ${catalog.commit.slice(0,12)} · ${catalog.entries.length} 个 Skill` : '选择来源后加载真实目录列表。' }}</p><p class="muted">切换上方来源可发现更多 Skill，支持按用途或仓库名搜索来源。下方列表只搜索当前来源的名称、路径及已加载简介；打开后查看说明、依赖和许可。</p><button v-if="snapshot.sources?.includes(sourceInput)" class="skills-inline-button" :disabled="busy" @click="forgetSource">移除此来源</button></section>
        <label class="skills-search skills-remote-search"><AppIcon name="search"/><input v-model="remoteQuery" placeholder="搜索当前来源的 Skill 名称或路径…" aria-label="搜索当前来源"/></label>
        <section class="skills-list"><button v-for="entry in visibleRemote" :key="entry.directory" class="skills-list-row" :disabled="busy" @click="openRemote(entry)"><span class="skills-list-icon"><AppIcon name="skills"/></span><span class="skills-row-main"><strong>{{ entry.name }}</strong><span>{{ entry.description || (entry.error ? `简介读取失败：${entry.error}` : '打开查看完整说明') }}</span><small class="skills-path">{{ entry.directory || '仓库根目录' }}</small></span><span class="skills-badge">{{ snapshot.skills.some(s=>s.source?.repo.toLowerCase()===catalog.repo.toLowerCase()&&s.source?.directory===entry.directory)?'已安装':'查看并安装' }}</span><span aria-hidden="true">›</span></button></section>
        <div v-if="remoteFiltered.length>pageSize" class="skills-pagination"><button :disabled="remotePage===0" @click="remotePage--">上一页</button><span>{{ remotePage+1 }} / {{ Math.ceil(remoteFiltered.length/pageSize) }}</span><button :disabled="(remotePage+1)*pageSize>=remoteFiltered.length" @click="remotePage++">下一页</button></div><section v-if="catalog && !remoteFiltered.length && !busy" class="skills-empty"><h2>没有匹配的 Skill</h2><p>当前目录可能没有 SKILL.md，或搜索条件不匹配。</p></section>
      </template>
      <div v-if="installOpen" class="skills-overlay" @keydown.esc.stop="closeInstall" @click.self="closeInstall">
        <section ref="installDialog" class="skills-modal" role="dialog" aria-modal="true" aria-labelledby="skills-install-title" tabindex="-1" @keydown.tab="trapFocus">
          <header class="skills-section-heading"><div><h2 id="skills-install-title">安装 {{ prepared.package.name }}</h2><p class="muted">创建独立副本；{{ formatBytes(prepared.package.bytes) }} · {{ prepared.package.files.length }} 个文件</p><p class="skills-path">{{ prepared.source ? `${prepared.source.repo} / ${prepared.source.directory} · ${prepared.source.commit.slice(0,12)}` : `本地来源 · ${prepared.local_path}` }}</p></div><button :disabled="busy" aria-label="关闭安装预览" @click="closeInstall">×</button></header>
          <p v-if="error" class="skills-alert" role="alert">{{ error }}</p><p v-if="busy" role="status" class="muted">{{ loadingText }}</p>
          <template v-if="!results.length">
            <label class="skills-confirm-check" v-if="prepared.package.warnings.length"><input type="checkbox" v-model="dependenciesAccepted" :disabled="busy"/>我已阅读说明中的外部依赖提示；依赖需在目标客户端单独配置。</label>
            <div class="skills-source-controls"><SearchableSelect v-model="targetScope" :options="[{value:'global',label:'全局目录'},{value:'project',label:'项目目录'}]"/><button class="button" :disabled="busy" @click="addProject">添加项目</button></div>
            <p class="muted">选择需要安装的位置。共享目录可能被多个客户端共同读取；同一路径只写一次。</p>
            <label v-for="root in targetRoots" :key="root.id" class="skills-target"><input type="checkbox" :value="root.id" v-model="targetIds" :disabled="busy || root.readonly"/><span><strong>{{ root.name }}</strong><span v-if="root.readonly" class="skills-badge">只读</span><span class="skills-path">{{ root.path }}/{{ prepared.folder_name }}</span><small v-if="root.shared_with.length>1">可能被 {{ root.shared_with.join('、') }} 共同读取</small></span></label>
            <p v-if="!targetRoots.length" class="muted">请先添加项目或登记目录。</p>
            <article v-for="plan in plans" :key="plan.root_id" class="skills-plan"><strong>{{ rootName(plan.root_id) }}</strong><p class="skills-path">{{ plan.target }}</p><p :class="plan.status==='blocked'?'skills-alert':'muted'">{{ plan.message }}</p><p v-if="plan.current_source" class="muted">现有来源 {{ plan.current_source.repo }} / {{ plan.current_source.directory }} · {{ plan.current_source.commit.slice(0,12) }}</p><details v-if="plan.changes.length"><summary>查看 {{ plan.changes.length }} 项文件变化</summary><pre class="skills-diff">{{ plan.changes.join('\n') }}</pre></details><button v-if="plan.current_digest" class="skills-inline-button" :disabled="busy" @click="compareBody(plan)">查看正文对比</button><div v-if="comparison[plan.root_id]" class="skills-body-compare"><section><h3>当前副本</h3><pre class="skills-diff">{{ comparison[plan.root_id] }}</pre></section><section><h3>准备安装</h3><pre class="skills-diff">{{ prepared.package.body }}</pre></section></div><label v-if="['conflict','modified'].includes(plan.status)" class="skills-confirm-check"><input type="checkbox" v-model="replaceIds" :value="plan.root_id" :disabled="busy"/>先保存完整备份，再替换此位置</label></article>
          </template>
          <template v-else><article v-for="result in results" :key="result.target" class="skills-plan"><strong>{{ result.status==='success'?'安装完成':'安装失败' }}</strong><p class="skills-path">{{ result.target }}</p><p class="muted">{{ result.message }}</p></article></template>
          <footer class="skills-modal-footer"><button class="button" :disabled="busy" @click="closeInstall">{{ results.length ? '完成' : '取消' }}</button><button v-if="!results.length && !plans.length" class="button primary-button" :disabled="busy || !targetIds.length" @click="preflight">比较并预览</button><button v-if="!results.length && plans.length" class="button" :disabled="busy" @click="preflight">重新比较</button><button v-if="!results.length && plans.length" class="button primary-button" :disabled="!canInstall" @click="install">确认安装</button><button v-if="results.some(r=>r.status==='failed')" class="button primary-button" :disabled="busy" @click="retryFailed">仅重试失败位置</button></footer>
        </section>
      </div>
      <div v-if="confirmation" class="skills-overlay" @click.self="!busy && (confirmation=null)" @keydown.esc.stop="!busy && (confirmation=null)"><section ref="confirmDialog" class="skills-modal skills-confirm-modal" role="dialog" aria-modal="true" aria-labelledby="skills-confirm-title" tabindex="-1" @keydown.tab="trapFocus"><h2 id="skills-confirm-title">{{ confirmation.kind==='remove'?'备份并移除此安装？':'恢复到原位置？' }}</h2><p class="skills-path">{{ confirmation.item?.path || confirmation.backup.original }}</p><p class="muted">{{ confirmation.kind==='remove'?'只移除此位置。会保存完整备份，其他独立副本不受影响；共享目录的所有读取客户端会同时失去此 Skill。':'恢复会保留备份。原位置已有内容时会停止，不会覆盖。' }}</p><p v-if="error" class="skills-alert" role="alert">{{ error }}</p><footer class="skills-modal-footer"><button class="button" :disabled="busy" @click="confirmation=null">取消</button><button class="button primary-button" :disabled="busy" @click="confirmMaintenance">{{ busy?'正在处理…':confirmation.kind==='remove'?'备份并移除':'恢复' }}</button></footer></section></div>
    </template>
  </main>
</template>
<script setup>
import {computed,nextTick,onMounted,onUnmounted,ref,watch} from 'vue';
import AppIcon from './AppIcon.vue';
import SearchableSelect from './SearchableSelect.vue';
import builtInSources from '../data/skill-sources.json';
import {skillsSupported,skillsRequest,chooseSkillsDirectory,cancelSkillsRequest,listenSkillsProgress,groupSkills} from '../platform/skills.js';
const props=defineProps({mode:{type:String,default:'local'}}),emit=defineEmits(['busy']);
const supported=skillsSupported(),snapshot=ref({roots:[],skills:[],warnings:[],backups:[],operations:[],sources:[]});
const busy=ref(false),error=ref(''),notice=ref(''),loadingText=ref('正在读取…'),progress=ref(null),networkBusy=ref(false);
const query=ref(''),agentFilter=ref(''),scopeFilter=ref(''),statusFilter=ref(''),sourceFilter=ref(''),directories=ref(false),maintenance=ref(false),detail=ref(null),selected=ref(null),selectedFileText=ref(null),selectedFileName=ref(''),page=ref(null);
const rootDraft=ref(emptyRoot()),prepared=ref(null),installOpen=ref(false),installDialog=ref(null),confirmDialog=ref(null),confirmation=ref(null);
const targetIds=ref([]),targetScope=ref('global'),plans=ref([]),replaceIds=ref([]),results=ref([]),comparison=ref({}),dependenciesAccepted=ref(false);
const sourceInput=ref('anthropics/skills'),sourceDraft=ref(''),sourceForm=ref(false),catalog=ref(null),remoteQuery=ref(''),remotePage=ref(0),localPage=ref(0),pageSize=50;
const agents=[{value:'shared',label:'Codex / 标准共享'},{value:'codex',label:'Codex'},{value:'claude',label:'Claude Code'},{value:'cursor',label:'Cursor'},{value:'pi',label:'Pi'},{value:'opencode',label:'OpenCode'},{value:'custom',label:'自定义'}];
const sourceOptions=computed(()=>{
  const options=new Map(builtInSources.map(s=>[s.value.toLowerCase(),s]));
  for(const value of snapshot.value.sources||[])if(!options.has(value.toLowerCase()))options.set(value.toLowerCase(),{value,label:value});
  return [...options.values()];
});
const grouped=computed(()=>groupSkills(snapshot.value.skills));
const filtered=computed(()=>grouped.value.map(s=>({...s,installations:s.installations.filter(i=>{
  const root=snapshot.value.roots.find(r=>r.id===i.root_id);
  return (!agentFilter.value || root?.agent===agentFilter.value || root?.shared_with.includes(agents.find(a=>a.value===agentFilter.value)?.label))&&(!scopeFilter.value||root?.scope===scopeFilter.value)&&(!statusFilter.value || (statusFilter.value==='readonly'?i.readonly:statusFilter.value==='managed'?Boolean(i.installed_digest):!i.readonly&&!i.installed_digest));
})})).filter(s=>s.installations.length&&(!sourceFilter.value||(sourceFilter.value==='github'?s.source:sourceFilter.value==='cache'?s.installations.some(i=>i.root_id.includes('cache')):sourceFilter.value==='local'?s.installations.some(i=>i.installed_digest&&!i.source):s.installations.some(i=>!i.installed_digest)))&&`${s.name} ${s.description} ${s.source?.repo||''}`.toLowerCase().includes(query.value.trim().toLowerCase())));
const visibleLocal=computed(()=>filtered.value.slice(localPage.value*pageSize,(localPage.value+1)*pageSize));
const remoteFiltered=computed(()=>(catalog.value?.entries||[]).filter(e=>`${e.name} ${e.directory} ${e.description||''}`.toLowerCase().includes(remoteQuery.value.trim().toLowerCase())));
const visibleRemote=computed(()=>remoteFiltered.value.slice(remotePage.value*pageSize,(remotePage.value+1)*pageSize));
const targetRoots=computed(()=>snapshot.value.roots.filter(r=>r.scope===targetScope.value));
const canInstall=computed(()=>!busy.value&&plans.value.length>0&&(!prepared.value?.package.warnings.length||dependenciesAccepted.value)&&plans.value.some(p=>p.status!=='blocked')&&plans.value.every(p=>p.status==='blocked'||!['conflict','modified'].includes(p.status)||replaceIds.value.includes(p.root_id)));
let requestId='',disposed=false,scrollTop=0,lastRequest=null,lastApply=null,unlisten=()=>{},returnFocus=null;
function emptyRoot(){return {id:'',name:'',agent:'custom',scope:'global',path:'',readonly:false,custom:true,shared_with:[],status:''};}
async function run(request,apply,message='正在读取…'){
  if(busy.value)return;const id=crypto.randomUUID();requestId=id;busy.value=true;error.value='';loadingText.value=message;progress.value=null;
  networkBusy.value=['catalog','descriptions','prepare_remote','check_update'].includes(request.action);
  lastRequest=request;lastApply=apply;
  try{const result=await skillsRequest(request,id);if(!disposed&&id===requestId)await apply?.(result);return result;}
  catch(e){if(!disposed&&id===requestId)error.value=String(e.message||e);}
  finally{if(id===requestId){busy.value=false;networkBusy.value=false;}}
}
async function retryRequest(){if(lastRequest)await run(lastRequest,lastApply);}
async function cancel(){try{await cancelSkillsRequest(requestId);}catch(e){error.value=String(e.message||e);}}
async function refresh(){if(!supported)return;await run({action:'snapshot'},data=>{snapshot.value=data;localPage.value=Math.min(localPage.value,Math.max(0,Math.ceil(filtered.value.length/pageSize)-1));},'正在扫描已登记目录…');}
function rootName(id){return snapshot.value.roots.find(r=>r.id===id)?.name||id;}
function scopeName(scope){return scope==='project'?'项目':'全局';}
function formatBytes(n){return n<1024?`${n} B`:n<1024*1024?`${(n/1024).toFixed(1)} KB`:`${(n/1024/1024).toFixed(1)} MB`;}
function date(n){return n?new Date(n*1000).toLocaleString():'尚未检查';}
function showPrepared(p){prepared.value=p;selected.value={path:p.local_path||'',installations:[]};detail.value={package:p.package,modified:false};selectedFileText.value=null;selectedFileName.value='';directories.value=false;maintenance.value=false;page.value?.scrollTo?.(0,0);}
async function releasePrepared(){const old=prepared.value;prepared.value=null;if(old){try{await skillsRequest({action:'discard',id:old.id});}catch{/* A temporary preview can be removed on a later cleanup. */}}}
async function openDetail(skill){scrollTop=page.value?.scrollTop||0;await releasePrepared();const path=skill.installations[0].path;await run({action:'detail',path},data=>{selected.value={...skill,path};detail.value=data;selectedFileText.value=null;selectedFileName.value='';page.value?.scrollTo?.(0,0);});}
async function viewInstallation(item){await releasePrepared();await run({action:'detail',path:item.path},data=>{selected.value={...selected.value,path:item.path};detail.value=data;selectedFileText.value=null;selectedFileName.value='';});}
async function back(){if(busy.value)return;await releasePrepared();directories.value=false;maintenance.value=false;detail.value=null;selected.value=null;selectedFileText.value=null;await nextTick();if(page.value)page.value.scrollTop=scrollTop;}
async function readFile(file){await run(prepared.value?{action:'read_prepared',id:prepared.value.id,file}:{action:'read_file',path:selected.value.path,file},text=>{selectedFileText.value=text;selectedFileName.value=file;});}
async function openFolder(path){await run({action:'open_directory',path});}
async function pickRoot(){try{const path=await chooseSkillsDirectory();if(path)rootDraft.value.path=path;}catch(e){error.value=String(e.message||e);}}
function resetRoot(){rootDraft.value=emptyRoot();}
function editRoot(root){rootDraft.value={...root};page.value?.scrollTo?.(0,0);}
async function saveRoot(){const result=await run({action:'save_root',root:rootDraft.value});if(result){resetRoot();notice.value='目录已登记';await refresh();}}
async function addProject(){try{const path=await chooseSkillsDirectory();if(!path)return;const result=await run({action:'add_project',path});if(result){notice.value='已登记五个客户端的项目目录；扫描不会创建或修改文件。';await refresh();targetScope.value='project';}}catch(e){error.value=String(e.message||e);}}
async function forgetRoot(root){const result=await run({action:'forget_root',id:root.id});if(result!==undefined){notice.value='已取消扫描登记，目录文件和恢复备份保留。';await refresh();}}
async function importLocal(){try{const path=await chooseSkillsDirectory();if(!path)return;await releasePrepared();scrollTop=page.value?.scrollTop||0;await run({action:'prepare_local',path},showPrepared,'正在校验并暂存完整 Skill 文件包…');}catch(e){error.value=String(e.message||e);}}
async function loadCatalog(){await run({action:'catalog',input:sourceInput.value},data=>{catalog.value=data;remotePage.value=0;remoteQuery.value='';},'正在读取 GitHub 固定版本目录…');await loadDescriptions();}
async function loadDescriptions(){if(busy.value||!catalog.value||detail.value)return;const entries=visibleRemote.value.filter(e=>!e.loaded&&!e.error);if(!entries.length)return;const source={...catalog.value,directory:''};await run({action:'descriptions',source,entries},data=>{if(!data)return;for(const entry of data){const index=catalog.value.entries.findIndex(e=>e.directory===entry.directory);if(index>=0)catalog.value.entries[index]=entry;}},'正在读取当前页的真实简介…');}
async function addSource(){const input=sourceDraft.value.trim();const result=await run({action:'save_source',input,remove:false},data=>snapshot.value.sources=data);if(result){sourceInput.value=input;sourceDraft.value='';sourceForm.value=false;await loadCatalog();}}
async function forgetSource(){await run({action:'save_source',input:sourceInput.value,remove:true},data=>snapshot.value.sources=data);}
async function openRemote(entry){await releasePrepared();scrollTop=page.value?.scrollTop||0;const source={repo:catalog.value.repo,reference:catalog.value.reference,commit:catalog.value.commit,license:catalog.value.license,directory:entry.directory};await run({action:'prepare_remote',source},showPrepared,'正在下载预览文件包…');}
async function beginInstall(){if(!prepared.value){const result=await run({action:'prepare_local',path:selected.value.path},p=>prepared.value=p,'正在校验待复制文件…');if(!result)return;}returnFocus=document.activeElement;targetIds.value=[];plans.value=[];replaceIds.value=[];results.value=[];dependenciesAccepted.value=false;installOpen.value=true;await nextTick();installDialog.value?.focus();}
async function compareBody(plan){await run({action:'read_file',path:plan.target,file:'SKILL.md'},text=>comparison.value[plan.root_id]=text);}
async function preflight(){comparison.value={};await run({action:'preflight',id:prepared.value.id,root_ids:targetIds.value},data=>{plans.value=data;replaceIds.value=[];},'正在比较目标文件…');await nextTick();installDialog.value?.focus();installDialog.value?.querySelector('.skills-plan')?.scrollIntoView?.({block:'nearest'});}
async function install(){if(!canInstall.value)return;const selections=plans.value.map(p=>({root_id:p.root_id,expected_digest:p.current_digest,replace:replaceIds.value.includes(p.root_id)}));await run({action:'install',id:prepared.value.id,selections},data=>results.value=data,'正在逐位置安装，请保持窗口打开…');await refresh();await nextTick();installDialog.value?.focus();}
async function retryFailed(){const failed=new Set(results.value.filter(r=>r.status==='failed').map(r=>r.target));const ids=plans.value.filter(p=>failed.has(p.target)).map(p=>p.root_id);results.value=[];targetIds.value=ids;await preflight();}
async function closeInstall(){if(busy.value)return;installOpen.value=false;await nextTick();returnFocus?.focus?.();}
async function askRemove(item){const data=await run({action:'detail',path:item.path});if(data){returnFocus=document.activeElement;confirmation.value={kind:'remove',item,digest:data.package.digest};}}
async function confirmMaintenance(){const pending=confirmation.value;const request=pending.kind==='remove'?{action:'remove',path:pending.item.path,root_id:pending.item.root_id,expected_digest:pending.digest}:{action:'restore',id:pending.backup.id};const result=await run(request,null,pending.kind==='remove'?'正在备份并移除…':'正在校验并恢复…');if(result!==undefined){confirmation.value=null;notice.value=pending.kind==='remove'?'已移除此位置，完整备份可在“备份与记录”恢复。':'已恢复到原位置。';await back();maintenance.value=true;await refresh();}}
async function checkUpdate(item){await releasePrepared();const p=await run({action:'check_update',path:item.path},null,'正在检查上游并下载用于比较的版本…');if(!p)return;const unchanged=p.package.digest===item.installed_digest;showPrepared(p);notice.value=unchanged?'上游与安装时内容一致；仍可比较本地修改。':`已获取上游 ${p.source.commit.slice(0,12)}，请比较后确认更新。`;await beginInstall();targetScope.value=snapshot.value.roots.find(r=>r.id===item.root_id)?.scope||'global';targetIds.value=[item.root_id];await preflight();}
function trapFocus(event){const elements=[...event.currentTarget.querySelectorAll('button:not(:disabled),input:not(:disabled),select:not(:disabled),[tabindex="0"]')].filter(e=>e.getClientRects().length);const first=elements[0],last=elements.at(-1);if(!first){event.preventDefault();return;}if(event.shiftKey&&(document.activeElement===first||document.activeElement===event.currentTarget)){event.preventDefault();last.focus();}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}}
watch([query,agentFilter,scopeFilter,statusFilter,sourceFilter],()=>localPage.value=0);
watch(remoteQuery,()=>{remotePage.value=0;});
watch(remotePage,()=>loadDescriptions());
watch(targetIds,()=>{plans.value=[];replaceIds.value=[];},{deep:true});
watch([busy,installOpen,confirmation],()=>emit('busy',busy.value||installOpen.value||Boolean(confirmation.value)),{flush:'sync'});
watch(confirmation,async value=>{if(value){returnFocus=document.activeElement;await nextTick();confirmDialog.value?.focus();}else{await nextTick();returnFocus?.focus?.();}});
watch(()=>props.mode,async()=>{if(!busy.value){await back();error.value='';if(props.mode==='square'&&!catalog.value)await loadCatalog();}});
onMounted(async()=>{unlisten=await listenSkillsProgress(p=>{if(p.request_id===requestId)progress.value=p;});if(disposed){unlisten();return;}await refresh();if(props.mode==='square')await loadCatalog();});
onUnmounted(()=>{disposed=true;unlisten();cancelSkillsRequest(requestId).catch(()=>{});if(prepared.value)skillsRequest({action:'discard',id:prepared.value.id}).catch(()=>{});});
function focusSearch(){page.value?.querySelector('.skills-search input')?.focus();}
defineExpose({busy,focusSearch});
</script>

<style scoped>
.skills-page { padding:28px 32px 48px; overflow:auto; background:var(--surface); }
.skills-header { display:flex; align-items:flex-start; justify-content:space-between; gap:24px; margin-bottom:24px; }
.skills-header h1 { margin:0 0 8px; font-size:26px; letter-spacing:-.5px; }
.skills-header p { margin:0; font-size:13px; line-height:1.7; }
.skills-header .skills-eyebrow { font-size:10px; letter-spacing:1.6px; margin-bottom:6px; color:var(--muted); }
.muted { color:var(--muted); font-size:12px; line-height:1.65; }
.skills-actions { display:flex; align-items:center; flex-wrap:wrap; gap:8px; }
.skills-page button { cursor:pointer; }
.skills-page button:disabled { opacity:.5; cursor:default; }
.skills-page button:focus-visible { outline:2px solid var(--text); outline-offset:2px; }
.skills-back { border:0; background:transparent; color:var(--muted); padding:0; margin-bottom:20px; }
.skills-panel { padding:20px; border:1px solid var(--line); border-radius:12px; margin-bottom:20px; min-width:0; }
.skills-panel h2 { font-size:15px; margin:0 0 18px; }
.skills-panel h2 small { color:var(--muted); font-weight:400; margin-left:6px; }
.skills-form { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:14px; margin-bottom:16px; }
.skills-form label { display:flex; flex-direction:column; gap:7px; color:var(--muted); font-size:12px; }
.skills-form .wide { grid-column:1/-1; }
.skills-form input,.skills-search input { color:var(--text); background:var(--surface); border:1px solid var(--line); border-radius:8px; padding:10px 12px; font:inherit; min-width:0; }
.skills-path-input { display:flex; gap:8px; }
.skills-path-input input { flex:1; }
.skills-root { display:flex; align-items:center; justify-content:space-between; gap:16px; padding:16px 0; border-top:1px solid var(--line); }
.skills-root>div:first-child { min-width:0; }
.skills-root p { margin:6px 0 0; }
.skills-root strong { font-size:13px; }
.skills-root button,.skills-path-input button { color:var(--text); background:transparent; border:1px solid var(--line); border-radius:7px; padding:7px 10px; white-space:nowrap; font-size:12px; }
.skills-path { font:11px/1.7 var(--font-code); overflow-wrap:anywhere; color:var(--muted); }
.skills-badge { display:inline-block; font-size:10px; color:var(--muted); background:var(--sidebar); border-radius:5px; padding:4px 7px; margin-left:8px; white-space:nowrap; }
.skills-filters { display:grid; grid-template-columns:minmax(200px,1fr) 150px 110px 140px 120px; gap:10px; margin-bottom:20px; }
.skills-search { display:flex; align-items:center; gap:8px; border:1px solid var(--line); border-radius:8px; padding:0 12px; color:var(--muted); }
.skills-search input { width:100%; border:0; padding:10px 0; outline:none; }
.skills-list { border:1px solid var(--line); border-radius:12px; overflow:hidden; }
.skills-list:empty { display:none; }
.skills-list-row { width:100%; display:flex; align-items:center; gap:16px; text-align:left; padding:18px; border:0; border-bottom:1px solid var(--line); color:var(--text); background:transparent; }
.skills-list-row:last-child { border-bottom:0; }
.skills-list-row:hover { background:var(--sidebar); }
.skills-list-icon { width:38px; height:38px; display:grid; place-items:center; background:var(--sidebar); border-radius:10px; flex-shrink:0; }
.skills-row-main { flex:1; min-width:0; display:grid; gap:5px; }
.skills-row-main strong { font-size:14px; overflow-wrap:anywhere; }
.skills-row-main>span { color:var(--muted); font-size:12px; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden; }
.skills-row-locations { max-width:220px; font-size:11px; color:var(--muted); }
.skills-row-locations small { display:block; margin-top:5px; }
.skills-empty { text-align:center; padding:72px 24px; color:var(--muted); }
.skills-empty>.app-icon { width:36px; height:36px; margin-bottom:12px; }
.skills-empty h1,.skills-empty h2 { color:var(--text); font-size:20px; }
.skills-alert,.skills-notice,.skills-warnings,.skills-warning { padding:12px 14px; border-radius:8px; font-size:12px; line-height:1.7; overflow-wrap:anywhere; background:var(--sidebar); }
.skills-alert { border:1px solid var(--danger,#ad5555); }
.skills-alert button,.skills-notice button { float:right; background:transparent; color:inherit; border:0; }
.skills-loading { font-size:12px; color:var(--muted); }
.skills-detail-grid { display:grid; grid-template-columns:minmax(0,1fr) 280px; gap:20px; }
.skills-section-heading { display:flex; justify-content:space-between; align-items:baseline; }
.skills-document { white-space:pre-wrap; overflow-wrap:anywhere; font:12px/1.85 var(--font-code); max-height:60vh; overflow:auto; margin-bottom:0; }
.skills-file { display:flex; align-items:baseline; gap:8px; width:100%; text-align:left; color:var(--text); background:transparent; border:0; border-radius:5px; padding:8px 4px; font-size:11px; }
.skills-file:hover { background:var(--sidebar); }
.skills-file span { flex:1; overflow-wrap:anywhere; }
.skills-file small { white-space:nowrap; color:var(--muted); }
@media(max-width:1100px){.skills-filters{grid-template-columns:1fr 1fr}.skills-search{grid-column:1/-1}.skills-detail-grid{grid-template-columns:1fr}.skills-row-locations{max-width:150px}}
@media(max-width:700px){.skills-page{padding:24px 18px}.skills-header,.skills-root{flex-direction:column}.skills-form{grid-template-columns:1fr}.skills-list-row{flex-wrap:wrap;gap:10px}.skills-row-main{min-width:65%}.skills-row-locations{margin-left:48px}}
.skills-source-controls { display:flex; gap:10px; align-items:center; margin:12px 0; }
.skills-source-controls>:first-child { flex:1; min-width:0; }
.skills-source-controls input { padding:11px; border:1px solid var(--line); border-radius:8px; background:var(--surface); color:var(--text); font:inherit; }
.skills-remote-search { margin-bottom:18px; }
.skills-inline-button { border:0; background:transparent; color:var(--text); text-decoration:underline; margin-left:10px; font:inherit; }
.skills-pagination { display:flex; justify-content:center; align-items:center; gap:18px; padding:20px; color:var(--muted); font-size:12px; }
.skills-pagination button { border:1px solid var(--line); background:var(--surface); color:var(--text); border-radius:7px; padding:7px 12px; }
.skills-overlay { position:fixed; inset:0; z-index:1200; background:rgba(0,0,0,.36); display:flex; align-items:center; justify-content:center; padding:24px; }
.skills-modal { width:min(760px,100%); max-height:calc(100dvh - 48px); overflow:auto; background:var(--surface); color:var(--text); border:1px solid var(--line); border-radius:16px; padding:24px; box-shadow:0 24px 80px rgba(0,0,0,.2); }
.skills-modal h2 { font-size:20px; margin:0 0 10px; }
.skills-modal .skills-section-heading>button { border:0; background:transparent; color:var(--muted); font-size:24px; }
.skills-modal-footer { display:flex; justify-content:flex-end; gap:10px; position:sticky; bottom:-24px; background:var(--surface); border-top:1px solid var(--line); margin:20px -24px -24px; padding:18px 24px; }
.skills-target { display:flex; gap:12px; align-items:flex-start; padding:13px 0; border-bottom:1px solid var(--line); font-size:12px; }
.skills-target>span { min-width:0; }
.skills-target .skills-path { display:block; }
.skills-target small { display:block; color:var(--muted); margin-top:4px; }
.skills-target input,.skills-confirm-check input { accent-color:var(--text); margin-top:3px; flex-shrink:0; }
.skills-plan { padding:16px 0; border-bottom:1px solid var(--line); font-size:12px; }
.skills-plan p { margin:7px 0; }
.skills-diff { white-space:pre-wrap; overflow-wrap:anywhere; max-height:200px; overflow:auto; font:11px/1.7 var(--font-code); }
.skills-confirm-check { display:flex; gap:8px; align-items:flex-start; font-size:12px; line-height:1.7; padding:12px; margin:10px 0; background:var(--sidebar); border-radius:8px; }
.skills-body-compare { display:grid; grid-template-columns:1fr 1fr; gap:12px; }
.skills-body-compare>section { min-width:0; }
.skills-body-compare h3 { font-size:12px; }
.skills-confirm-modal { max-width:520px; }
.skills-detail-grid aside .skills-panel:first-child { max-height:65vh; overflow:auto; }
@media(max-width:700px){.skills-source-controls{flex-wrap:wrap}.skills-source-controls>:first-child{flex-basis:100%}.skills-overlay{padding:12px}.skills-modal{max-height:calc(100dvh - 24px);padding:18px}.skills-modal-footer{margin:18px -18px -18px;padding:16px 18px;bottom:-18px}}
</style>
