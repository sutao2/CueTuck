import {invokeCommand} from './tauri.js';
let transport=null;export function setIdentityTransport(value){transport=value;}
export async function identityRequest(action,config={}){
  if(!['options','request','confirm'].includes(action))throw Error('不支持的验证动作');
  if(transport)return transport(action,config);
  if(typeof window!=='undefined'&&window.__TAURI_INTERNALS__)return invokeCommand('identity_request',{action,config});
  throw Error('请在桌面应用中完成邮箱验证，浏览器预览不创建账号');
}
