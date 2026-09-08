<template>
  <section class="panel operations">
    <div class="panel-heading"><h2>{{ headings[mode] }}</h2><button :disabled="busy" @click="load">刷新</button></div>
    <form v-if="mode==='audit'" class="operations-filters" @submit.prevent="offset=0;load()">
      <label>操作者<input v-model="filters.actor" placeholder="邮箱" maxlength="254"></label>
      <label>操作<input v-model="filters.action" placeholder="操作标识 / request_failed" maxlength="100"></label>
      <label>开始日期（UTC）<input v-model="filters.from" type="date"></label>
      <label>结束日期（UTC）<input v-model="filters.to" type="date"></label>
      <button :disabled="busy">查询</button><button type="button" :disabled="busy || !data" @click="exportLog">导出 JSON</button>
    </form>
    <label v-if="mode==='overview'" class="operations-period">新增指标时间范围<select v-model="days" :disabled="busy" @change="load"><option :value="7">近 7 天</option><option :value="30">近 30 天</option><option :value="90">近 90 天</option></select></label>
    <p v-if="error" role="alert" class="error-message">{{ error }}</p>
    <p v-if="notice" role="status" class="success-message">{{ notice }}</p>
    <p v-if="busy" role="status" class="muted">正在读取…</p>
    <template v-if="data && mode==='overview'">
      <div class="operations-metrics"><article v-for="[key,label] in metrics" :key="key"><span>{{ label }}</span><strong>{{ number(data[key]) }}</strong></article></div>
      <h3>近 {{ data.days }} 天新增</h3>
      <div class="operations-metrics"><article v-for="[key,label] in additions" :key="key"><span>{{ label }}</span><strong>{{ number(data[key]) }}</strong></article></div>
      <p class="muted">未记录创建时间：账号 {{ number(data.unknown_account_dates) }} 个，投稿 {{ number(data.unknown_publication_dates) }} 条，不计入新增。下载为已记录累计数，收藏为当前关系数，不代表活跃使用量。</p>
      <p class="muted">Mock {{ data.mock_enabled ? '已启用' : '已关闭，保留历史' }}；订单不含真实支付与金额。统计采集启用：{{ date(data.collection_started_at) }} · 更新：{{ date(data.updated_at) }}</p>
    </template>
    <template v-if="data && mode==='audit'">
      <p class="muted">共 {{ number(data.total) }} 条 · 每页 {{ data.limit }} 条。仅显示脱敏摘要，不包含请求正文或密钥。导出最多 500 条。</p>
      <div class="operations-events"><article v-for="item in data.items" :key="item.id"><div><strong>{{ item.action }}</strong><span :class="item.result==='failure'?'error-message':'muted'">{{ item.result==='failure'?'请求失败':'操作成功' }}</span></div><p>{{ item.actor_email || '未认证请求' }} · {{ date(item.created_at) }}</p><details><summary>查看摘要</summary><pre>{{ JSON.stringify(item.details,null,2) }}</pre><small class="muted">记录 ID：{{ item.id }}</small></details></article></div>
      <p v-if="!data.items.length" class="muted">没有符合条件的记录。</p>
      <div class="operations-pagination"><button :disabled="busy || !data.offset" @click="offset=Math.max(0,data.offset-data.limit);load()">上一页</button><span>{{ data.items.length?data.offset+1:0 }}–{{ data.offset+data.items.length }} / {{ data.total }}</span><button :disabled="busy || data.offset+data.items.length>=data.total" @click="offset=data.offset+data.limit;load()">下一页</button></div>
    </template>
    <template v-if="data && mode==='system'">
      <div class="operations-metrics"><article v-for="[key,label] in dependencies" :key="key"><span>{{ label }}</span><strong class="operations-health" :data-status="data[key]?.status">{{ status[data[key]?.status] || '未知' }}</strong><small v-if="data[key]?.latency_ms!==undefined">{{ data[key].latency_ms }} ms</small></article></div>
      <dl class="operations-facts"><dt>后端版本</dt><dd>{{ data.version }}</dd><dt>检测时间</dt><dd>{{ date(data.checked_at) }}</dd><dt>加密密钥</dt><dd>{{ data.encryption_key_loaded?'已载入（不代表备份完成）':'不可用' }}</dd><dt>模拟支付</dt><dd>{{ data.mock_billing?'已启用':'已关闭' }}</dd></dl>
      <h3>邮件队列</h3><p v-if="data.mail_queue===null" role="alert">队列读取失败，请刷新重试。</p><p v-else-if="!Object.keys(data.mail_queue).length" class="muted">暂无投递记录。</p><ul v-else><li v-for="(count,key) in data.mail_queue" :key="key">{{ mailStatus[key] || key }}：{{ count }}</li></ul>
      <h3>风险通知队列</h3><p v-if="data.notification_queue==null" role="alert">队列读取失败，请刷新重试。</p><p v-else-if="!Object.keys(data.notification_queue).length" class="muted">暂无风险通知。</p><ul v-else><li v-for="(count,key) in data.notification_queue" :key="key">{{ mailStatus[key] || key }}：{{ count }}</li></ul>
      <h3>备份与恢复</h3><p>{{ data.recovery }}</p><p class="muted">配套保存数据库备份、加密密钥与对象存储。先在隔离环境恢复并验证，再授权正式切换。操作手册：docs/how-to/backend-recovery.md。</p>
    </template>
  </section>
