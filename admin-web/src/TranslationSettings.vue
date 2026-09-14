<template>
  <section class="panel translation-settings">
    <header class="panel-heading"><div><h2>提示词翻译</h2><p class="muted">中文浏览，原文随时可用。英文按需生成，已有译文共同复用。</p></div><button :disabled="busy" @click="refresh">刷新进度</button></header>
    <p v-if="error" role="alert" class="error-banner">{{ error }}</p><p v-if="message" role="status">{{ message }}</p>
    <template v-if="summary">
      <div class="translation-stats"><article v-for="stat in stats" :key="stat.label"><span>{{ stat.label }}</span><strong>{{ stat.value.toLocaleString() }}</strong></article></div>
      <div class="translation-progress"><progress :max="Math.max(1,summary.total)" :value="summary.zh_ready || 0" aria-label="翻译完成进度" /><p>{{ draft.enabled ? '后台翻译已启用' : '后台翻译已暂停' }} · 今日用量（含处理中及失败预留）{{ summary.used_tokens.toLocaleString() }} / {{ draft.daily_tokens.toLocaleString() }} tokens</p></div>
      <div class="translation-actions"><button class="primary" :disabled="busy || !savedModel" @click="action('queue')">为全部广场内容生成中文</button><button :disabled="busy || !savedModel" @click="action(draft.enabled ? 'pause' : 'resume')">{{ draft.enabled ? '暂停新任务' : '继续任务' }}</button><button :disabled="busy || !summary.counts.failed" @click="action('retry')">重试失败项</button></div>
      <p class="muted">任务可断点继续；暂停后当前请求会完成。达到每日限额后等待次日。原始标题、正文、附件及统计保持不变。</p>
      <details class="translation-config" open><summary>翻译模型配置 · 仅用于公开广场</summary>
        <form @submit.prevent="save"><fieldset :disabled="busy"><div class="translation-fields">
          <label>完整 Chat Completions 地址<input v-model="draft.endpoint" @input="models=[]" type="url" required placeholder="https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"></label>
          <label>API Key<input v-model="key" @input="models=[]" type="password" autocomplete="new-password" :placeholder="draft.has_key ? '已加密保存，留空保留' : '填写服务端翻译 Key'"></label>
          <label>翻译模型<SearchableSelect v-model="draft.model" aria-label="广场翻译模型" :options="options" placeholder="获取模型列表后选择" /><button type="button" :disabled="!draft.endpoint" @click="discover">获取模型列表</button><small>推荐低价专用翻译模型 qwen-mt-flash；列表可见不代表调用测试通过。</small></label>
          <label>每日 token 上限<input v-model.number="draft.daily_tokens" type="number" min="10000" max="100000000" required><small>包括输入、输出和失败请求的保守预留；不是人民币费用。</small></label>
        </div><label><input v-model="draft.enabled" type="checkbox">启用后台翻译</label><div class="translation-save"><label>当前管理员密码<input v-model="password" type="password" autocomplete="current-password" required></label><button class="primary" :disabled="!draft.model || !password">保存配置</button></div></fieldset></form>
      </details>
      <section v-if="summary.failures.length"><h3>最近失败</h3><article v-for="failure in summary.failures" :key="failure.id+failure.target" class="translation-failure"><strong>{{ failure.title }}</strong><span>{{ failure.target === 'zh' ? '中文' : '英文' }} · {{ failure.error }}</span></article></section>
    </template><p v-else class="empty-state">正在读取翻译任务…</p>
  </section>
