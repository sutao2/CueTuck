<template><section class="panel ai-jobs"><h3>AI 审核任务</h3><p>投稿先保存再异步审核；执行中断可恢复，最多尝试三次。失败或附件稿件可继续人工审核。</p><button :disabled="busy" @click="load">刷新任务</button><p v-if="error" role="alert">{{ error }}</p><p v-if="message" role="status">{{ message }}</p><p v-if="items && !items.length">暂无任务</p><article v-for="item in items" :key="item.id"><strong>{{ item.id }}</strong> · {{ labels[item.status] || item.status }} · {{ item.attempts }}/3<p>{{ item.error }}</p><button v-if="item.status==='failed' && item.attempts<3" :disabled="busy" @click="retry(item)">立即重试</button></article><footer><button :disabled="busy||!offset" @click="offset-=25;load()">上一页</button><span>{{ total }} 条</span><button :disabled="busy||offset+25>=total" @click="offset+=25;load()">下一页</button></footer></section></template>
<script setup>
import {onMounted,ref} from 'vue';
import {listAiJobs,retryAiJob} from './adminApi.js';
const emit=defineEmits(['busy-change']);const items=ref(null),offset=ref(0),total=ref(0),busy=ref(false),error=ref(''),message=ref('');
const labels={queued:'等待执行',running:'执行中',failed:'失败',completed:'已完成',cancelled:'人工或其他流程已处置'};
async function load(){if(busy.value)return;busy.value=true;error.value='';try{const data=await listAiJobs(offset.value);items.value=data.items;total.value=data.total}catch(e){error.value=e.message}finally{busy.value=false}}
async function retry(item){if(busy.value)return;busy.value=true;emit('busy-change',true);error.value='';message.value='';let queued=false;try{const data=await retryAiJob(item.id);if(!data.queued)throw Error('服务器未确认入队');queued=true;message.value='已重新排队，不代表审核通过。'}catch(e){error.value=e.message}finally{busy.value=false;emit('busy-change',false)}if(queued)await load();}
onMounted(load);
</script>
<style scoped>.ai-jobs{padding:24px;margin-top:24px}.ai-jobs article{padding:16px 0;border-bottom:1px solid var(--line);overflow-wrap:anywhere}.ai-jobs p{color:var(--muted);font-size:13px;line-height:1.6}.ai-jobs footer{display:flex;justify-content:space-between;margin-top:16px}</style>
