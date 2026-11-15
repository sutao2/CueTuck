import { invokeCommand } from './tauri.js';
import { copyLauncherText } from './paste.js';

export const clipTypes = { text:'文本', url:'网址', account:'账号', password:'密码', template:'模板' };
export const clipIcons = { text:'file', url:'globe', account:'user', password:'lock', template:'models' };
export function clipRequest(request) {
  return invokeCommand('quick_clip', {request});
}
export function findClips(query,limit=null, {kind=null, category_id=null}={}) {
  if (!window.__TAURI_INTERNALS__) return Promise.resolve([]);
  return clipRequest({action:'search',query,limit,kind,category_id});
}
export function clipCategories() {
  if (!window.__TAURI_INTERNALS__) return Promise.resolve({categories:[],total:0,uncategorized:0});
  return clipRequest({action:'categories'});
}
export async function copyClip(id) {
  // Do not use the prompt "last rendered" setting or variable substitution.
  await copyLauncherText(await clipRequest({action:'copy_value',id}));
}
