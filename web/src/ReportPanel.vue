<template>
  <section class="editor" aria-label="举报与进度"><h2>{{ targetId ? '举报广场内容' : '我的举报' }}</h2><p>只提交指定公开条目及描述，不读取本地库。</p><form v-if="targetId" @submit.prevent="submit"><label>类型<select v-model="category"><option v-for="(label,key) in categories" :key="key" :value="key">{{ label }}</option></select></label><label>原因<textarea v-model="reason" maxlength="1000" required placeholder="说明具体问题，勿填写秘密"></textarea></label><button :disabled="busy || !reason.trim()">提交举报</button></form><p v-if="error" role="alert">{{ error }}</p><p v-if="message" role="status">{{ message }}</p><button :disabled="busy" @click="load(0)">刷新我的举报</button><article v-for="row in rows" :key="row.id"><strong>{{ statuses[row.status] }} · {{ categories[row.category] }}</strong><p>{{ row.reason }}</p><p v-if="row.resolution">处理结果：{{ row.resolution }}</p><small>{{ row.target_id }} · {{ new Date(row.updated_at).toLocaleString() }}</small></article><p v-if="loaded && !rows.length">暂无举报记录</p><button v-if="offset>0" :disabled="busy" @click="load(offset-25)">上一页</button><button v-if="offset+25<total" :disabled="busy" @click="load(offset+25)">下一页</button><button :disabled="busy" @click="$emit('close')">关闭</button></section>
</template>
<script setup>
import { onMounted,ref } from 'vue';import { reportRequest } from './reports.js';
const props=defineProps({targetId:{type:String,default:''}});defineEmits(['close']);
const categories={privacy:'隐私与密钥',medical:'医疗健康',financial:'金融投资',copyright:'版权与肖像',danger:'危险内容',spam:'垃圾广告',other:'其他'},statuses={pending:'待处理',processing:'处理中',dismissed:'已驳回',closed:'已下架结案'};
const category=ref('other'),reason=ref(''),busy=ref(false),error=ref(''),message=ref(''),rows=ref([]),loaded=ref(false),offset=ref(0),total=ref(0);
async function load(next=0){if(busy.value)return;busy.value=true;error.value='';try{const data=await reportRequest(null,next);if(!Array.isArray(data.items))throw Error('举报响应无效');rows.value=data.items;total.value=data.total;offset.value=next;loaded.value=true;}catch(e){error.value=e.message;rows.value=[];}finally{busy.value=false;}}
async function submit(){if(busy.value||!reason.value.trim())return;busy.value=true;error.value='';message.value='';try{const data=await reportRequest({target_id:props.targetId,category:category.value,reason:reason.value});if(!data.id)throw Error('服务端未确认举报');reason.value='';message.value='举报已受理，未结案的重复举报会合并。';}catch(e){error.value=e.message;}finally{busy.value=false;}if(message.value)await load();}
onMounted(()=>load());
</script>
<style scoped>
form,label {display:grid;gap:8px;} textarea {min-height:90px;} select,textarea {font:inherit;padding:8px;border:1px solid #ddd;border-radius:6px;width:100%;} article {padding:12px 0;border-bottom:1px solid #eee;overflow-wrap:anywhere;}
</style>
