import { apiBase } from "../../shared/apiBase.js";
const API_BASE=apiBase();
export async function adminIdentityRequest(action,config={}) {
  if(!['options','request','confirm'].includes(action))throw Error('不支持的验证动作');
  if(action!=='options'&&!['reset','invitation'].includes(config.kind))throw Error('管理台不提供公开管理员注册');
  const response=await fetch(`${API_BASE}/v1/session/identity/${action}`,{method:action==='options'?'GET':'POST',headers:{'content-type':'application/json'},...(action==='options'?{}:{body:JSON.stringify(config)})});
  if(!response.ok)throw Error(response.status===429?'操作频繁，请稍后重试':response.status===503?'站点尚未启用邮件服务':response.status===400?'验证码无效或已过期，或密码不符合 12–128 字符要求':'邮箱验证失败，请稍后重试');
  return response.json();
}
