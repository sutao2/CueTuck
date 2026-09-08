import { apiBase } from "../../shared/apiBase.js";
const API_BASE=apiBase();let transport=null;export function setIdentityTransport(value){transport=value;}
export async function identityRequest(action,config={}){
  if(!['options','request','confirm'].includes(action))throw Error('不支持的验证动作');if(transport)return transport(action,config);
  const response=await fetch(`${API_BASE}/v1/session/identity/${action}`,{method:action==='options'?'GET':'POST',headers:{'content-type':'application/json'},...(action==='options'?{}:{body:JSON.stringify(config)})});
  if(!response.ok)throw Error(response.status===429?'操作频繁，请一分钟后重试':response.status===503?'站点邮件服务暂不可用':response.status===403?'站点暂未开放新账号注册':response.status===400?'验证码无效或已过期，或密码不符合 12–128 字符要求':'邮箱验证请求失败，请稍后重试');
  return response.json();
}
