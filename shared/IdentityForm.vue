<template>
  <form class="identity-form" @submit.prevent="submit" :aria-busy="busy" data-testid="identity-form">
    <h3>{{ titles[mode] }}</h3><p class="identity-note">{{ mode==='invitation'?'请填写管理员邀请邮件中的验证码。':confirming?'请填写邮件中的验证码并设置新密码。':'先验证邮箱，验证码仅使用一次，有效期 30 分钟。' }}</p>
    <label>邮箱<input v-model="email" type="email" autocomplete="username" maxlength="254" required :disabled="busy" data-testid="identity-email"></label>
    <template v-if="confirming"><label>邮件验证码<input v-model="token" autocomplete="one-time-code" autocapitalize="off" spellcheck="false" maxlength="64" minlength="64" required :disabled="busy" data-testid="identity-code"></label><label>新密码<input v-model="password" type="password" autocomplete="new-password" minlength="12" maxlength="128" required :disabled="busy" data-testid="identity-new-password"></label><label>确认新密码<input v-model="confirmation" type="password" autocomplete="new-password" minlength="12" maxlength="128" required :disabled="busy" data-testid="identity-confirm-password"></label><small class="identity-note">12–128 个字符。验证成功后返回登录，不自动登录或保存验证码。</small></template>
    <p v-if="error" role="alert" class="identity-error">{{ error }}</p><p v-if="message" role="status" class="identity-note">{{ message }}</p>
    <p v-if="!confirming&&options&&!options.email_enabled" class="identity-note">站点尚未启用邮件服务，暂时不能发起验证。</p><p v-if="!confirming&&mode==='registration'&&options&&!options.registration_open" class="identity-note">站点暂未开放新账号注册。</p>
    <button class="identity-primary" type="submit" :disabled="busy||(!confirming&&!canRequest)" data-testid="identity-submit">{{ busy?'处理中…':confirming?'验证并设置密码':'发送验证邮件' }}</button>
    <button v-if="!confirming" type="button" :disabled="busy" @click="confirming=true;error='';message=''">已有验证码</button><button v-else-if="mode!=='invitation'" type="button" :disabled="busy" @click="confirming=false;token='';password='';confirmation=''">重新请求验证码</button>
    <button v-if="!confirming&&!options" type="button" :disabled="busy" @click="load">重试加载</button><button type="button" :disabled="busy" @click="$emit('back')">返回登录</button>
  </form>
</template>
<script setup>
import {computed,onMounted,ref} from 'vue';
const props=defineProps({mode:{type:String,required:true},request:{type:Function,required:true},initialEmail:{type:String,default:''}});const emit=defineEmits(['back','done','busy-change']);
const titles={registration:'创建账号',reset:'找回密码',invitation:'接受管理员邀请'};const email=ref(props.initialEmail),token=ref(''),password=ref(''),confirmation=ref(''),confirming=ref(props.mode==='invitation'),options=ref(null),busy=ref(false),error=ref(''),message=ref('');
const canRequest=computed(()=>options.value?.email_enabled&&(props.mode!=='registration'||options.value.registration_open));
async function load(){try{options.value=await props.request('options');error.value='';}catch(e){error.value=e.message;}}
async function submit(){if(busy.value)return;if(confirming.value&&password.value!==confirmation.value){error.value='两次密码不一致';return;}if(!confirming.value&&!canRequest.value)return;busy.value=true;emit('busy-change',true);error.value='';message.value='';try{
    if(confirming.value){const result=await props.request('confirm',{kind:props.mode,email:email.value.trim(),token:token.value.trim(),new_password:password.value});if(result.status!=='verified')throw Error('服务端未确认验证完成');token.value='';password.value='';confirmation.value='';emit('done',email.value.trim());}
    else{const result=await props.request('request',{kind:props.mode,email:email.value.trim()});message.value=result.message;confirming.value=true;}
  }catch(e){error.value=e.message;}finally{busy.value=false;emit('busy-change',false);}}
onMounted(load);
</script>
<style scoped>
.identity-form{display:grid;gap:14px;padding:24px;max-width:520px;margin:0 auto;width:100%;box-sizing:border-box}.identity-form h3{margin:0;font-size:19px}.identity-form label{display:grid;gap:7px;font-size:13px}.identity-form input{box-sizing:border-box;min-width:0;width:100%;font:inherit;color:inherit;background:var(--surface,#fff);border:1px solid var(--border-color,#d9dfe1);border-radius:8px;padding:11px 12px}.identity-note{font-size:12px;color:var(--text-secondary,#78848a);line-height:1.6;margin:0}.identity-error{color:#b34236;font-size:13px;margin:0}.identity-form button{font:inherit;font-size:13px;border:1px solid var(--border-color,#d9dfe1);border-radius:8px;padding:10px;background:var(--surface,#fff);color:inherit;cursor:pointer}.identity-form .identity-primary{background:#19776c;color:#fff;border-color:#19776c}.identity-form button:disabled{opacity:.5;cursor:default}
</style>
