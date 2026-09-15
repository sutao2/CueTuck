import { getSession } from './session.js';
import { invokeCommand } from './tauri.js';
import { apiBase } from '../../../shared/apiBase.js';
let transport;
export const setSkillMarketTransport = value => { transport = value; };
export async function skillMarket(action, { id, query, body } = {}) {
  const accessToken=getSession().accessToken;
  if (['submit','mine','withdraw'].includes(action)&&!accessToken&&!transport) throw new Error('请先登录，填写昵称后再发布 Skill');
  if(transport)return transport(action,{id,query,body});
  if(window.__TAURI_INTERNALS__)return invokeCommand('skill_market_request',{action,id:id||null,query:query||null,body:body||null,access_token:accessToken||null});
  const path=action==='mine'?'/v1/skills/mine':['detail','bundle','withdraw'].includes(action)?`/v1/skills/${encodeURIComponent(id)}${action==='detail'?'':'/'+action}`:'/v1/skills';
  const response=await fetch(`${apiBase()}${path}${query?'?'+new URLSearchParams(query):''}`,{method:['submit','withdraw'].includes(action)?'POST':'GET',headers:{'Content-Type':'application/json',...(accessToken?{Authorization:`Bearer ${accessToken}`}:{})},body:body?JSON.stringify(body):undefined,signal:AbortSignal.timeout(90000)});
  if(!response.ok)throw new Error(({401:'请先登录后继续',403:'当前账号或站点不允许此操作',404:'Skill 不存在或已下架',409:'状态已变化，请刷新重试',422:'请先在账号设置中填写昵称',429:'今日发布次数已用完'})[response.status]||`Skill 社区请求失败（${response.status}）`);
  return response.json();
}
export const skillStatus = value => ({pending:'待审核',approved:'已上架',rejected:'已退回',withdrawn:'已撤回 / 下架'})[value]||value;
