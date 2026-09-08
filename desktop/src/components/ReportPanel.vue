<template>
  <details class="report-panel" @toggle="onToggle"><summary>举报内容 / 我的举报进度</summary>
    <p class="muted">仅提交当前广场条目和你填写的原因，不包含本地库。</p>
    <form @submit.prevent="submit"><fieldset :disabled="busy"><label>举报类型<select v-model="category"><option v-for="(label,key) in categories" :key="key" :value="key">{{ label }}</option></select></label><label>举报原因<textarea v-model="reason" maxlength="1000" placeholder="描述具体问题，勿填写密码或其他秘密" required data-testid="report-reason"></textarea></label><button class="button ghost-button" :disabled="!reason.trim()">{{ busy?'正在提交…':'提交举报' }}</button></fieldset></form>
    <p v-if="error" role="alert">{{ error }}</p><p v-if="message" role="status">{{ message }}</p>
    <h3>我的举报</h3><button class="button ghost-button" type="button" :disabled="busy" @click="load(0)">刷新进度</button><p v-if="loaded && !rows.length">暂无举报记录</p><article v-for="row in rows" :key="row.id"><strong>{{ statuses[row.status] }} · {{ categories[row.category] }}</strong><p>{{ row.reason }}</p><p v-if="row.resolution">处理结果：{{ row.resolution }}</p><small>{{ new Date(row.updated_at).toLocaleString() }} · {{ row.target_id }}</small></article><div><button v-if="offset>0" class="button ghost-button" :disabled="busy" @click="load(offset-25)">上一页</button><button v-if="offset+25<total" class="button ghost-button" :disabled="busy" @click="load(offset+25)">下一页</button></div>
  </details>
</template>
<script setup>
import { ref } from 'vue';
import { reportRequest } from '../platform/reports.js';
const props=defineProps({targetId:{type:String,required:true}});
const categories={privacy:'隐私与密钥',medical:'医疗健康',financial:'金融投资',copyright:'版权与肖像',danger:'危险内容',spam:'垃圾广告',other:'其他'};
const statuses={pending:'待处理',processing:'处理中',dismissed:'已驳回',closed:'已下架结案'};
const category=ref('other'),reason=ref(''),busy=ref(false),error=ref(''),message=ref(''),rows=ref([]),loaded=ref(false),offset=ref(0),total=ref(0);
async function load(next=0){if(busy.value)return;busy.value=true;error.value='';try{const data=await reportRequest(null,next);if(!Array.isArray(data.items))throw Error('举报响应无效');rows.value=data.items;total.value=data.total;offset.value=next;loaded.value=true;}catch(e){error.value=e.message;rows.value=[];}finally{busy.value=false;}}
function onToggle(e){if(e.target.open&&!loaded.value)load();}
async function submit(){if(busy.value||!reason.value.trim())return;busy.value=true;error.value='';message.value='';try{const data=await reportRequest({target_id:props.targetId,category:category.value,reason:reason.value});if(!data.id)throw Error('服务端未确认举报');reason.value='';message.value='举报已受理；重复提交会合并到尚未结案的同一举报。';}catch(e){error.value=e.message;}finally{busy.value=false;}if(message.value)await load();}
</script>
<style scoped>
.report-panel { margin-top: 20px; border-top: 1px solid var(--border-color,#ddd); padding-top: 16px; }
summary { cursor: pointer; }
form, label { display: grid; gap: 8px; margin: 12px 0; }
fieldset { border: 0; padding: 0; min-width: 0; }
textarea, select { width: 100%; font: inherit; padding: 8px; border: 1px solid var(--border-color,#ddd); border-radius: 7px; background: var(--panel-bg,#fff); color: inherit; }
textarea { min-height: 80px; resize: vertical; }
article { padding: 12px 0; border-bottom: 1px solid var(--border-color,#ddd); overflow-wrap: anywhere; }
</style>
