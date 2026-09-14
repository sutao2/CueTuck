import { invokeCommand } from './tauri.js';
import { apiBase } from '../../../shared/apiBase.js';
import { getSession } from './session.js';
const pending = new Map();
export function sourceLanguage(text) {
  // Reuse only clear monolingual originals; mixed/ambiguous text remains translatable.
  const clean = text.replace(/```[\s\S]*?```|`[^`]+`|https?:\/\/\S+|\{[^}]*\}/g, '');
  const han = (clean.match(/\p{Script=Han}/gu) || []).length;
  const latin = (clean.match(/[a-zA-Z]/g) || []).length;
  if (han > 0 && han >= latin / 2) return 'zh';
  if (!han && latin > 4) return 'en';
  return '';
}
function native() { return Boolean(window.__TAURI_INTERNALS__); }
export async function localVersion(text, target, generate = false) {
  if (sourceLanguage(text) === target) return { text, original: true };
  if (!native()) { if (!generate) return null; throw Error('请在桌面客户端的设置 → AI 与模型中配置本地翻译模型'); }
  return invokeCommand(generate ? 'translate_local_prompt' : 'get_prompt_translation', { text, target });
}
export async function squareVersions(id, target = null) {
  const token = getSession().accessToken || null;
  if (native()) return invokeCommand('square_translations', { id, target, access_token: token });
  const response = await fetch(`${apiBase()}/v1/square/items/${encodeURIComponent(id)}/translations${target ? `/${target}` : ''}`, { method: target ? 'POST' : 'GET', headers: token ? { Authorization: `Bearer ${token}` } : {} });
  if (!response.ok) throw Error(({401:'请先登录再生成广场译文',429:'今日翻译次数已用完，请稍后重试',503:'广场翻译暂时暂停，请稍后重试',404:'该提示词已不可用'})[response.status] || '读取翻译失败，请重试');
  return response.json();
}
export function translateOnce(text, target) {
  const key = JSON.stringify([text, target]);
  if (!pending.has(key)) pending.set(key, localVersion(text, target, true).finally(() => pending.delete(key)));
  return pending.get(key);
}
