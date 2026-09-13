import { invokeCommand } from './tauri.js';
let transport = null;
export function setSkillsTransportForTests(value) { transport = value; }
export function skillsSupported() { return Boolean(window.__TAURI_INTERNALS__ || transport); }
export async function skillsRequest(request, requestId = crypto.randomUUID()) {
  if (transport) return transport(request, requestId);
  if (!skillsSupported()) throw new Error('请在 CueTuck 桌面客户端管理本机 Skills');
  return invokeCommand('skills_command', { request, requestId });
}
export async function chooseSkillsDirectory() {
  if (transport) return transport({action:'choose_directory'});
  return invokeCommand('skills_choose_directory');
}
export async function cancelSkillsRequest(requestId) { if (!transport && window.__TAURI_INTERNALS__) await invokeCommand('skills_cancel', {requestId}); }
export async function listenSkillsProgress(handler) {
  if (!window.__TAURI_INTERNALS__) return () => {};
  const { listen } = await import('@tauri-apps/api/event');
  return listen('skills-progress', event => handler(event.payload));
}
export function groupSkills(skills) {
  const parents=skills.map((_,i)=>i),identities=new Map();
  function find(i){while(parents[i]!==i){parents[i]=parents[parents[i]];i=parents[i];}return i;}
  skills.forEach((skill,index)=>{
    const keys=[`path:${skill.physical_path||skill.path}`];
    if(skill.source)keys.push(`source:${skill.source.repo.toLowerCase()}:${skill.source.directory}`);
    for(const key of keys){if(identities.has(key))parents[find(index)]=find(identities.get(key));else identities.set(key,index);}
  });
  const groups=new Map();
  skills.forEach((skill,index)=>{
    const id=find(index);
    if(!groups.has(id))groups.set(id,{...skill,installations:[]});
    const group=groups.get(id);group.installations.push(skill);if(!group.source&&skill.source)group.source=skill.source;
  });
  return [...groups.values()];
}