</template>
<script setup>
import {onMounted,ref} from 'vue';
import {getOperations} from './adminApi.js';
const props=defineProps({mode:{type:String,required:true}});
const emit=defineEmits(['busy-change']);
const headings={overview:'运行概况',audit:'操作记录',system:'服务健康'};
const metrics=[['accounts','全部账号'],['active_accounts','启用账号'],['online_content','在线内容'],['pending','待审投稿'],['recorded_downloads','累计记录下载'],['favorites','当前收藏'],['mock_orders','Mock 历史订单'],['mock_success','Mock 成功开通订单']];
const additions=[['new_accounts','新增账号'],['new_publications','新增投稿'],['new_mock_orders','新增 Mock 订单']];
const dependencies=[['postgres','PostgreSQL'],['redis','Redis'],['media','对象存储']];
const status={healthy:'可连接',unavailable:'不可用',not_configured:'未配置'};
const mailStatus={queued:'等待发送',mail_queued:'进入邮件队列',sending:'发送中',accepted:'对端已接收',failed:'失败',expired:'已过期',config_changed:'配置已变化',quota_limited:'额度不足，已跳过',unavailable:'渠道不可用'};
const data=ref(null),busy=ref(false),error=ref(''),notice=ref(''),days=ref(7),offset=ref(0),filters=ref({actor:'',action:'',from:'',to:''});
const number=value=>Number(value??0).toLocaleString();
const date=value=>value?new Date(value).toLocaleString():'未记录';
function query(){return props.mode==='overview'?{days:days.value}:props.mode==='audit'?{...filters.value,offset:offset.value}:{}}
async function load(){if(busy.value)return;busy.value=true;error.value='';notice.value='';data.value=null;try{data.value=await getOperations(props.mode,query())}catch(e){error.value=e.message}finally{busy.value=false}}
async function exportLog(){if(busy.value||!window.confirm('导出当前筛选条件下最新的最多 500 条脱敏记录？文件包含操作账号，请妥善保管。'))return;busy.value=true;emit('busy-change',true);error.value='';notice.value='';try{const result=await getOperations('audit/export',{...filters.value});const blob=new Blob([JSON.stringify(result,null,2)],{type:'application/json'});const url=URL.createObjectURL(blob);const link=document.createElement('a');link.href=url;link.download=`promptark-audit-${new Date().toISOString().slice(0,10)}.json`;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);notice.value=`已导出 ${result.items.length} 条记录（匹配 ${result.total} 条，上限 500 条）。`}catch(e){error.value=e.message}finally{busy.value=false;emit('busy-change',false)}}
onMounted(load);
</script>
<style scoped>
.operations .panel-heading{padding:0 0 20px}.operations-period select{font:inherit;color:inherit;background:#fff;border:1px solid #dce1e4;border-radius:7px;padding:10px 12px;min-height:42px}
.operations{padding:24px}.operations-filters{display:flex;gap:12px;align-items:end;flex-wrap:wrap;margin-bottom:20px}.operations-filters label{flex:1 1 160px;min-width:0}.operations-period{display:flex;align-items:center;gap:12px;margin:16px 0}.operations-period select{width:auto}.operations-metrics{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px;margin:20px 0}.operations-metrics article{display:flex;flex-direction:column;gap:10px;padding:20px;border:1px solid var(--border,#e6e7e9);border-radius:12px}.operations-metrics span{color:var(--muted,#747980);font-size:13px}.operations-metrics strong{font-size:28px;font-weight:600}.operations-metrics strong.operations-health{font-size:20px}.operations-health[data-status=unavailable]{color:#b6413c}.operations-health[data-status=healthy]{color:#247356}.operations-events article{padding:18px 0;border-bottom:1px solid var(--border,#e6e7e9);overflow-wrap:anywhere}.operations-events article>div{display:flex;justify-content:space-between;gap:12px}.operations-events p{font-size:13px;color:var(--muted,#747980)}.operations-events pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px;background:var(--surface,#f6f7f8);padding:16px;border-radius:8px}.operations-pagination{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-top:20px}.operations-facts{display:grid;grid-template-columns:120px 1fr;gap:12px}.operations-facts dd{margin:0;overflow-wrap:anywhere}.operations p{line-height:1.7}.operations h3{margin-top:28px;font-size:16px}@media(max-width:600px){.operations{padding:16px}.operations-metrics{grid-template-columns:repeat(2,minmax(0,1fr))}.operations-metrics article{padding:14px}.operations-facts{grid-template-columns:90px 1fr}.operations-events article>div{flex-wrap:wrap}}
</style>
