<template>
  <section class="panel"><div class="panel-heading"><h2>投稿初筛策略</h2><span class="badge">仅所有者可配置</span></div><p class="muted">默认人工审核。启用自动通过会公开符合策略的投稿，请核对条件后保存。所有检查只处理主动发布的公开快照。</p>
    <p v-if="error" class="error-banner" role="alert">{{ error }}</p><p v-if="message" class="success-message" role="status">{{ message }}</p><p v-if="loading" role="status" class="empty-state">正在加载…</p><button v-else-if="!loaded" @click="load">重新加载</button>
    <form v-else class="moderation-form" @submit.prevent="save"><fieldset :disabled="busy">
      <label class="moderation-toggle"><input v-model="draft.enabled" type="checkbox" data-testid="moderation-enabled"><span><strong>启用自动初筛</strong><small>关闭时保持人工队列，不执行本策略限额。</small></span></label>
      <label class="moderation-toggle"><input v-model="draft.ai_decides" type="checkbox" data-testid="moderation-direct" @change="setDirect"><span><strong>AI 直接通过或拒绝（宽松）</strong><small>覆盖提示词、合集和 Skill；只拒绝明确风险，普通措辞、格式和重复模板不拦截。异常或文件未完整检查保留待审核。需配置三类审核路由。</small></span></label>
      <div class="moderation-numbers"><label>每作者 24 小时投稿上限<input v-model.number="draft.daily_limit" type="number" min="1" max="1000" required></label><label v-if="!draft.ai_decides">自动通过：风险分数低于<input v-model.number="draft.approve_below" type="number" min="1" :max="draft.manual_at-1" required></label><label v-if="!draft.ai_decides">高风险：转人工分数<input v-model.number="draft.manual_at" type="number" :min="draft.approve_below+1" max="100" required></label></div>
      <template v-if="!draft.ai_decides"><h3>检查项目</h3><label v-for="(label,key) in checks" :key="key" class="moderation-toggle"><input v-model="draft[key]" type="checkbox"><span>{{ label }}</span></label>
      <label class="moderation-toggle"><input v-model="draft.require_ai" type="checkbox" data-testid="moderation-ai"><span><strong>必须完成 AI 审核</strong><small>未配置模型时进入人工队列，不会假报 AI 通过。</small></span></label>
      <label class="moderation-toggle"><input v-model="draft.auto_approve" type="checkbox" data-testid="moderation-approve"><span><strong>允许低风险自动通过并上架</strong><small>仅当所有必需检查完成且没有转人工原因；本地规则初筛不是安全保证。</small></span></label>
      </template><div class="notice"><p>勾选图片检查后，稿件图片可进入已授权视觉模型的任务队列；需在审核模型启用图片发送并选择 Skill 路由。未配置模型、未支持的文档或外部图片引用转人工。开启直接判定后，完整图文审核通过可直接上架；关闭时仍需人工确认。</p></div>
      <footer class="provider-footer"><span class="muted">配置版本 {{ draft.revision }}</span><button class="primary" :disabled="!hasUnsavedChanges" data-testid="moderation-save">{{ busy?'保存中…':'保存策略' }}</button></footer>
    </fieldset></form>
  </section>
</template>
<script setup>
import { computed,onMounted,ref } from 'vue';import { getModerationPolicy,saveModerationPolicy } from './adminApi.js';
const emit=defineEmits(['busy-change']);const draft=ref(null),saved=ref(''),loaded=ref(false),loading=ref(false),busy=ref(false),error=ref(''),message=ref('');
const checks={check_duplicates:'检查重复正文与合集成员',check_structure:'检查快照完整性及变量标记配对',check_sensitive:'使用已保存安全词库',check_images:'检测图片引用并要求图片安全检查'};
const hasUnsavedChanges=computed(()=>loaded.value&&JSON.stringify(draft.value)!==saved.value);defineExpose({hasUnsavedChanges,isBusy:busy});
function accept(data){if(!Number.isInteger(data?.revision)||typeof data.enabled!=='boolean'||!Number.isInteger(data.daily_limit))throw Error('策略响应无效');draft.value=structuredClone(data);saved.value=JSON.stringify(data);loaded.value=true;}
function setDirect(){if(draft.value.ai_decides)Object.assign(draft.value,{require_ai:true,auto_approve:true,check_images:true});}
async function load(){loading.value=true;error.value='';try{accept(await getModerationPolicy());}catch(e){error.value=e.message;}finally{loading.value=false;}}
async function save(){if(busy.value||!loaded.value)return;if(draft.value.enabled&&draft.value.auto_approve&&!window.confirm('确认启用自动上架？符合当前检查条件的低风险内容会直接公开。'))return;busy.value=true;emit('busy-change',true);error.value='';message.value='';try{const data=await saveModerationPolicy(JSON.parse(JSON.stringify(draft.value)));if(data.revision!==draft.value.revision+1)throw Error('服务端未确认策略保存');accept(data);message.value='策略已保存，对后续投稿生效';}catch(e){error.value=e.message;}finally{busy.value=false;emit('busy-change',false);}}
onMounted(load);
</script>
<style scoped>
.moderation-form {padding:24px;}.moderation-toggle {display:flex;gap:12px;align-items:flex-start;margin:20px 0;}.moderation-toggle input {margin-top:4px;}.moderation-toggle small {display:block;color:var(--muted);font-weight:400;margin-top:4px;}.moderation-numbers {display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px;margin:24px 0;}.provider-footer {padding:20px 0 0;}@media(max-width:700px){.moderation-numbers{grid-template-columns:1fr;}}
</style>