</template>
<script setup>
import {computed,onMounted,onBeforeUnmount,ref} from 'vue';
import SearchableSelect from '../../desktop/src/components/SearchableSelect.vue';
import {getTranslation,saveTranslation,translationAction,discoverAiModels} from './adminApi.js';
const emit=defineEmits(['busy-change']);
const summary=ref(null),draft=ref(null),key=ref(''),password=ref(''),models=ref([]),busy=ref(false),error=ref(''),message=ref(''),saved=ref('');
const savedModel=computed(()=>saved.value&&JSON.parse(saved.value).model);
const hasUnsavedChanges=computed(()=>Boolean(draft.value)&&(JSON.stringify(draft.value)!==saved.value||Boolean(key.value)));
defineExpose({hasUnsavedChanges,isBusy:busy});
const options=computed(()=>[...new Set([...models.value,draft.value?.model].filter(Boolean))].map(id=>({value:id,label:models.value.includes(id)?id:`${id}（已有配置，待核实）`})));
const stats=computed(()=>[{label:'广场条目',value:summary.value.total},{label:'已就绪版本',value:summary.value.counts.ready||0},{label:'等待与处理中',value:(summary.value.counts.queued||0)+(summary.value.counts.running||0)},{label:'失败',value:summary.value.counts.failed||0}]);
async function refresh(){try{const value=await getTranslation();summary.value=value;if(!hasUnsavedChanges.value){draft.value=value.config;saved.value=JSON.stringify(value.config);}}catch(e){error.value=e.message;}}
async function operation(fn){if(busy.value)return;busy.value=true;emit('busy-change',true);error.value='';message.value='';try{await fn();}catch(e){error.value=e.message;}finally{busy.value=false;emit('busy-change',false);}}
function discover(){return operation(async()=>{models.value=[];const result=await discoverAiModels({endpoint:draft.value.endpoint,key:key.value,translation:true});if(result.error)throw Error(result.error);models.value=result.models;message.value=`已获取 ${result.models.length} 个模型。`;});}
function save(){return operation(async()=>{const {has_key,...config}=draft.value;try{const result=await saveTranslation({...config,key:key.value,current_password:password.value});draft.value=result;saved.value=JSON.stringify(result);key.value='';message.value='翻译配置已保存。';await refresh();}finally{password.value='';}});}
function action(name){return operation(async()=>{if(hasUnsavedChanges.value)throw Error('请先保存模型配置或重新进入此页。');const result=await translationAction(name);message.value=name==='queue'?`新增 ${result.affected} 个任务，已有版本不会重复计费。`:'任务状态已更新。';await refresh();});}
let timer;onMounted(async()=>{await refresh();timer=setInterval(()=>{if(!busy.value)refresh();},10000);});onBeforeUnmount(()=>clearInterval(timer));
</script>
<style scoped>
.translation-settings>p,.translation-config,.translation-progress,.translation-actions,.translation-settings>section{margin:20px 24px}.translation-stats{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:14px;margin:20px 24px}.translation-stats article{border:1px solid var(--line);border-radius:10px;padding:18px;display:grid;gap:12px}.translation-stats span{color:var(--muted);font-size:12px}.translation-stats strong{font-size:26px}.translation-progress progress{width:100%;height:8px;accent-color:var(--text)}.translation-progress p,.translation-config small{font-size:12px;color:var(--muted)}.translation-actions{display:flex;gap:10px;flex-wrap:wrap}.translation-config{border-top:1px solid var(--line);padding-top:22px}.translation-config summary{font-weight:600;cursor:pointer}.translation-config form{margin-top:20px}.translation-fields{display:grid;grid-template-columns:1fr 1fr;gap:20px;margin-bottom:20px}.translation-fields label{display:grid;gap:8px;min-width:0}.translation-fields input,.translation-save input{width:100%;box-sizing:border-box}.translation-save{display:flex;gap:16px;align-items:end;margin-top:20px;justify-content:flex-end}.translation-failure{padding:12px 0;border-bottom:1px solid var(--line);display:grid;gap:6px}.translation-failure span{color:var(--muted);font-size:12px}@media(max-width:700px){.translation-stats,.translation-fields{grid-template-columns:1fr 1fr}.translation-fields{grid-template-columns:1fr}.translation-save{display:grid}}
</style>
