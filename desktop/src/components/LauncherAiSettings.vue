<template>
  <div class="settings-group launcher-ai-settings">
    <h4>本机模型配置</h4>
    <p>为 AI 优化和本地提示词翻译选择模型。仅主动操作时发送对应正文；API Key 保存在此设备的系统凭据库，不参与同步或备份。</p>
    <p v-if="!native" role="status">请在桌面客户端配置和使用本机 AI 服务。</p>
    <template v-else>
      <p v-if="loading">正在读取本机配置…</p>
      <fieldset :disabled="busy || loading || loadFailed">
        <label class="field"><span>API 基础地址</span><input v-model="endpoint" data-testid="ai-endpoint" placeholder="https://dashscope.aliyuncs.com/compatible-mode/v1" autocomplete="off"></label>
        <label class="field"><span>API Key</span><input v-model="apiKey" data-testid="ai-key" type="password" :placeholder="hasKey ? '已保存；留空保留，仅同一地址可沿用' : '填写自己的 API Key'" autocomplete="new-password"></label>
        <div><button type="button" class="button" data-testid="ai-fetch-models" :disabled="!endpoint.trim()" @click="fetchModels">{{ busy ? '处理中…' : '获取模型列表' }}</button><small>填写接口后直接获取，无需先保存模型。</small></div>
        <label class="field"><span>启动器优化模型</span><SearchableSelect v-model="model" data-testid="ai-model" aria-label="启动器优化模型" :options="options" :disabled="busy || loading" placeholder="获取模型列表后选择" /></label>
        <label class="field"><span>本地翻译模型</span><SearchableSelect v-model="translationModel" data-testid="ai-translation-model" aria-label="本地翻译模型" :options="options" :disabled="busy || loading" placeholder="未配置翻译模型" /><small>百炼可选择 qwen-mt-flash 等专用翻译模型，也可使用通用文本模型。模型列表可搜索；列表可见不代表已通过调用测试。</small></label>
        <div class="modal-actions"><button type="button" class="button primary-button" data-testid="ai-save" :disabled="!model && !translationModel" @click="save">保存本机 AI 配置</button><button type="button" class="button" :disabled="dirty || !saved.model" @click="test">测试优化模型</button><button type="button" class="button" :disabled="!saved.endpoint" @click="clear">清除配置与密钥</button></div>
      </fieldset>
      <p v-if="note" role="status">{{ note }}</p><button v-if="loadFailed" class="button" type="button" @click="load">重试读取</button>
      <small>获取列表只读取模型目录；测试只发送固定示例，不读取个人提示词。列表失败不会替换已保存配置。</small>
    </template>
  </div>
</template>
<script setup>
import { computed, onMounted, ref, watch } from 'vue';
import SearchableSelect from './SearchableSelect.vue';
import { getLauncherAiConfig, saveLauncherAiConfig, clearLauncherAiConfig, listLauncherAiModels, optimizeLauncherPrompt } from '../platform/launcherAi.js';
const emit=defineEmits(['dirty','busy']);
const native=Boolean(window.__TAURI_INTERNALS__);
const endpoint=ref(''),model=ref(''),translationModel=ref(''),apiKey=ref(''),hasKey=ref(false),models=ref([]),note=ref(''),loading=ref(false),busy=ref(false),loadFailed=ref(false),saved=ref({endpoint:'',model:'',translation_model:''});
const dirty=computed(()=>endpoint.value!==saved.value.endpoint || model.value!==saved.value.model || translationModel.value!==saved.value.translation_model || Boolean(apiKey.value));
const options=computed(()=>[{value:'',label:'不启用此用途'},...[...new Set([...models.value,model.value,translationModel.value].filter(Boolean))].map(id=>({value:id,label:models.value.includes(id)?id:`${id}（已有选择，待获取核实）`}))]);
watch(dirty,value=>emit('dirty',value));watch([busy,loading],()=>emit('busy',busy.value||loading.value));
watch([endpoint,apiKey],()=>{models.value=[];});
function apply(value) { endpoint.value=value.endpoint||'';model.value=value.model||'';translationModel.value=value.translation_model||'';hasKey.value=!!value.has_key;apiKey.value='';saved.value={endpoint:endpoint.value,model:model.value,translation_model:translationModel.value}; }
async function load(){loading.value=true;loadFailed.value=false;try{apply(await getLauncherAiConfig());note.value='';}catch(e){loadFailed.value=true;note.value=String(e.message||e);}finally{loading.value=false;}}
async function operation(fn){if(busy.value)return;busy.value=true;note.value='';try{await fn();}catch(e){note.value=String(e.message||e);}finally{busy.value=false;}}
function config(){return {endpoint:endpoint.value,model:model.value,translation_model:translationModel.value,api_key:apiKey.value};}
function save(){return operation(async()=>{apply(await saveLauncherAiConfig(config()));note.value='AI 配置已保存在本机；尚未测试连接。';});}
function fetchModels(){return operation(async()=>{models.value=[];const result=await listLauncherAiModels(config());if(!Array.isArray(result)||!result.length)throw Error('供应商未返回可选模型，请检查地址与密钥权限后重试');models.value=result;note.value=`已读取 ${models.value.length} 个模型；请选择用途模型并保存。`;});}
function test(){return operation(async()=>{await optimizeLauncherPrompt('用三句话介绍如何整理书桌。');note.value='连接成功，优化模型已返回完整文本。';});}
function clear(){return operation(async()=>{await clearLauncherAiConfig();apply({});models.value=[];note.value='本机 AI 配置和密钥已清除。';});}
onMounted(()=>{if(native)load();});
</script>
<style scoped>
.launcher-ai-settings p,.launcher-ai-settings small {font-size:12px;color:var(--muted);line-height:1.7}.launcher-ai-settings fieldset{border:0;padding:0;margin:0;display:grid;gap:16px;min-width:0}.launcher-ai-settings .modal-actions{flex-wrap:wrap;justify-content:flex-start}.launcher-ai-settings h4{margin:0}.launcher-ai-settings small{display:block;margin-top:6px}
</style>
