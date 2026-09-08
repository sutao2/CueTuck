import { getSession } from './session.js';
import { invokeCommand } from './tauri.js';
let transport=null;
export function setReportTransport(value){transport=value;}
export async function reportRequest(config=null,offset=0){
  const token=getSession().accessToken;if(!token)throw Error('举报需要登录，请先登录后重试');
  let result;
  if(transport)result=await transport({config,offset});
  else if(window.__TAURI_INTERNALS__)result=await invokeCommand('square_reports',{access_token:token,config,offset});
  else {const response=await fetch(`http://127.0.0.1:8787/v1/reports?offset=${offset}`,{method:config?'POST':'GET',headers:{authorization:`Bearer ${token}`,'content-type':'application/json'},...(config?{body:JSON.stringify(config)}:{})});if(!response.ok)throw Error(response.status===429?'每天最多提交 20 件举报，请稍后重试':response.status===401?'登录已失效，请重新登录':response.status===404?'该内容已不可举报':'举报请求失败，请重试');result=await response.json();}
  if(getSession().accessToken!==token)throw Error('会话已变化，请重新打开举报');return result;
}
