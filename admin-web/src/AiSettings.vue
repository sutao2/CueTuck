<template>
  <section class="panel ai-settings">
    <div class="panel-heading"><h2>{{ mode === 'models' ? '审核模型接口' : '版本化审核 Skills' }}</h2><span class="badge">仅所有者 · 版本 {{ draft?.revision ?? '—' }}</span><button :disabled="busy || loading" @click="reload">重新加载</button></div>
    <p class="muted">{{ mode === 'models' ? '用于主动投稿和明确发起的样本测试，不读取本地私有库。密钥仅在后端加密保存；配置并不代表测试通过。' : 'Skill 是审核指令和路由规则，不会执行脚本。每次保存生成不可覆盖的新版本，历史结论保留原版本。' }}</p>
    <p v-if="error" role="alert" class="error-banner">{{ error }}</p><p v-if="message" role="status" class="success-message">{{ message }}</p><p v-if="loading" role="status" class="empty-state">正在加载…</p>
    <form v-else-if="draft" @submit.prevent="save"><fieldset :disabled="busy">
      <div v-if="mode === 'models'" class="ai-cards">
        <p v-if="!draft.models.length" class="empty-state">尚未配置审核模型。要求 AI 的投稿会进入人工队列。</p>
        <article v-for="(model,index) in draft.models" :key="model.id" class="ai-card">
          <div class="panel-heading"><h3>{{ model.name || '新模型' }}</h3><span class="badge">{{ modelState(model.id) }}</span><button type="button" @click="removeModel(index)">移除</button></div>
          <div class="ai-fields"><label>显示名称<input v-model="model.name" maxlength="80" required :data-testid="`ai-model-name-${index}`"></label><label>模型<SearchableSelect v-model="model.model" :options="modelOptions(model)" aria-label="审核模型" placeholder="获取模型后选择" /><button type="button" :disabled="discovering === model.id || !model.endpoint" @click="discover(model)">{{ discovering === model.id ? '正在获取…' : '获取模型列表' }}</button></label>
            <label class="wide">完整 Chat Completions 地址<input v-model="model.endpoint" @input="delete modelLists[model.id]" type="url" required placeholder="https://…/v1/chat/completions"></label><label>API 密钥<input v-model="secrets[model.id]" @input="delete modelLists[model.id]" type="password" autocomplete="new-password" :placeholder="configuredKeys[model.id] ? '已加密保存，留空保留' : '首次启用必须填写'" maxlength="4096"></label><label>单模型超时（秒）<input v-model.number="model.timeout_seconds" type="number" min="1" max="15" required></label>
          </div><div class="ai-checks"><label><input v-model="model.enabled" type="checkbox">启用</label><label><input v-model="model.vision" type="checkbox">允许发送稿件图片给此视觉模型</label><label><input v-model="model.json_mode" type="checkbox">请求 JSON object 模式</label><label><input v-model="model.redact" type="checkbox">发送前遮盖文本邮箱和常见令牌（不遮盖图片）</label></div><p class="muted">图片限 PNG/JPEG/WebP、最多 4 张和 10 MiB，只读取稿件选中的附件；需在 Skill 路由中选择视觉模型。图片内敏感信息不会自动遮盖。附件最终公开仍须人工确认。</p>
          <small class="muted">仅接受公网 HTTPS，不跟随重定向。百炼使用非思考 JSON 模式；不兼容的接口会显示测试失败。文本检查不代表完成图片审核。</small>
        </article><button type="button" data-testid="ai-add-model" @click="addModel">添加审核模型</button>
      </div>
      <div v-else class="ai-cards">
        <article v-for="(skill,index) in draft.skills" :key="skill.id" class="ai-card">
          <div class="panel-heading"><h3>{{ skill.name || '新 Skill' }}</h3><label><input v-model="skill.enabled" type="checkbox">启用</label><button type="button" @click="draft.skills.splice(index,1)">移除</button></div>
          <div class="ai-fields"><label>名称<input v-model="skill.name" maxlength="80" required :data-testid="`ai-skill-name-${index}`"></label><label>审核路由<select v-model="skill.route"><option value="fallback">主备：按顺序尝试</option><option value="consensus">双模型：结论一致</option><option value="majority">三模型：多数结论</option></select></label>
            <label class="wide">审核指令<textarea v-model="skill.instruction" rows="5" maxlength="10000" required /></label>
          </div><div class="ai-checks"><label><input v-model="skill.kinds" value="prompt" type="checkbox">提示词</label><label><input v-model="skill.kinds" value="collection" type="checkbox">合集</label></div>
          <label>模型顺序（主备按此顺序；一致需 2 个，多数需 3 个）</label><div class="ai-model-order"><label v-for="position in 3" :key="position">{{ position }}<select :value="skill.models[position-1] || ''" @change="setModel(skill,position-1,$event.target.value)"><option value="">不使用</option><option v-for="m in draft.models" :key="m.id" :value="m.id">{{ m.name }}{{ m.enabled ? '' : '（停用）' }}</option></select></label></div>
          <details><summary>适用分类（不选代表全部）· 已选 {{ skill.categories.length }}</summary><div class="ai-categories"><label v-for="category in categories" :key="category.id"><input v-model="skill.categories" type="checkbox" :value="category.id">{{ category.name }}</label></div></details>
        </article><button type="button" data-testid="ai-add-skill" @click="addSkill">添加 Skill</button>
      </div>
      <footer class="ai-save"><label>当前管理员密码<input v-model="password" type="password" autocomplete="current-password" maxlength="512" data-testid="ai-password"></label><button class="primary" data-testid="ai-save" :disabled="!hasUnsavedChanges || !password">{{ busy ? '处理中…' : '保存新版本' }}</button></footer>
    </fieldset></form>
    <section v-if="draft" class="ai-test"><h3>测试已保存的配置</h3><p class="muted">测试会发送下方样本到所选接口，最多等待 25 秒。不要填写真实密码或私密内容。草稿不会参与测试。</p>
      <div class="ai-fields"><label>已保存 Skill<select v-model="testSkill" :disabled="busy"><option v-for="s in savedSkills" :key="s.id" :value="s.id">{{ s.name }}</option></select></label><label>测试范围<select v-model="testModel" :disabled="busy"><option value="">完整 Skill 路由</option><option v-for="m in savedModels" :key="m.id" :value="m.id">仅 {{ m.name }}</option></select></label><label class="wide">样本<textarea v-model="sample" rows="3" maxlength="8000" :disabled="busy" data-testid="ai-sample" /></label></div>
      <label class="ai-checks"><input v-model="imageSample" type="checkbox" :disabled="busy" data-testid="ai-image-sample">附带固定测试图（纯色方块，不读取个人图片）</label>
      <button :disabled="busy || hasUnsavedChanges || !sample.trim() || !testSkill" data-testid="ai-test" @click="test">发送样本并测试</button><small v-if="hasUnsavedChanges" class="muted">先保存或重新加载，避免测试旧配置。</small>
      <div v-if="testResult" class="notice" role="status"><strong>{{ testResult.verdict ? (testResult.image_sample ? '收到有效图文审核结果' : '收到有效文本审核结果') : '测试未通过' }}</strong><p>{{ testResult.error }}</p><p v-if="testResult.verdict">{{ decisionNames[testResult.verdict.decision] }} · 风险分 {{ testResult.verdict.risk_score }}（配置版本 {{ testResult.revision }}）</p><ul><li v-for="(result,index) in testResult.models" :key="index">{{ result.id }}：{{ result.error || decisionNames[result.result?.decision] }}</li><li v-for="reason in testResult.verdict?.reasons || []" :key="reason">{{ reason }}</li></ul></div>
    </section>
    <section v-if="draft" class="ai-history"><button :disabled="busy || historyLoading" @click="loadHistory">查看最近 50 个版本</button><details v-for="entry in history" :key="entry.revision"><summary>版本 {{ entry.revision }} · {{ entry.actor }} · {{ new Date(entry.created_at).toLocaleString() }}</summary><div v-for="skill in entry.data.skills" :key="skill.id"><strong>{{ skill.name }} · {{ skill.route }}</strong><p class="ai-instruction">{{ skill.instruction }}</p><p class="muted">模型：{{ skill.models.join(' → ') || '未指定' }}；{{ skill.enabled ? '启用' : '停用' }}</p></div></details></section>
  </section>
