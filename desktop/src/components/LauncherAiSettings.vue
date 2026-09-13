<template>
  <div class="settings-group launcher-ai-settings">
    <h4>启动器 AI 优化</h4>
    <p>使用自己的 OpenAI 兼容接口。仅在点击「AI 优化」时发送当前输入；密钥保存在此设备的系统凭据库，不参与云同步或库备份。</p>
    <p v-if="!native" role="status">请在桌面客户端配置和使用 AI 优化。</p>
    <template v-else>
      <p v-if="loading">正在读取本机配置…</p>
      <fieldset :disabled="busy || loading || loadFailed">
        <label class="field"><span>API 基础地址</span><input v-model="endpoint" data-testid="ai-endpoint" placeholder="https://dashscope.aliyuncs.com/compatible-mode/v1" autocomplete="off"></label>
        <label class="field"><span>API Key</span><input v-model="apiKey" data-testid="ai-key" type="password" :placeholder="hasKey ? '已保存；留空保留，仅同一地址可沿用' : '填写自己的 API Key'" autocomplete="new-password"></label>
        <label class="field"><span>模型 ID</span><input v-model="model" data-testid="ai-model" list="launcher-ai-models" placeholder="填写或选择供应商的模型 ID"><datalist id="launcher-ai-models"><option v-for="id in models" :key="id" :value="id" /></datalist></label>
        <div class="modal-actions"><button type="button" class="button primary-button" data-testid="ai-save" @click="save">保存 AI 配置</button><button type="button" class="button" :disabled="dirty || !saved.endpoint" @click="fetchModels">获取模型列表</button><button type="button" class="button" :disabled="dirty || !saved.endpoint" @click="test">测试连接</button><button type="button" class="button" :disabled="!saved.endpoint" @click="clear">清除配置与密钥</button></div>
      </fieldset>
      <p v-if="note" role="status">{{ note }}</p><button v-if="loadFailed" class="button" type="button" @click="load">重试读取</button>
      <small>先保存再测试；测试只发送固定示例，不读取个人提示词。供应商不支持模型列表时可手动填写。</small>
    </template>
  </div>
</template>
<script setup>
import { computed, onMounted, ref, watch } from 'vue';
import { getLauncherAiConfig, saveLauncherAiConfig, clearLauncherAiConfig, listLauncherAiModels, optimizeLauncherPrompt } from '../platform/launcherAi.js';
const emit=defineEmits(['dirty','busy']);
const native=Boolean(window.__TAURI_INTERNALS__);
const endpoint=ref(''),model=ref(''),apiKey=ref(''),hasKey=ref(false),models=ref([]),note=ref(''),loading=ref(false),busy=ref(false),loadFailed=ref(false),saved=ref({endpoint:'',model:''});
const dirty=computed(()=>endpoint.value!==saved.value.endpoint || model.value!==saved.value.model || Boolean(apiKey.value));
watch(dirty,value=>emit('dirty',value));watch([busy,loading],()=>emit('busy',busy.value||loading.value));
function apply(value) { endpoint.value=value.endpoint||'';model.value=value.model||'';hasKey.value=!!value.has_key;apiKey.value='';saved.value={endpoint:endpoint.value,model:model.value}; }
async function load(){loading.value=true;loadFailed.value=false;try{apply(await getLauncherAiConfig());note.value='';}catch(e){loadFailed.value=true;note.value=String(e.message||e);}finally{loading.value=false;}}
async function operation(fn){if(busy.value)return;busy.value=true;note.value='';try{await fn();}catch(e){note.value=String(e.message||e);}finally{busy.value=false;}}
function save(){return operation(async()=>{apply(await saveLauncherAiConfig({endpoint:endpoint.value,model:model.value,api_key:apiKey.value}));models.value=[];note.value='AI 配置已保存在本机；尚未测试连接。';});}
function fetchModels(){return operation(async()=>{models.value=await listLauncherAiModels();note.value=`已读取 ${models.value.length} 个模型；请在模型输入框选择后保存。`;});}
function test(){return operation(async()=>{await optimizeLauncherPrompt('用三句话介绍如何整理书桌。');note.value='连接成功，所选模型已返回完整文本。';});}
function clear(){return operation(async()=>{await clearLauncherAiConfig();apply({});models.value=[];note.value='本机 AI 配置和密钥已清除。';});}
onMounted(()=>{if(native)load();});
</script>
<style scoped>
.launcher-ai-settings p,.launcher-ai-settings small {font-size:12px;color:var(--muted);line-height:1.7}.launcher-ai-settings fieldset{border:0;padding:0;margin:0;display:grid;gap:16px;min-width:0}.launcher-ai-settings .modal-actions{flex-wrap:wrap;justify-content:flex-start}.launcher-ai-settings h4{margin:0}
</style>
