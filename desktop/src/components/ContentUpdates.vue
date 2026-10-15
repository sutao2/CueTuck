<template>
 <section v-page-focus="close" class="workspace-page" aria-labelledby="updates-title">
  <header class="modal-header"><h2 id="updates-title">{{ tr("下载内容更新") }}</h2><button class="page-back" :disabled="busy" @click="close">{{ tr("← 返回") }}</button></header>
  <div class="create-body">
   <p>{{ tr("更新前可比较本地与广场版本。替换会先完整备份；旧下载版本未知时，请另存新版。") }}</p>
   <p v-if="error" role="alert">{{ tr(error) }}</p><p v-if="notice" role="status">{{ tr(notice) }}</p>
   <nav class="tabs"><button class="button" :class="{'primary-button':tab==='downloads'}" :disabled="busy" @click="tab='downloads';selected=null;load()">{{ tr("下载内容") }}</button><button class="button" :class="{'primary-button':tab==='backups'}" :disabled="busy" @click="tab='backups';selected=null;load()">{{ tr("更新前备份") }}</button></nav>
   <p v-if="busy" role="status">{{ tr("正在处理…") }}</p>
   <template v-if="selected">
    <h3>{{ selected.title }}</h3>
    <p>{{ tr("本地") }} {{ selected.version ? `v${selected.version}` : tr('版本未知') }} {{ tr("→ 广场 v") }}{{ latest?.version }} · {{ latest?.change_note || tr('首次发布') }}</p>
    <p v-if="preview?.modified" class="warning">{{ tr("本地存在修改。选择“保留本地并另存新版”可以保留两份；“备份后替换”会明确覆盖本地修改。") }}</p>
    <p v-if="!preview?.origin" class="warning">{{ tr("没有可靠的下载基准，不能直接覆盖。") }}</p>
    <div v-if="latest" class="comparison"><section><h4>{{ tr("当前本地") }}</h4><pre>{{ readable(preview.local) }}</pre></section><section><h4>{{ tr("广场新版") }}</h4><pre>{{ readableRemote(latest.snapshot) }}</pre></section></div>
    <details v-if="preview?.origin"><summary>{{ tr("查看下载时的原始版本") }}</summary><pre>{{ readableRemote(preview.origin.remote) }}</pre></details>
    <div class="actions"><button class="button" :disabled="busy" @click="selected=null">{{ tr("保留本地") }}</button><button class="button" :disabled="busy||!latest" @click="apply(false)">{{ tr("保留本地并另存新版") }}</button><button class="button primary-button" :disabled="busy||!canReplace" @click="confirmReplace=true">{{ tr("备份后替换") }}</button></div>
    <div v-if="confirmReplace" class="confirm"><p>{{ tr("确认用广场 v") }}{{ latest.version }} {{ tr("替换当前内容？当前完整内容会保存在“更新前备份”，可恢复为副本。") }}</p><button class="button" :disabled="busy" @click="confirmReplace=false">{{ tr("取消") }}</button><button class="button primary-button" :disabled="busy" @click="apply(true)">{{ tr("确认备份并替换") }}</button></div>
   </template>
   <template v-else>
    <p v-if="!busy&&!rows.length">{{ tab==='backups' ? tr('还没有更新前备份。') : tr('没有可检查的下载内容。社区 Skill 请在对应安装位置检查更新。') }}</p>
    <article v-for="row in rows" :key="row.local_id||row.id" class="update-row"><div><strong>{{ row.title }}</strong><p>{{ row.kind==='collection'?tr('合集'):tr('提示词') }} · {{ tab==='backups' ? new Date(Number(row.saved_at)).toLocaleString() : row.version ? `已下载 v${row.version}` : tr('版本未知') }}</p></div><button class="button" :disabled="busy" @click="tab==='backups'?restore(row):check(row)">{{ tab==='backups'?tr('恢复为副本'):tr('检查更新') }}</button></article>
   </template>
  </div>
 </section>
</template>
<script setup>
import { tr } from '../platform/interfaceLanguage.js';
import { computed, ref, onMounted } from 'vue';
import { vPageFocus } from '../lib/pageFocus.js';
import { contentVersions, localContentUpdate, prepareVersionDownload } from '../platform/contentVersions.js';
const emit=defineEmits(['close','changed','busy']);
const rows=ref([]),tab=ref('downloads'),busy=ref(false),error=ref(''),notice=ref(''),selected=ref(null),preview=ref(null),latest=ref(null),confirmReplace=ref(false);
function close(){if(!busy.value)emit('close');}
async function run(work){if(busy.value)return;busy.value=true;emit('busy',true);error.value='';try{await work();}catch(e){error.value=String(e.message||e);}finally{busy.value=false;emit('busy',false);}}
async function load(){await run(async()=>{rows.value=await localContentUpdate({action:tab.value==='backups'?'backups':'list'});});}
async function check(row){await run(async()=>{notice.value='';selected.value=null;latest.value=null;confirmReplace.value=false;const local=await localContentUpdate({action:'preview',kind:row.kind,local_id:row.local_id});const history=await contentVersions(row.kind,row.remote_id);const remote=await contentVersions(row.kind,row.remote_id,history.current_version);preview.value=local;latest.value=remote;selected.value=row;if(row.version===remote.version)notice.value='已是当前广场版本；本地修改会保留。';});}
const canReplace=computed(()=>preview.value?.origin&&latest.value?.version>selected.value?.version&&(selected.value.kind!=='collection'||Object.keys(preview.value.origin.member_map||{}).length>0&&preview.value.origin.remote.members?.every(m=>m.member_id)&&latest.value.snapshot.members.every(m=>m.member_id)));
async function apply(replace){await run(async()=>{const payload=await prepareVersionDownload(latest.value);await localContentUpdate({action:'apply',kind:selected.value.kind,local_id:selected.value.local_id,remote_id:selected.value.remote_id,version:latest.value.version,remote:latest.value.snapshot,payload,expected_digest:preview.value.local_digest,replace});notice.value=replace?'已备份并更新，可在“更新前备份”恢复为副本。':'已另存新版，原内容保留。';selected.value=null;confirmReplace.value=false;emit('changed');rows.value=await localContentUpdate({action:'list'});});}
async function restore(row){await run(async()=>{await localContentUpdate({action:'restore_copy',id:row.id});notice.value='已恢复为新副本，当前内容保持不变。';emit('changed');});}
function readable(data){return [data.root.title,data.root.content,...(data.members||[]).map(m=>`${m.title}\n${m.content}`)].filter(Boolean).join('\n\n');}
function readableRemote(data){return [data.title,data.content,...(data.members||[]).map(m=>`${m.title}\n${m.content}`)].filter(Boolean).join('\n\n');}
onMounted(load);
</script>
<style scoped>
.tabs,.actions{display:flex;gap:12px;flex-wrap:wrap;margin:20px 0}.update-row{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:18px 0;border-bottom:1px solid var(--line)}p{color:var(--muted);line-height:1.7}.comparison{display:grid;grid-template-columns:1fr 1fr;gap:16px}.comparison section,.confirm{padding:16px;border:1px solid var(--line);border-radius:14px;min-width:0}pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:420px;overflow:auto;font:inherit;line-height:1.7}.warning{color:var(--text)}@media(max-width:700px){.comparison{grid-template-columns:1fr}}
</style>
