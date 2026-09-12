<template>
  <section class="panel site-settings"><div class="panel-heading"><h2>社区站点</h2><button :disabled="busy||loading" @click="reload">重新加载</button></div><p v-if="error" class="error-banner" role="alert">{{ error }}</p><p v-if="message" class="success-message" role="status">{{ message }}</p><p v-if="loading" class="empty-state">正在加载…</p>
    <form v-else-if="config" @submit.prevent="save" data-testid="settings-panel"><fieldset :disabled="busy"><div class="site-grid">
      <label>站点名称<input v-model="config.name" required maxlength="60" data-testid="site-name"></label><label>支持邮箱<input v-model="config.support_email" type="email" maxlength="254"></label>
      <label class="wide">站点说明<textarea v-model="config.description" rows="2" maxlength="300"></textarea></label><label class="wide">标识图片 URL<input v-model="config.logo_url" type="url" placeholder="https://…" maxlength="2048"><small>可留空；仅 HTTPS 图片，展示在社区区域，不替换用户本地应用图标。</small></label>
      <label class="toggle"><input v-model="config.square_public" type="checkbox" data-testid="setting-square-public">允许匿名浏览广场</label><label class="toggle"><input v-model="config.publishing_open" type="checkbox" data-testid="site-publishing">允许新投稿</label>
      <p class="wide muted">关闭投稿仅限制新的云端投稿，本地编辑、启动器和已有内容审核不受影响。<button type="button" class="text-button" @click="$emit('registration')">管理注册策略与邀请 →</button></p>
      <label class="wide">广场公告<textarea v-model="config.announcement" rows="4" maxlength="2000" placeholder="纯文本公告，留空不展示" data-testid="site-announcement"></textarea></label>
      <label>开始时间（本地时区，可留空）<input :value="localTime(config.announcement_start)" type="datetime-local" @input="setTime('announcement_start',$event.target.value)"></label><label>结束时间（本地时区，可留空）<input :value="localTime(config.announcement_end)" type="datetime-local" @input="setTime('announcement_end',$event.target.value)"></label>
    </div><footer><span class="muted">版本 {{ config.revision }} · 客户端重新加载广场时更新</span><button class="primary" :disabled="!hasUnsavedChanges" data-testid="settings-save">{{ busy?'正在保存…':'保存设置' }}</button></footer></fieldset></form>
  </section>
</template>
<script setup>
import {computed,onMounted,ref} from 'vue';import {getSiteConfig,saveSiteConfig} from './adminApi.js';
const emit=defineEmits(['busy-change','registration']);const config=ref(null),saved=ref(''),loading=ref(false),busy=ref(false),error=ref(''),message=ref('');const hasUnsavedChanges=computed(()=>!!config.value&&JSON.stringify(config.value)!==saved.value);defineExpose({hasUnsavedChanges,isBusy:busy});
function accept(value){if(!Number.isInteger(value?.revision)||typeof value?.square_public!=='boolean'||typeof value?.name!=='string')throw Error('设置响应无效，请重新加载');config.value=value;saved.value=JSON.stringify(value);}
async function load(){loading.value=true;error.value='';try{accept(await getSiteConfig());}catch(e){error.value=e.message;}finally{loading.value=false;}}
function reload(){if(hasUnsavedChanges.value&&!window.confirm('放弃未保存的站点配置？'))return;load();}
function localTime(value){if(!value)return '';const date=new Date(value);return new Date(date.getTime()-date.getTimezoneOffset()*60000).toISOString().slice(0,16);}
function setTime(key,value){config.value[key]=value?new Date(value).toISOString():null;}
async function save(){if(busy.value||!hasUnsavedChanges.value)return;error.value='';message.value='';busy.value=true;emit('busy-change',true);try{const result=await saveSiteConfig({...config.value});if(result.revision!==config.value.revision+1)throw Error('服务端未确认保存');accept(result);message.value='设置已保存';}catch(e){error.value=e.message;}finally{busy.value=false;emit('busy-change',false);}}
onMounted(load);
</script>
<style scoped>
.site-settings form{padding:24px}.site-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:22px}.site-grid .wide{grid-column:1/-1}.site-grid label:not(.toggle){display:grid;gap:8px}.site-grid input:not([type=checkbox]),.site-grid textarea{width:100%;min-width:0;font:inherit;color:inherit;box-sizing:border-box;border:1px solid var(--line-strong);border-radius:7px;padding:10px 12px}.site-grid small{color:#7a838d;font-size:12px}.site-grid .toggle{display:flex;align-items:center;gap:10px}.site-settings footer{display:flex;align-items:center;justify-content:space-between;gap:16px;margin-top:28px}.site-settings .text-button{border:0;background:transparent;color:#19776c;padding:5px 0;display:block}@media(max-width:650px){.site-grid{grid-template-columns:1fr}.site-settings footer{align-items:flex-start;flex-direction:column}}
</style>
