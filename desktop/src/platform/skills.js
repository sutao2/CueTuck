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
  const groups = new Map();
  for (const skill of skills) {
    if (!groups.has(skill.key)) groups.set(skill.key, {...skill, installations:[]});
    groups.get(skill.key).installations.push(skill);
  }
  return [...groups.values()];
}
