<template>
  <section class="media-reclaim" aria-label="云端附件回收">
    <h3>云端附件回收</h3>
    <p class="muted">仅列出超过 7 天未复用且未被服务器任何快照引用的已完成文件或中断上传。软删除、驳回和下架记录仍受保护；不清理近期上传，不自动执行。</p>
    <h4>只读存储对账</h4><p>检查本项目 promptark/ 前缀。未知对象可能属于其他实例，只报告、不删除；检查期间的并发上传也可能产生暂时差异。</p><button :disabled="busy" @click="scan('database')">检查数据库文件</button><button :disabled="busy" @click="scan('bucket')">检查存储桶对象</button>
    <section v-if="scanResult"><p>本页检查 {{ scanResult.checked }} 项，发现 {{ scanResult.items.length }} 项差异。{{ scanResult.next_cursor ? '尚有下一页，未完成全部检查。' : '已到该方向末页；另一方向需单独检查。' }}</p><p v-for="row in scanResult.items" :key="row.id">{{ row.id }}：{{ scanLabels[row.status] || row.status }}</p><button v-if="scanResult.next_cursor" :disabled="busy" @click="scan(scanResult.side,scanResult.next_cursor)">检查下一页</button></section>
    <h4>确认清理</h4>
    <button :disabled="busy" data-testid="media-inspect" @click="load">{{ busy ? '正在处理…' : '检查可回收文件' }}</button>
    <p v-if="error" class="error-message" role="alert">{{ error }}</p><p v-if="notice" role="status">{{ notice }}</p>
    <template v-if="items !== null">
      <p v-if="!items.length" class="muted">当前没有可回收文件。</p>
      <p v-if="more" class="muted">仅显示最早的 25 项；清理后重新检查可查看后续候选。</p>
      <article v-for="file in items" :key="file.id" class="reclaim-item">
        <div class="reclaim-row">
        <div><strong>{{ file.name }}</strong><small>{{ bytes(file.size) }} · {{ file.deleting ? '上次未完成，可重试清理' : file.ready === false ? '中断上传，超过保护期' : '未被服务器引用' }}</small><small>{{ file.id }}</small></div>
        <button :disabled="busy" @click="selected=file;error='';notice=''">{{ file.deleting ? '重试清理' : '清理' }}</button>
        </div>
      <div v-if="selected?.id===file.id" class="reclaim-confirm" role="region" aria-label="确认回收">
        <strong>永久清理「{{ selected.name }}」？</strong>
        <p>仅删除这份云端文件，不删除本地副本。无法撤销，请先确认备份。服务器无法识别离线设备尚未提交的草稿；这些草稿之后可能需要重新上传附件。执行时会再次检查服务器引用。</p>
        <button :disabled="busy" data-testid="media-cancel" @click="selected=null">取消</button>
        <button :disabled="busy" data-testid="media-confirm" @click="purge">{{ busy ? '正在清理…' : '确认永久清理' }}</button>
      </div>
      </article>
    </template>
  </section>
</template>
<script setup>
import {ref} from 'vue';
import {listOrphanMedia,purgeOrphanMedia,scanMedia} from './adminApi.js';
const emit=defineEmits(['busy-change']);
const items=ref(null),more=ref(false),selected=ref(null),busy=ref(false),error=ref(''),notice=ref('');
const scanResult=ref(null),scanLabels={untracked:'本数据库无记录（仅报告）',missing:'数据库有记录但对象缺失',incomplete:'上传未完成但对象存在',incomplete_missing:'上传未完成且对象缺失'};
async function scan(side,cursor=''){if(busy.value)return;working(true);error.value='';scanResult.value=null;try{const result=await scanMedia(side,cursor);if(!Array.isArray(result.items))throw Error('对账响应无效');scanResult.value=result}catch(e){error.value=e.message}finally{working(false)}}
const bytes=value=>value>=1048576?`${(value/1048576).toFixed(1)} MiB`:`${Math.ceil(value/1024)} KiB`;
function working(value){busy.value=value;emit('busy-change',value)}
async function load(){
  if(busy.value)return;working(true);error.value='';notice.value='';selected.value=null;items.value=null;
  try{const result=await listOrphanMedia();if(!Array.isArray(result.items))throw Error('候选响应无效，请重试');items.value=result.items;more.value=Boolean(result.more)}
  catch(e){error.value=e.message}finally{working(false)}
}
async function purge(){
  if(busy.value||!selected.value)return;const file=selected.value;working(true);error.value='';notice.value='';
  try{const result=await purgeOrphanMedia(file.id);if(result.removed!==true)throw Error('服务器未确认回收完成，请重新检查');items.value=items.value.filter(item=>item.id!==file.id);selected.value=null;notice.value=`已清理「${file.name}」的云端文件；本地副本未删除。`}
  catch(e){error.value=e.message}finally{working(false)}
}
</script>
<style scoped>
.media-reclaim{margin-top:28px;border-top:1px solid var(--border,#e6e7e9);padding-top:8px}.reclaim-row{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:14px 0;border-bottom:1px solid var(--border,#e6e7e9)}.reclaim-row>div{min-width:0;overflow-wrap:anywhere}.reclaim-row small{display:block;color:var(--muted,#747980);margin-top:6px}.reclaim-confirm{margin-top:18px;padding:18px;border:1px solid #d7b6a6;border-radius:10px}.reclaim-confirm button{margin-right:8px}p{font-size:13px;line-height:1.7}
</style>