</template>
<script setup>
import SearchableSelect from '../../desktop/src/components/SearchableSelect.vue';
import {computed,onMounted,ref} from 'vue';
import {discoverAiModels,getAiConfig,saveAiConfig,testAiConfig,getAiHistory,listCatalog} from './adminApi.js';
defineProps({mode:{type:String,default:'models'}});const emit=defineEmits(['busy-change']);
const draft=ref(null),saved=ref(''),secrets=ref({}),configuredKeys=ref({}),password=ref(''),tests=ref([]),categories=ref([]),loading=ref(false),busy=ref(false),error=ref(''),message=ref('');
const modelLists=ref({}),discovering=ref('');
function modelOptions(model){const ids=modelLists.value[model.id]||[];return [...new Set([...ids,model.model].filter(Boolean))].map(id=>({value:id,label:ids.includes(id)?id:`${id}（已有配置，待核实）`}));}
async function discover(model){setBusy(true);discovering.value=model.id;error.value='';try{const result=await discoverAiModels({endpoint:model.endpoint,key:secrets.value[model.id]||'',model_id:model.id});if(result.error)throw Error(result.error);modelLists.value[model.id]=result.models;message.value=`已获取 ${result.models.length} 个模型，选择后保存。`;}catch(e){error.value=e.message;}finally{discovering.value='';setBusy(false);}}
const imageSample=ref(false);
const testSkill=ref(''),testModel=ref(''),sample=ref(''),testResult=ref(null),history=ref([]),historyLoading=ref(false);
const decisionNames={approve:'建议通过',reject:'建议驳回',manual:'需人工复核'};
const savedSkills=computed(()=>saved.value?JSON.parse(saved.value).skills:[]),savedModels=computed(()=>saved.value?JSON.parse(saved.value).models:[]);
const hasUnsavedChanges=computed(()=>!!draft.value&&(JSON.stringify(draft.value)!==saved.value||Object.values(secrets.value).some(Boolean)));defineExpose({hasUnsavedChanges,isBusy:busy});
function accept(value){if(!Number.isInteger(value?.revision)||!Array.isArray(value.models)||!Array.isArray(value.skills))throw Error('模型配置响应无效');configuredKeys.value=Object.fromEntries(value.models.map(m=>[m.id,m.secret_configured]));draft.value={revision:value.revision,models:value.models.map(({secret_configured,...m})=>m),skills:value.skills};saved.value=JSON.stringify(draft.value);secrets.value={};password.value='';tests.value=value.tests||[];testSkill.value=value.skills.some(s=>s.id===testSkill.value)?testSkill.value:value.skills[0]?.id||'';testModel.value='';}
async function load(){loading.value=true;error.value='';try{const [config,catalog]=await Promise.all([getAiConfig(),listCatalog('categories')]);categories.value=catalog.items||[];accept(config);}catch(e){error.value=e.message;}finally{loading.value=false;}}
function reload(){if(hasUnsavedChanges.value&&!window.confirm('放弃未保存的配置？'))return;load();}
function addModel(){draft.value.models.push({id:crypto.randomUUID(),name:'',endpoint:'',model:'',enabled:false,vision:false,json_mode:true,redact:true,timeout_seconds:10});}
function removeModel(index){if(!window.confirm('移除模型也会清除当前 Skill 对它的引用。历史版本仍保留，确认移除？'))return;const id=draft.value.models[index].id;draft.value.models.splice(index,1);delete secrets.value[id];for(const skill of draft.value.skills)skill.models=skill.models.filter(m=>m!==id);}
function addSkill(){draft.value.skills.push({id:crypto.randomUUID(),name:'',instruction:'',enabled:false,kinds:['prompt','collection'],categories:[],route:'fallback',models:[]});}
function setModel(skill,index,id){const models=[...skill.models];models[index]=id;skill.models=models.filter(Boolean);}
function modelState(id){const test=tests.value.find(t=>t.revision===draft.value.revision&&t.model_id===id);return test?(test.success?(test.image_sample?'当前版本图文测试通过':'当前版本文本测试通过'):'当前版本测试失败'):'当前版本未单独测试';}
function setBusy(value){busy.value=value;emit('busy-change',value);}
async function save(){if(busy.value||!hasUnsavedChanges.value||!password.value)return;setBusy(true);error.value='';message.value='';try{const result=await saveAiConfig({...JSON.parse(JSON.stringify(draft.value)),secrets:{...secrets.value},current_password:password.value});if(result.revision!==draft.value.revision+1)throw Error('服务端未确认新版本');accept(result);testResult.value=null;message.value='新版本已保存；旧版本测试不代表新配置可用。';}catch(e){error.value=e.message;}finally{password.value='';setBusy(false);}}
async function test(){if(busy.value||hasUnsavedChanges.value)return;if(!window.confirm('将当前样本发送到已保存的审核模型接口，确认测试？'))return;setBusy(true);error.value='';testResult.value=null;try{testResult.value=await testAiConfig({revision:draft.value.revision,skill_id:testSkill.value,model_id:testModel.value||null,text:sample.value,image_sample:imageSample.value});const config=await getAiConfig();if(config.revision===draft.value.revision)tests.value=config.tests||[];else error.value='配置在测试期间已变化，请重新加载。';}catch(e){error.value=e.message;}finally{setBusy(false);}}
async function loadHistory(){historyLoading.value=true;try{history.value=(await getAiHistory()).items||[];}catch(e){error.value=e.message;}finally{historyLoading.value=false;}}
onMounted(load);
</script>
<style scoped>
.ai-settings>p,.ai-cards,.ai-save,.ai-test,.ai-history{margin:20px 24px}.ai-card{border:1px solid var(--line,#e4e7eb);border-radius:12px;padding:18px;margin-bottom:16px;min-width:0}.ai-card .panel-heading{padding:0 0 16px}.ai-fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}.ai-fields .wide{grid-column:1/-1}.ai-checks{display:flex;flex-wrap:wrap;gap:18px;margin:18px 0}.ai-checks label,.ai-categories label{display:flex;align-items:center;gap:8px}.ai-save{display:flex;align-items:end;justify-content:flex-end;gap:16px;padding-top:20px;border-top:1px solid #e4e7eb}.ai-save label{max-width:300px}.ai-test,.ai-history{padding-top:24px;border-top:1px solid #e4e7eb}.ai-test>button{margin:16px 12px 0 0}.ai-model-order{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin:12px 0}.ai-categories{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;padding:16px 0}.ai-history details{padding:16px 0;border-bottom:1px solid #e4e7eb}.ai-instruction{white-space:pre-wrap;overflow-wrap:anywhere}.ai-settings input:not([type=checkbox]),.ai-settings textarea,.ai-settings select{width:100%;min-width:0}.ai-settings fieldset{min-width:0}@media(max-width:650px){.ai-fields,.ai-model-order,.ai-categories{grid-template-columns:1fr}.ai-save{align-items:stretch;flex-direction:column}.ai-cards,.ai-save,.ai-test,.ai-history{margin:16px}.ai-card{padding:12px}}
</style>
